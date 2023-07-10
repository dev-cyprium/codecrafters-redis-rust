use commands::{
    EchoCommand, GetCommand, PingCommand, RedisCommand, RedisValueBag, SetCommand, Task,
    UnknownCommand,
};
use std::io::prelude::*;
use std::net::TcpStream;

mod commands;
mod memory;

#[derive(Debug, Clone)]
pub enum RedisValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RedisValueBag>),
}

pub fn eval(val: &mut RedisValueBag, stream: &mut TcpStream) {
    let mut tasks = Vec::new();
    let mut tasks = match val.value {
        RedisValue::Array(ref mut args) => {
            let mut command: Box<dyn RedisCommand> = Box::new(UnknownCommand {});
            let mut command_idx = 0;
            for i in 0..args.len() {
                if !args[i].processed {
                    if i == 0 {
                        match args[i].value {
                            RedisValue::BulkString(ref s) => match s.to_lowercase().as_str() {
                                "ping" => command = Box::new(PingCommand {}),
                                "echo" => command = Box::new(EchoCommand { value: None }),
                                "set" => {
                                    command = Box::new(SetCommand {
                                        key: None,
                                        value: None,
                                    })
                                }
                                "get" => command = Box::new(GetCommand { key: None }),
                                _ => {}
                            },
                            _ => {}
                        }
                    } else {
                        command.set_arg(command_idx, args[i].value.clone());
                        command_idx += 1;
                    }
                    args[i].processed = true;
                }
            }
            tasks.extend(command.run());
            tasks
        }
        _ => vec![Task::NetworkError("-ERROR (invalid value)\r\n".to_string())],
    };

    for task in tasks.drain(..) {
        match task {
            Task::NetworkWrite(s) => {
                stream.write(s.as_bytes()).unwrap();
            }
            Task::NetworkError(s) => {
                stream.write(s.as_bytes()).unwrap();
            }
        }
    }
}

pub fn parse(buffer: &[u8], offset: &mut usize) -> RedisValueBag {
    if offset >= &mut buffer.len() {
        return RedisValueBag::new(RedisValue::Error("buffer too short".to_string()));
    }

    let r_val: RedisValue = match buffer[*offset] {
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
    };

    RedisValueBag::new(r_val)
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
