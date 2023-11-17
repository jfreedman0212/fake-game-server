use std::sync::Arc;
use actix_rt::net::UdpSocket;
use bytes::{Bytes};
use crate::connection_manager::{ConnectionManager, NewConnection};

pub struct Server {
    socket: Arc<UdpSocket>
}

impl Server {
    pub fn new(socket: UdpSocket) -> Self {
        Self { 
            socket: Arc::new(socket)
        }
    }
    
    pub async fn run(&self) {
        let mut buf = [0u8; 256];
        let conn_manager = ConnectionManager::new();

        while let Ok((_, addr)) = self.socket.recv_from(&mut buf).await {
            let conn_manager = conn_manager.clone();
            let socket = self.socket.clone();
            actix_rt::spawn(async move {
                let new_conn = NewConnection(addr, Bytes::copy_from_slice(&buf), socket);
                if let Err(e) = conn_manager.send(new_conn).await {
                    eprintln!("{e}");
                }
            });
        }
    }
}

#[cfg(test)]
mod test {
    use crate::packet_header::{PacketHeader, PROTOCOL_ID};
    use super::*;
    
    #[actix_rt::test]
    pub async fn hello_world() {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let addr = socket.local_addr().unwrap();
        let server = Server::new(socket);
        let handle = actix_rt::spawn(async move {
            server.run().await;
        });

        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        socket.connect(addr).await.unwrap();
        let initial_header = PacketHeader {
            protocol_id: PROTOCOL_ID,
            sequence: 100,
            ack: 200,
            ack_bitfield: 0
        };
        let bytes: Bytes = initial_header.into();

        socket.send(&bytes).await.unwrap();
        
        let mut bytes = [0u8; 256];
        
        socket.recv(&mut bytes).await.unwrap();
        
        let header: PacketHeader = Bytes::copy_from_slice(&bytes).try_into().unwrap();
        
        assert_eq!(header.sequence, 101);
        assert_eq!(header.ack, 201);

        handle.abort();
    }
}