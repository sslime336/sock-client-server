#[macro_use]
extern crate tracing;

use std::{
    io::{stdin, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Ok;
use tcp_config::Config;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    info!("hello from tcp-client");

    let addr = Config::get()
        .iter()
        .find(|node| node.tag == "master")
        .map(|addr| SocketAddr::new(addr.ip.parse().unwrap(), addr.port))
        .unwrap();

    let stream = Arc::new(TcpStream::connect(addr)?);

    // 持续读
    let rx_stream = Arc::clone(&stream);
    thread::spawn(move || {
        let mut buf = vec![0; 2048];
        let mut rx_stream = rx_stream.try_clone().unwrap();
        loop {
            match rx_stream.read(&mut buf) {
                std::result::Result::Ok(end) => {
                    let rev = buf[..end].to_owned();
                    if !rev.is_empty() {
                        info!("From server [{}]:{}", addr, String::from_utf8(rev).unwrap());
                    }
                }
                Err(err) => {
                    warn!("[READ_ERROR] Server {} disconnected: {}, you can enter `quit` to end current process.", addr, err);
                    return;
                }
            }
            sleep(Duration::from_millis(200));
        }
    });

    let stdin = stdin();
    let mut buf = String::with_capacity(2048);

    let mut tx_stream = stream.try_clone()?;
    loop {
        let end = stdin.read_line(&mut buf)?;
        let data = buf[..end].to_string();
        let content = data.trim_end();
        if content == "quit" {
            return Ok(());
        }
        if !data.is_empty() {
            if let Err(err) = tx_stream.write_all(content.as_bytes()) {
                error!("[WRITE_ERROR] Server {} disconnected: {}, you can enter `quit` to end current process.", addr, err);
            }
            buf.clear();
        }
    }
}
