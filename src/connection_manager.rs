use std::collections::HashMap;
use std::net::SocketAddr;
use std::os::unix::raw::mode_t;
use std::sync::Arc;
use actix::{Actor, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, WrapFuture};
use actix_rt::net::UdpSocket;
use bytes::Bytes;
use crate::packet_header::ToPacketHeaderError;
use crate::player_connection::{Packet, PlayerConnection};

pub struct ConnectionManager {
    connections: HashMap<SocketAddr, Addr<PlayerConnection>>
}

impl ConnectionManager {
    pub fn new() -> Addr<Self> {
        Self { connections: HashMap::new() }.start()
    }
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), ToPacketHeaderError>")]
pub struct NewConnection(pub SocketAddr, pub Bytes, pub Arc<UdpSocket>);

impl Handler<NewConnection> for ConnectionManager {
    type Result = Result<(), ToPacketHeaderError>;

    fn handle(&mut self, NewConnection(addr, bytes, socket): NewConnection, ctx: &mut Self::Context) -> Self::Result {
        let connection = self.connections.entry(addr)
            .or_insert_with(|| PlayerConnection::new(addr, ctx.address().recipient(), socket));
        
        let packet = Packet(bytes.try_into()?);
        let connection = connection.clone();
        let future = async move { 
            let _ = connection.send(packet).await;
        };
        future.into_actor(self).spawn(ctx);
        
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Prune(pub SocketAddr);

impl Handler<Prune> for ConnectionManager {
    type Result = ();

    fn handle(&mut self, Prune(addr): Prune, _: &mut Self::Context) -> Self::Result {
        println!("Pruning {addr}");
        self.connections.remove(&addr);
    }
}