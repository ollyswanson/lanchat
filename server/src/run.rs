use protocol::message::LanChatMessage;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::{connection, server};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let (b_send, _) = broadcast::channel::<String>(10);
    let (tx, rx) = mpsc::channel::<LanChatMessage>(100);

    {
        let b_send = b_send.clone();
        tokio::spawn(async move { server::run_server(rx, b_send).await });
    }

    loop {
        let (socket, addr) = listener.accept().await?;

        let b_recv = b_send.subscribe();
        let tx = tx.clone();

        tokio::spawn(async move {
            connection::handle_connection(socket, addr, tx, b_recv).await;
        });
    }
}
