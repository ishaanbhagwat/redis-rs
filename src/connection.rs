use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

use crate::resp::parse_resp;
use crate::store::KvStore;

fn handle_set(store: &Arc<Mutex<KvStore>>, parts: &Vec<String>) -> String {
    if parts.len() != 3 {
        return "Usage SET <key> <value>".to_string();
    }

    let mut store = store.lock().unwrap();
    let key = parts[1].to_string();
    let value = parts[2].to_string();
    store.set(key, value);
    return "OK\n".to_string();
}

fn handle_get(store: &Arc<Mutex<KvStore>>, parts: &Vec<String>) -> String {
    if parts.len() != 2 {
        return "Usage GET <key>\n".to_string();
    }

    let mut store = store.lock().unwrap();
    let key = parts[1].to_string();
    match store.get(&key) {
        Some(v) => return format!("{}\n", v),
        None => return "nil\n".to_string()
    }
}

fn handle_del(store: &Arc<Mutex<KvStore>>, parts: &Vec<String>) -> String {
    if parts.len() != 2 {
        return "Usage DEL <key>/n".to_string();
    }

    let mut store = store.lock().unwrap();
    let key = &parts[1];
    match store.del(key) {
        Some(_) => return format! ("deleted {}\n", key),
        None => return "key not found\n".to_string()
    }
}

pub async fn handle_connection (
    stream: TcpStream,
    store: Arc<Mutex<KvStore>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read <= 0 {
            "Connection Terminated\n".to_string();
        }

        let parts= match parse_resp(&line) {
            Ok(p) => p,
            Err(_) => {
                writer.write_all(b"Invalid RESP forma \r\n").await?;
                continue;
            }
        };

        let response = match parts[0].to_lowercase().as_str() {
            "set" => handle_set(&store, &parts),
            "get" => handle_get(&store, &parts),
            "del" => handle_del(&store, &parts),
            _ => "Unknown Command\n".to_string()
        };

        writer.write_all(response.as_bytes()).await?;
    }
}
