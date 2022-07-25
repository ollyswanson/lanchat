use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use protocol::{codec::LanChatCodec, message::LanChatMessage};
use tokio::{
    net::TcpStream,
    sync::{broadcast::Receiver, mpsc::Sender, oneshot},
};
use tokio_util::codec::Framed;

use crate::internal_message::{InternalMessage, Response};

pub(crate) async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<InternalMessage>,
    mut msg_broadcast: Receiver<String>,
) {
    let (mut send_frame, mut recv_frame) =
        Framed::new(socket, LanChatCodec::with_max_length(4096)).split();

    loop {
        tokio::select!(
            msg = recv_frame.next() => {
                // TODO: Handle errors
                if let Some(Ok(msg)) = msg {
                    let (once_send, once_recv) = oneshot::channel();
                    let _ = tx.send(InternalMessage::new(addr, msg, once_send)).await;
                    if let Ok(response) = once_recv.await {
                        match response {
                            Response::Ack => {},
                            Response::HangUp => { break; }
                        }
                    }
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
