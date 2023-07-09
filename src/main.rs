use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};

fn connection_loop(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break; // connection was closed
        }

        stream.write(b"+PONG\r\n")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                connection_loop(stream)?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
