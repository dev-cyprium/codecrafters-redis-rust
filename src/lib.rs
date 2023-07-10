use std::io::prelude::*;
use std::net::TcpStream;

use commands::Command;

mod commands;

#[derive(Debug, Clone)]
pub enum RedisValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RedisValue>),
}

pub fn eval(val: &RedisValue, stream: &mut TcpStream, next_val: Option<&RedisValue>) {
    match val {
        RedisValue::SimpleString(s) => println!("SimpleString: {}", s),
        RedisValue::Error(s) => println!("Error: {}", s),
        RedisValue::Integer(i) => println!("Integer: {}", i),
        RedisValue::BulkString(s) => {
            let cloned_option = next_val.cloned();
            if let Some(cmd) = to_command(s.as_str(), cloned_option) {
                let tasks = cmd.run();
                for task in tasks {
                    match task {
                        commands::Task::NetworkWrite(s) => {
                            stream.write(s.as_bytes()).unwrap();
                        }
                        commands::Task::NetworkError(s) => {
                            stream.write(s.as_bytes()).unwrap();
                        }
                    }
                }
            }
        }
        RedisValue::Array(a) => {
            for (i, val) in a.iter().enumerate() {
                if i + 1 < a.len() {
                    let next_val = &a[i + 1];
                    eval(val, stream, Some(next_val));
                } else {
                    eval(val, stream, None);
                }
            }
        }
    }
}

pub fn parse(buffer: &[u8], offset: &mut usize) -> RedisValue {
    if offset >= &mut buffer.len() {
        return RedisValue::Error("buffer too short".to_string());
    }

    match buffer[*offset] {
        b'*' => {
            *offset += 1;
            let length = get_length(buffer, offset);
            *offset += 1; // skip \n
            let mut array = Vec::with_capacity(length as usize);
            for _ in 0..length {
                array.push(parse(buffer, offset));
            }
            RedisValue::Array(array)
        }
        b'$' => {
            *offset += 1;
            let length = get_length(buffer, offset);
            *offset += 1; // skip \n
            let mut bulk_string = Vec::with_capacity(length as usize);
            for _ in 0..length {
                bulk_string.push(buffer[*offset]);
                *offset += 1;
            }
            *offset += 2;
            RedisValue::BulkString(String::from_utf8_lossy(&bulk_string).to_string())
        }
        _ => RedisValue::Error("unknown type".to_string()),
    }
}

fn get_length(buffer: &[u8], offset: &mut usize) -> u32 {
    let mut length = 0;
    while buffer[*offset] != b'\r' {
        length = length * 10 + (buffer[*offset] - b'0') as u32;
        *offset += 1;
    }
    *offset += 1;
    length
}

fn to_command(str: &str, next: Option<RedisValue>) -> Option<Command> {
    match str.to_lowercase().as_str() {
        "ping" => Some(Command::new("ping".to_string(), vec![])),
        "echo" => Some(Command::new("echo".to_string(), vec![next])),
        _ => None,
    }
}
