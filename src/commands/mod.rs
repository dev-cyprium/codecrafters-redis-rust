use crate::{
    memory::{get_value, set_value},
    RedisValue,
};

pub trait RedisCommand {
    fn run(&self) -> Vec<Task>;
    fn num_args(&self) -> usize;
    fn set_arg(&mut self, idx: usize, val: RedisValue);
}

#[derive(Debug, Clone)]
pub struct RedisValueBag {
    pub value: RedisValue,
    pub processed: bool,
}

impl RedisValueBag {
    pub fn new(value: RedisValue) -> Self {
        Self {
            value,
            processed: false,
        }
    }
}

pub struct PingCommand {}
pub struct EchoCommand {
    pub value: Option<RedisValue>,
}
pub struct SetCommand {
    pub key: Option<RedisValue>,
    pub value: Option<RedisValue>,
    pub ttl: Option<RedisValue>,
    pub cache: bool,
}
pub struct GetCommand {
    pub key: Option<RedisValue>,
}
pub struct UnknownCommand {}

impl RedisCommand for PingCommand {
    fn run(&self) -> Vec<Task> {
        vec![Task::NetworkWrite("+PONG\r\n".to_string())]
    }

    fn num_args(&self) -> usize {
        0
    }

    // has no args
    fn set_arg(&mut self, _idx: usize, _val: RedisValue) {}
}

impl RedisCommand for EchoCommand {
    fn run(&self) -> Vec<Task> {
        match &self.value {
            Some(RedisValue::BulkString(v)) => vec![Task::NetworkWrite(format!("+{}\r\n", v))],
            _ => vec![Task::NetworkError("-ERROR (invalid value)\r\n".to_string())],
        }
    }

    fn num_args(&self) -> usize {
        1
    }

    fn set_arg(&mut self, _idx: usize, val: RedisValue) {
        self.value = Some(val);
    }
}

impl RedisCommand for UnknownCommand {
    fn run(&self) -> Vec<Task> {
        vec![Task::NetworkError(
            ("-ERROR (unknown command)\r\n").to_string(),
        )]
    }

    fn num_args(&self) -> usize {
        0
    }

    fn set_arg(&mut self, _idx: usize, _val: RedisValue) {}
}

impl RedisCommand for GetCommand {
    fn run(&self) -> Vec<Task> {
        match &self.key {
            Some(RedisValue::BulkString(k)) => {
                if let Some(v) = get_value(k.as_str()) {
                    vec![Task::NetworkWrite(format!("+{}\r\n", v))]
                } else {
                    vec![Task::NetworkWrite("$-1\r\n".to_string())]
                }
            }
            _ => vec![Task::NetworkError("-ERROR (invalid value)\r\n".to_string())],
        }
    }

    fn num_args(&self) -> usize {
        1
    }

    fn set_arg(&mut self, idx: usize, val: RedisValue) {
        match idx {
            0 => self.key = Some(val),
            _ => {}
        }
    }
}

impl RedisCommand for SetCommand {
    fn run(&self) -> Vec<Task> {
        match (&self.key, &self.value) {
            (Some(RedisValue::BulkString(k)), Some(RedisValue::BulkString(v))) => {
                set_value(k.as_str(), v.as_str(), self.cache, self.ttl.clone());
                vec![Task::NetworkWrite(format!("+OK\r\n"))]
            }
            _ => vec![Task::NetworkError("-ERROR (invalid value)\r\n".to_string())],
        }
    }

    fn num_args(&self) -> usize {
        2
    }

    fn set_arg(&mut self, idx: usize, val: RedisValue) {
        match idx {
            0 => self.key = Some(val),
            1 => self.value = Some(val),
            2 => self.cache = true,
            3 => self.ttl = Some(val),
            _ => {
                panic!("SetCommand::set_arg: invalid index")
            }
        }
    }
}

pub enum Task {
    NetworkWrite(String),
    NetworkError(String),
}
