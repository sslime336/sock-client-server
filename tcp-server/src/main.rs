#[macro_use]
extern crate tracing;

use std::{
    collections::HashMap,
    io::{stdin, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use once_cell::sync::Lazy;
use rand::Rng;
use tcp_config::Config;

static CLIENT_MAP: Lazy<Mutex<HashMap<String, Arc<(TcpStream, SocketAddr)>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    info!("tcp-server starting");

    let addr = Config::get()
        .iter()
        .find(|node| node.tag == "master")
        .map(|addr| SocketAddr::new(addr.ip.parse().unwrap(), addr.port))
        .expect("need node with `master` tag");

    let listener = TcpListener::bind(addr)?;

    // 接受 TCP 连接，为每个连接分配一个新线程读取传来的数据
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            let (stream, incoming_addr) = listener.accept().unwrap();

            let stream_pair = Arc::new((stream, incoming_addr));

            let mut client_map = CLIENT_MAP.lock().unwrap();

            let random_id = ((rng.gen::<f64>() * 10000.0) as usize).to_string();
            client_map.insert(random_id.clone(), Arc::clone(&stream_pair));

            info!("Incoming address: {incoming_addr:?} with id: {}", random_id);

            thread::spawn(move || {
                let mut stream = stream_pair.0.try_clone().unwrap();
                let addr = stream_pair.1;
                let mut rng = rand::thread_rng();
                let mut buf = vec![0; 2048];
                let client_id = random_id.clone();

                loop {
                    match stream.read(&mut buf) {
                        Ok(end) => {
                            let rev = String::from_utf8(buf[..end].to_owned()).unwrap();
                            if !buf.is_empty() {
                                info!("From {}(ip: {:?}): {}", client_id, addr, rev);
                            }
                        }
                        Err(err) => {
                            warn!("[READ_ERROR] Client {} disconnected: {}", client_id, err);
                            return;
                        }
                    }

                    // 随机休眠一定时间，避免多线程阻塞峰值
                    let sleep_millis: f64 = rng.gen();
                    sleep(Duration::from_millis((sleep_millis * 20.0) as u64));
                }
            });

            sleep(Duration::from_millis(300));
        }
    });

    // 向指定的 Client 写数据
    let mut buf = String::with_capacity(512);
    let stdin = stdin();
    loop {
        let end = stdin.read_line(&mut buf)?;
        buf = buf[..end].to_string();
        if !buf.is_empty() {
            if let Some(idx) = buf.find('#') {
                let client_id = buf[..idx].to_string();
                let data = buf[idx + 1..].to_string();

                let mut client_map = CLIENT_MAP.lock().unwrap();
                if let Some(stream_pair) = client_map.get_mut(&client_id) {
                    let mut stream = stream_pair.0.try_clone().unwrap();

                    if let Err(err) = stream.write_all(data.trim_end().as_bytes()) {
                        error!("[WRITE_ERROR] Client {} disconnected: {}", client_id, err);
                        client_map.remove(&client_id).unwrap();
                    }
                } else {
                    error!("[WRITE_ERROR] Client {} doesn't exist", client_id);
                }
            }
            buf.clear();
        }
    }
}
