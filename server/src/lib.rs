use std::net::SocketAddr;

use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

pub mod database;

pub async fn process_socket(socket: TcpStream, addr: SocketAddr) {
    let mut framed = Framed::new(socket, LinesCodec::new_with_max_length(4096));

    while let Some(frame) = framed.next().await {
        if let Ok(msg) = frame {
            println!("{}", msg);
        }
    }
}
