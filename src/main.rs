use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

use std::io::Write;
use std::io::Read;

use std::thread;

use std::sync::mpsc::{channel, Sender, Receiver};

use std::sync::Arc;




enum Message {
    Connected{client: Arc<TcpStream>},
    Disconnected{client: Arc<TcpStream>},
    NewMessage{
        data: Vec<u8>,
        client: Arc<TcpStream>,
    }
}


struct Client {
    Conn: Arc<TcpStream>,
}



fn server(messages: Receiver<Message>){

    let mut  clients = HashMap::new();


    loop {

        let msg = messages.recv().expect("ERROR: Receiving message from client");

        match msg {


            Message::Connected{client} => {

                let address = client.peer_addr().unwrap();

             
                clients.insert(address.clone(), Client{Conn: client.clone()});

                println!("Client connected: {}", client.peer_addr().unwrap());

            }

            Message::Disconnected{client} => {

                let address = client.peer_addr().unwrap();

                clients.remove(&address);

                println!("Client disconnected: {}", client.peer_addr().unwrap());


            }

            Message::NewMessage{data, client} => {

                let address = client.peer_addr().unwrap();

                for (client_address, client) in clients.iter() {

                    if client_address != &address {

                        client.Conn.as_ref().write(&data).expect("ERROR: Sending message to client");

                    }

                }

             

            }


        }

    }



  


}






fn client(mut stream: Arc<TcpStream>, message: Sender<Message>) -> Result<(),()> {


    
    message.send(Message::Connected{client: stream.clone()}).map_err(|err| {

        eprintln!("ERROR: Sending message to server {err}");

       let _ =  message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

            eprintln!("ERROR: Sending message to server {err}");

        });


    })?;


    
    let mut buffer = [0; 1024];

    loop {

        let bytes_read = stream.as_ref().read(&mut buffer).map_err(|err| {

            eprintln!("ERROR: Reading from user {err}");

           let _ = message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

                eprintln!("ERROR: Sending message to server {err}");

            });

        })?;

        if bytes_read == 0 {

            message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

                eprintln!("ERROR: Sending message to server {err}");

            })?;

            return Ok(());

        }

        message.send(Message::NewMessage{data: buffer[0..bytes_read].to_vec(), client: stream.clone()}).map_err(|err| {

            eprintln!("ERROR: Sending message to server {err}");

        })?;

    }





}




fn main() -> Result<(),()> {

        let address = "127.0.0.1:3030";


        let listener = TcpListener::bind(address).map_err(|error| {
            eprintln!("Error binding to {address}: {error}");
        })?;



        println!("Listening on {address}");


        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) = channel();

        thread::spawn(|| server(message_receiver)); 


        for stream in listener.incoming() {

            match stream {

                Ok(stream) => {


                    let stream = Arc::new(stream);

                    let message_sender = message_sender.clone();

                    thread::spawn(|| client(stream, message_sender));

                }

                Err(err) => {
                    
                    eprintln!("ERROR: Accepting connections:{err}")

                }

            }


        }

    

    Ok(())
      
}
