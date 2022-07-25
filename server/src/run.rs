use std::net::SocketAddr;

use protocol::message::LanChatMessage;
use tokio::sync::{broadcast, mpsc};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{connection, response::Response, server};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let (b_send, _) = broadcast::channel::<String>(8);
    let (tx, rx) = mpsc::channel::<(SocketAddr, LanChatMessage, oneshot::Sender<Response>)>(128);

    let server_bcast = b_send.clone();
    tokio::spawn(async move { server::run_server(rx, server_bcast).await });

    loop {
        let (socket, addr) = listener.accept().await?;

        let b_recv = b_send.subscribe();
        let tx = tx.clone();

        tokio::spawn(async move {
            connection::handle_connection(socket, addr, tx, b_recv).await;
        });
    }
}
