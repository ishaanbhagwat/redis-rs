mod connection;
mod store;
mod resp;

use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use store::KvStore;
use connection::handle_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let listner = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server running on 127.0.0.1:6379");

    let store = Arc::new(Mutex::new( KvStore::new()));

    loop{
        let (socket, _) =  listner.accept().await?;
        let store = Arc::clone(&store);

        tokio::spawn(async move{
            if let Err(e) = handle_connection(socket, store).await {
                eprintln!("Connection error: {:?}", e);
            }
        });
    }
}
