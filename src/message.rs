use actix::prelude::*;
use uuid::Uuid;
use actix_web::web::Bytes;

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
