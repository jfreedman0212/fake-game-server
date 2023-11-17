mod player_connection;
mod connection_manager;
mod server;
mod packet_header;

use actix_rt::net::UdpSocket;
use crate::server::Server;

#[actix_rt::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    let server = Server::new(socket);
    server.run().await;
}
