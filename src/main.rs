use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::thread;

use redis_starter_rust::{eval, parse};

fn connection_loop(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 256];

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break; // connection was closed
        }

        let buffer = &buffer[..n];
        println!("RAW BUFFER: {:?}", String::from_utf8_lossy(buffer));
        let mut offset = 0;

        let mut r_values = parse(&buffer, &mut offset);
        // println!("PARSED: {:?}", r_values);
        eval(&mut r_values, &mut stream);
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                thread::spawn(move || {
                    connection_loop(stream).expect("error in connection loop");
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
