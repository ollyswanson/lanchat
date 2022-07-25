use tokio::sync::{broadcast, mpsc};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{connection, internal_message::InternalMessage, server};

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let (b_send, _) = broadcast::channel::<String>(8);
    let (tx, rx) = mpsc::channel::<InternalMessage>(128);

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
