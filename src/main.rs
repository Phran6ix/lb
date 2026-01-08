use cmd::tcplistener;

mod cmd;
mod internal;
fn main() {
    println!("Hello, world!");

    tcplistener::listen_for_http().expect("An error occured in TCP listener");
}
