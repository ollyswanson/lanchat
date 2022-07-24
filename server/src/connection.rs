use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use protocol::{
    codec::LanChatCodec,
    command::Command,
    message::{LanChatMessage, Prefix},
};
use tokio::{
    net::TcpStream,
    sync::{broadcast::Receiver, mpsc::Sender},
};
use tokio_util::codec::Framed;

pub(crate) async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<LanChatMessage>,
    mut msg_broadcast: Receiver<String>,
) {
    let (mut send_frame, mut recv_frame) =
        Framed::new(socket, LanChatCodec::with_max_length(4096)).split();

    // First message from the client must be the NICK command
    let nickname = if let Some(Ok(LanChatMessage {
        command: Command::Nick(nickname),
        ..
    })) = recv_frame.next().await
    {
        nickname
    } else {
        return;
    };

    let prefix = Prefix { nick: nickname };

    loop {
        tokio::select!(
            msg = recv_frame.next() => {
                // TODO: Handle errors
                if let Some(Ok(msg)) = msg {
                    let _ = tx.send(msg).await;
                }
            }
            msg = msg_broadcast.recv() => {
                if let Ok(msg) = msg {
                    let _ = send_frame.send(msg).await;
                }
            }
        )
    }
}
