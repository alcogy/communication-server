use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use uuid::Uuid;
use std::collections::{HashMap};
use actix_web::web::Bytes;
use actix_web::web::Data;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub Bytes, pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect { 
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendBinary {
    pub id: Uuid,
    pub bin: Bytes,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendText {
    pub id: Uuid,
    pub text: String,
}

struct Session {
    sessions: HashMap<Uuid, Recipient<Message>>
}

impl Session {
    pub fn new() -> Session {
        Session {
            sessions: HashMap::new(),
        }
    }

    fn send_binary(&self, binary: Bytes, me: Uuid) {
        for (key, value) in &self.sessions {
            if *key != me {
                let _ = value.do_send(Message(binary.clone(), "".to_string()));
            }
        }
    }

    fn send_text(&self, text: String, me: Uuid) {
        for (key, value) in &self.sessions {
            if *key != me {
                let _ = value.do_send(Message(Bytes::from(&b""[..]), text.to_owned()));
            }
        }
    }
}

impl Actor for Session {
    type Context = Context<Self>;
}

impl Handler<Connect> for Session {
    type Result = ();
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.id, msg.addr);
    }
}

impl Handler<Disconnect> for Session {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<SendBinary> for Session {
    type Result = ();
    fn handle(&mut self, msg: SendBinary, _: &mut Context<Self>) {
        self.send_binary(msg.bin, msg.id);
    }
}

impl Handler<SendText> for Session {
    type Result = ();
    fn handle(&mut self, msg: SendText, _: &mut Context<Self>) {
        self.send_text(msg.text, msg.id);
    }
}

struct Connection {
    id: Uuid,
    addr: Addr<Session>,
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

async fn index(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<Session>>) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        Connection {
            id: Uuid::new_v4(),
            addr: srv.get_ref().clone()
        },
        &req,
        stream
    );
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let session = Session::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(session.clone()))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await
}

