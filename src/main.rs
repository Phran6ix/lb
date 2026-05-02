use cmd::tcplistener::listen_for_http;

mod cmd;
mod internal;
mod framework;
fn main() {
    println!("Hello, world!");

    let _ = listen_for_http();
}
