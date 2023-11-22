use std::collections::HashMap;
use std::net::SocketAddr;
use actix::{Actor, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, WrapFuture};
use crate::net::{Packet, ConnectionWriter};
use crate::player_connection::{IncomingRequest, PlayerConnection};

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
#[rtype(result = "()")]
pub struct NewConnection(pub Packet, pub ConnectionWriter);

impl Handler<NewConnection> for ConnectionManager {
    type Result = ();

    fn handle(&mut self, NewConnection(packet, conn_writer): NewConnection, ctx: &mut Self::Context) -> Self::Result {
        let connection = self.connections.entry(conn_writer.to_address())
            .or_insert_with(|| PlayerConnection::new(conn_writer, ctx.address().recipient()));
        let future = connection.send(IncomingRequest(packet));
        let future = async move { 
            let _ = future.await;
        };
        future.into_actor(self).spawn(ctx);
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