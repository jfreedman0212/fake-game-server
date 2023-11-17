use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, Recipient, SpawnHandle, WrapFuture};
use actix_rt::net::UdpSocket;
use bytes::Bytes;
use crate::connection_manager::Prune;
use crate::packet_header::PacketHeader;

pub struct PlayerConnection {
    addr: SocketAddr,
    recipient: Recipient<Prune>,
    socket: Arc<UdpSocket>,
    stop_task: SpawnHandle
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Packet(pub PacketHeader);

impl PlayerConnection {
    pub fn new(addr: SocketAddr, recipient: Recipient<Prune>, socket: Arc<UdpSocket>) -> Addr<Self> {
        Self::create(|ctx| {
            Self {
                addr,
                recipient,
                socket,
                stop_task: Self::spawn_stop_task(ctx)
            }
        })
    }
    
    fn spawn_stop_task(ctx: &mut Context<Self>) -> SpawnHandle {
        ctx.run_later(Duration::from_secs(10), |this, ctx| {
            ctx.stop();
            this.recipient.do_send(Prune(this.addr));
        })
    }
}

impl Actor for PlayerConnection {
    type Context = Context<Self>;
}

impl Handler<Packet> for PlayerConnection {
    type Result = ();

    fn handle(&mut self, Packet(mut header): Packet, ctx: &mut Self::Context) -> Self::Result {
        // cancel the stop task since a request came in
        ctx.cancel_future(self.stop_task);
        println!("{}: {:?}", self.addr, header);
        let socket = self.socket.clone();
        let to_addr = self.addr.clone();
        let future = async move {
            header.sequence += 1;
            header.ack += 1;
            let data: Bytes = header.into();
            let _ = socket.send_to(&data, to_addr).await;
        };
        future.into_actor(self).spawn(ctx);
        // then, restart the stop task so we time out if no requests come in again
        self.stop_task = Self::spawn_stop_task(ctx);
    }
}