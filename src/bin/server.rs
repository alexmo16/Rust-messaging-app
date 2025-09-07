use std::{io::Read, net::{TcpListener, TcpStream}, sync::mpsc::{channel}, thread};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use rust_chat::utils::constants;

fn handle_connection(mut stream: TcpStream, sender: Sender<String>, clients: Arc<Mutex<Vec<TcpStream>>>) {
    {
        // Add this client to the shared list
        let mut clients = clients.lock().unwrap();
        clients.push(stream.try_clone().expect("Failed to clone stream"));
    }

    let mut buffer = [0; constants::MAX_CHARS];

    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Connection closed by client");
                    return;
                }

                let message = String::from_utf8_lossy(&buffer[..bytes_read])
                    .trim()
                    .to_string();

                println!("received {}", message);

                if let Err(e) = sender.send(message) {
                    eprintln!("Failed to send message: {}", e);
                    return;
                }
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running on 127.0.1:8080");

    let (sender, receiver) = channel::<String>();

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    // Thread to broadcast messages to all clients
    {
        let clients = clients.clone();
        thread::spawn(move || {
            for msg in receiver {
                println!("Broadcasting message: {}", msg);
                let mut clients = clients.lock().unwrap();
                clients.retain(|stream| {
                    let mut stream = stream.try_clone().unwrap();

                    if let Err(e) = stream.write_all(&msg.as_bytes()) {
                        eprintln!("Failed to write to client: {}", e);
                        false // remove broken connection
                    }
                    else {
                        true
                    }
                });
            }
        });
    }

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                println!("New connection established from {}", stream.peer_addr()?);
                let clients = Arc::clone(&clients);
                let sender = sender.clone();
                thread::spawn(move || handle_connection(stream, sender, clients));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
