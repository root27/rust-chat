use std::net::{TcpListener, TcpStream};

use std::io::Write;

use std::thread;

use std::sync::mpsc::{channel, Sender, Receiver};




enum Message {
    Connected,
    Disconnected,
    NewMessage,
}





fn server(message: Receiver<Message>){

}






fn client(mut stream: TcpStream, message: Sender<Message>) -> Result<(),()> {


    let _ = writeln!(stream, "Hello from server").map_err(|err| {

        eprintln!("ERROR: Writing message to user {err}")

    });

    Ok(())




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
