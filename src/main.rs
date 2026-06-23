use cmd::tcplistener::listen_for_http;
use lb::{Response, Router};

mod cmd;
fn main() {
    println!("Hello, world!");

    let mut router = Router::new();

    router.get("/test", |_req| {
        Response::new(200, "Ok", Some("We are fxking live".as_bytes().to_vec()))
    }).unwrap();

    let _ = listen_for_http(&mut router);
}
