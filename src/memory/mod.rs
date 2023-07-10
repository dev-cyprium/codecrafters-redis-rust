use std::{
    collections::HashMap,
    sync::{Mutex, Once},
    time,
};

use crate::RedisValue;

static ONCE: Once = Once::new();

static mut MUTEX: *const Mutex<HashMap<String, TimedCachedValue>> = 0 as *const _;

struct TimedCachedValue {
    value: String,
    expires: Option<u128>,
}

fn get_mutex() -> &'static Mutex<HashMap<String, TimedCachedValue>> {
    unsafe {
        ONCE.call_once(|| {
            MUTEX = Box::into_raw(Box::new(Mutex::new(HashMap::new())));
        });
        &*MUTEX
    }
}

fn unix_timestamp() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn set_value(key: &str, value: &str, cache: bool, _ttl: Option<RedisValue>) {
    let mutex = get_mutex();
    let mut map = mutex.lock().unwrap();

    let mut timed_value = TimedCachedValue {
        value: value.to_string(),
        expires: None,
    };

    if cache {
        if let Some(ttl) = _ttl {
            if let RedisValue::BulkString(ttl) = ttl {
                let ttl = ttl.parse::<u128>().unwrap();
                let ttl = unix_timestamp() + ttl;
                timed_value.expires = Some(ttl);
            }
        }
    }

    map.insert(key.to_string(), timed_value);
}

pub fn get_value(key: &str) -> Option<String> {
    let mutex = get_mutex();
    let map = mutex.lock().unwrap();

    if let Some(timed_value) = map.get(key) {
        if let Some(expires) = timed_value.expires {
            if expires > unix_timestamp() {
                return Some(timed_value.value.clone());
            } else {
                return None;
            }
        } else {
            return Some(timed_value.value.clone());
        }
    }
    None
}
