use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Recipient, WrapFuture};
use crate::connection_manager::Prune;
use crate::net::{Packet, ConnectionWriter, PacketManager};

pub struct PlayerConnection {
    recipient: Recipient<Prune>,
    conn_writer: ConnectionWriter,
    packet_manager: PacketManager
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IncomingRequest(pub Packet);

impl PlayerConnection {
    pub fn new(conn_writer: ConnectionWriter, recipient: Recipient<Prune>) -> Addr<Self> {
        Self::create(|ctx| {
            let result = Self {
                recipient,
                conn_writer,
                packet_manager: PacketManager::new()
            };
            result
        })
    }
}

impl Actor for PlayerConnection {
    type Context = Context<Self>;

    // fn started(&mut self, ctx: &mut Self::Context) {
    //     let frequency = Duration::from_secs(10);
    //     let this = ctx.address();
    //     let future = async move {
    //         let mut interval = interval_at(Instant::now() + frequency, frequency);
    //         loop {
    //             interval.tick().await;
    //             let _ = this.send(Stop).await;
    //         }
    // 
    //     };
    //     future.into_actor(self).spawn(ctx);
    // }
}

impl Handler<IncomingRequest> for PlayerConnection {
    type Result = ();

    fn handle(&mut self, IncomingRequest(Packet(header)): IncomingRequest, ctx: &mut Self::Context) -> Self::Result {
        println!("{}: {:?}", self.conn_writer.to_address(), header);
        if let Some(dropped_packets) = self.packet_manager.receive(&header) {
            println!("Uh oh, we dropped some packets: {:?}", dropped_packets);
        }
        let packet = Packet(self.packet_manager.send().unwrap());
        let future = self.conn_writer.send(packet);
        let future = async move { 
            let _ = future.await;
        };
        let future = future.into_actor(self);
        ctx.spawn(future);
    }
}
