use cmd::tcplistener::listen_for_http;

mod cmd;
fn main() {
    println!("Hello, world!");

    let _ = listen_for_http();
}
