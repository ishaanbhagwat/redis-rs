use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

use crate::resp::parse_resp;
use crate::store::KvStore;

async fn read_resp_message(reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut first_line = String::new();
    let bytes_read = reader.read_line(&mut first_line).await?;
    if bytes_read == 0 {
        return Err("Connection closed".into());
    }

    if !first_line.starts_with('*') {
        return Err("Invalid RESP format: expected array".into());
    }

    let count: usize = first_line[1..].trim().parse()?;
    let mut message = first_line;

    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).await?;
        message.push_str(&len_line);

        let mut value_line = String::new();
        reader.read_line(&mut value_line).await?;
        message.push_str(&value_line);
    }

    Ok(message)
}

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
        return "Usage DEL <key>\n".to_string();
    }

    let mut store = store.lock().unwrap();
    let key = &parts[1];
    match store.del(key) {
        Some(_) => return format!("deleted {}\n", key),
        None => return "key not found\n".to_string()
    }
}

pub async fn handle_connection (
    stream: TcpStream,
    store: Arc<Mutex<KvStore>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        let message = match read_resp_message(&mut reader).await {
            Ok(msg) => msg,
            Err(e) => {
                if e.to_string().contains("Connection closed") {
                    println!("Connection closed by client");
                    break;
                }
                writer.write_all(b"Invalid RESP format\r\n").await?;
                continue;
            }
        };

        let parts = match parse_resp(&message) {
            Ok(p) => p,
            Err(_) => {
                writer.write_all(b"Invalid RESP format\r\n").await?;
                continue;
            }
        };

        if parts.is_empty() {
            writer.write_all(b"Empty command\r\n").await?;
            continue;
        }

        let response = match parts[0].to_lowercase().as_str() {
            "set" => handle_set(&store, &parts),
            "get" => handle_get(&store, &parts),
            "del" => handle_del(&store, &parts),
            _ => "Unknown Command\n".to_string()
        };

        writer.write_all(response.as_bytes()).await?;
    }
    Ok(())
}
