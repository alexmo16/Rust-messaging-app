use std::{ io::Read, net::{ TcpListener, TcpStream }, sync::mpsc::{ channel, Sender } };

fn handle_connection(mut stream: &TcpStream, sender: Sender<String>) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Connection closed by client");
                    return;
                }

                let message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                if let Err(e) = sender.send(message) {
                    println!("Failed to send message: {}", e);
                    return;
                }
            }
            Err(e) => {
                println!("Error reading from stream: {}", e);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running on 127.0.1:8080");

    let (sender, receiver) = channel::<String>();

    std::thread::spawn(move || {
        for message in receiver {
            println!("Received: {}", message);
        }
    });

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                println!("New connection established from {}", stream.peer_addr()?);

                let sender = sender.clone();

                std::thread::spawn(move || { handle_connection(&stream, sender) });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
