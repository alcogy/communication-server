use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use uuid::Uuid;
use crate::message::{Message, Connect, Disconnect, SendBinary, SendText};
use crate::session::Session;

pub struct Connection {
    id: Uuid,
    addr: Addr<Session>,
}

impl Connection {
    pub fn new(addr: Addr<Session>) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            addr: addr
        }
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr.send(Connect {
            id: self.id,
            addr: addr.recipient(),
        })
        .into_actor(self)
        .then(|res, _act, ctx| {
            match res {
                Ok(_res) => (),
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {id: self.id});
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Connection {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                self.addr.do_send(SendText {
                    id: self.id,
                    text: text.to_string(),
                })
            },
            Ok(ws::Message::Binary(bin)) => {
                self.addr.do_send(SendBinary {
                    id: self.id,
                    bin,
                })
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<Message> for Connection {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        if msg.1 == "" {
            ctx.binary(msg.0);
        } else {
            ctx.text(msg.1);
        }
    }
}
