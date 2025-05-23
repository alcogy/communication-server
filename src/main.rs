use actix::prelude::*;
use actix::{Actor};
use actix_web::web::Data;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

mod message;
mod session;
use session::Session;
mod connection;
use connection::Connection;

async fn index(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<Session>>) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        Connection::new(srv.get_ref().clone()),
        &req,
        stream
    );
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let session = Session::new().start();
    let port = 9999;
    println!("Server is running! at {}", port);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(session.clone()))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await
}

