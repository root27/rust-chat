use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

use std::io::Write;
use std::io::Read;

use std::thread;

use std::sync::mpsc::{channel, Sender, Receiver};

use std::sync::Arc;

use std::time::{SystemTime, Duration};



const MESSAGE_RATE: f32 = 1.0;
const STRIKE_LIMIT: u8 = 3;
const BAN_DURATION: f32 = 10.0*60.0;



enum Message {
    Connected{client: Arc<TcpStream>},
    Disconnected{client: Arc<TcpStream>},
    NewMessage{
        data: Vec<u8>,
        client: Arc<TcpStream>,
    }
}


struct Client {
    conn: Arc<TcpStream>,
    last_message: SystemTime,
    strike_count: u8
}


fn server(messages: Receiver<Message>){

    let mut  clients = HashMap::new();

    let mut bann_list = HashMap::new();


    loop {

        let msg = messages.recv().expect("ERROR: Receiving message from client");

        match msg {


            Message::Connected{client} => {

                let address = client.peer_addr().unwrap();

                let author_ip = address.ip().to_string();

                let now = SystemTime::now();


                if bann_list.contains_key(&author_ip) {

                    let ban_time = bann_list.get(&author_ip).unwrap();

                    if now.duration_since(*ban_time).unwrap().as_secs_f32() < BAN_DURATION as f32{

                        client.as_ref().write("You are banned. Try again after {now.duration_since(*ban_time).unwrap().as_secs_f32()} seconds".as_bytes()).expect("ERROR: Sending message to client");
                        

                    } else {

                        bann_list.remove(&author_ip);

                        clients.insert(address.clone(), Client{conn: client.clone(),
                            last_message: SystemTime::now(),
                            strike_count: 0
                        });
        
                        println!("Client connected: {}", client.peer_addr().unwrap());
        

                        continue;

                    }

                } 

                    clients.insert(address.clone(), Client{conn: client.clone(),
                        last_message: SystemTime::now(),
                        strike_count: 0
                    });

                    println!("Client connected: {}", client.peer_addr().unwrap());

                
            }

            Message::Disconnected{client} => {

                let address = client.peer_addr().unwrap();

               

                clients.remove(&address);

                println!("Client disconnected: {}", client.peer_addr().unwrap());


            }

            Message::NewMessage{data, client} => {

                let address = client.peer_addr().unwrap();


                let author_ip = address.ip().to_string();


                let now = SystemTime::now();



                if clients.contains_key(&address) {

                    let  client = clients.get_mut(&address).unwrap();
                    
                    let time_since_last_message = now.duration_since(client.last_message).unwrap().as_secs_f32();

                    if time_since_last_message >= MESSAGE_RATE {

                        client.strike_count += 1;

                        if client.strike_count >= STRIKE_LIMIT {

                            bann_list.insert(author_ip, now);

                            let _ = client.conn.as_ref().write("You are banned".as_bytes());


                            client.conn.shutdown(std::net::Shutdown::Both).expect("ERROR: Shutting down client connection");

                            clients.remove(&address);

                            

                            println!("Client banned: {}",address);

                            return;

                        }

                    } else {

                        client.strike_count = 0;


                        println!("Received message from client {address} : {:?}", data);





                    }

                    client.last_message = now;

                }



                println!("Received message from client {address} : {:?}", data);


           
                for (client_address, client) in clients.iter() {

                    if client_address != &address {

                        client.conn.as_ref().write(&data).expect("ERROR: Sending message to client");

                    }

                }

            }

        }

    }

}



fn client(stream: Arc<TcpStream>, message: Sender<Message>) -> Result<(),()> {


    
    message.send(Message::Connected{client: stream.clone()}).map_err(|err| {

        eprintln!("ERROR: Sending message to server {err}");

       let _ =  message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

            eprintln!("ERROR: Sending message to server {err}");

        });


    })?;

    
    let mut buffer= Vec::new();

    buffer.resize(1024, 0);




    loop {

        let bytes_read = stream.as_ref().read(&mut buffer).map_err(|err| {

            eprintln!("ERROR: Reading from user {err}");

           let _ = message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

                eprintln!("ERROR: Sending message to server {err}");

            });

        })?;


        let cleaned_text = buffer[0..bytes_read].to_vec();

       
    


        if cleaned_text == b"exit\r\n" {

            message.send(Message::Disconnected{client: stream.clone()}).map_err(|err| {

                eprintln!("ERROR: Sending message to server {err}");

            })?;

            return Ok(());

        }

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
