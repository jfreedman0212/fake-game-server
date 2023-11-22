use actix_rt::net::UdpSocket;
use crate::connection_manager::{ConnectionManager, NewConnection};
use crate::net::ConnectionReader;

pub struct Server {
    conn_reader: ConnectionReader
}

impl Server {
    pub fn new(socket: UdpSocket) -> Self {
        Self { 
            conn_reader: ConnectionReader::new(socket)
        }
    }
    
    pub async fn run(mut self) {
        let conn_manager = ConnectionManager::new();

        while let Ok(result) = self.conn_reader.receive().await {
            if let Some((packet, conn_writer)) = result {
                let conn_manager = conn_manager.clone();
                actix_rt::spawn(async move {
                    let new_conn = NewConnection(packet, conn_writer);
                    if let Err(e) = conn_manager.send(new_conn).await {
                        eprintln!("{e}");
                    }
                });   
            }
        }
    }
}
