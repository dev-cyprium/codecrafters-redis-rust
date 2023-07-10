use crate::RedisValue;

#[derive(Debug)]
pub struct Command {
    name: String,
    args: Vec<Option<RedisValue>>,
}

pub enum Task {
    NetworkWrite(String),
    NetworkError(String),
}

impl Command {
    pub fn new(name: String, args: Vec<Option<RedisValue>>) -> Self {
        Self { name, args }
    }

    pub fn run(&self) -> Vec<Task> {
        match self.name.as_str() {
            "ping" => self.ping(),
            "echo" => self.echo(),
            _ => vec![Task::NetworkError(
                "-ERROR (unknown command)\r\n".to_string(),
            )],
        }
    }

    fn ping(&self) -> Vec<Task> {
        vec![Task::NetworkWrite("+PONG\r\n".to_string())]
    }

    fn echo(&self) -> Vec<Task> {
        if let Some(RedisValue::BulkString(v)) = &self.args[0] {
            vec![Task::NetworkWrite(format!("+{}\r\n", v))]
        } else {
            vec![Task::NetworkError(
                "-ERROR (wrong number of arguments for 'echo' command)\r\n".to_string(),
            )]
        }
    }
}
