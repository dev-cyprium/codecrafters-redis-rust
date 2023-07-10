use std::{
    collections::HashMap,
    sync::{Mutex, Once},
};

static ONCE: Once = Once::new();

static mut MUTEX: *const Mutex<HashMap<String, String>> = 0 as *const _;

fn get_mutex() -> &'static Mutex<HashMap<String, String>> {
    unsafe {
        ONCE.call_once(|| {
            MUTEX = Box::into_raw(Box::new(Mutex::new(HashMap::new())));
        });
        &*MUTEX
    }
}

pub fn set_value(key: &str, value: &str) {
    let mutex = get_mutex();
    let mut map = mutex.lock().unwrap();
    map.insert(key.to_string(), value.to_string());
}

pub fn get_value(key: &str) -> Option<String> {
    let mutex = get_mutex();
    let map = mutex.lock().unwrap();
    map.get(key).cloned()
}
