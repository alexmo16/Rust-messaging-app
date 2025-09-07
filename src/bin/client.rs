use std::io::{ Read, Write };
use std::net::TcpStream;
use std::thread;
use rust_chat::utils::constants;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to server");

    let read_stream = stream.try_clone()?; // Clone for reading

    // Receive and print messages from the server
    let receive_thread = thread::spawn(move || {
        let mut buffer = [0; constants::MAX_CHARS];
        let mut stream = read_stream;
        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        println!("Server closed connection");
                        return;
                    }
                    let message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                    println!("{}", message);
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    return;
                }
            }
        }
    });
    
    // Send messages to the server
    loop {
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");

        if input.trim() == "exit" {
            break;
        }

        stream.write_all(input.as_bytes()).expect("Failed to write to stream");
    }

    receive_thread.join().unwrap(); // Wait for the receive thread to finish

    Ok(())
}
