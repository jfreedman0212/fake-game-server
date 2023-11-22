use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use actix_rt::net::UdpSocket;
use bytes::BytesMut;
use crate::net::{Packet};

#[derive(Clone)]
pub struct ConnectionWriter {
    to_address: SocketAddr,
    socket: Arc<UdpSocket>,
}

impl ConnectionWriter {
    fn new(to_address: SocketAddr, socket: Arc<UdpSocket>) -> Self {
        Self {
            to_address,
            socket
        }
    }
    
    pub async fn send(&self, packet: Packet) -> io::Result<()> {
        let mut buffer = BytesMut::with_capacity(256);
        packet.serialize(&mut buffer);
        self.socket.send_to(&buffer, self.to_address).await?;
        
        Ok(())
    }
    
    pub fn to_address(&self) -> SocketAddr {
        self.to_address
    }
}

pub struct ConnectionReader {
    socket: Arc<UdpSocket>,
    buffer: BytesMut
}

impl ConnectionReader {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket: Arc::new(socket),
            buffer: BytesMut::with_capacity(256)
        }
    }
    
    pub async fn receive(&mut self) -> io::Result<Option<(Packet, ConnectionWriter)>> {
        let (_, from_address) = self.socket.recv_buf_from(&mut self.buffer).await?;
        
        let result = match Packet::try_deserialize(&mut self.buffer) {
            Ok(header) => {
                let conn_writer = ConnectionWriter::new(from_address, self.socket.clone()); 
                Some((header, conn_writer))
            }
            Err(err) => {
                println!("{from_address}: Received an error {err:?}");
                None
            }
        };
        
        Ok(result)
    }
}