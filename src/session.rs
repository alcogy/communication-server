use actix::prelude::*;
use uuid::Uuid;
use std::collections::{HashMap};
use actix_web::web::Bytes;

use crate::message::{Message, Connect, Disconnect, SendBinary, SendText};

pub struct Session {
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