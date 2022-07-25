use std::net::SocketAddr;

use protocol::message::LanChatMessage;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct InternalMessage {
    pub addr: SocketAddr,
    pub msg: LanChatMessage,
    pub respond: Sender<Response>,
}

impl InternalMessage {
    pub fn new(
        addr: SocketAddr,
        msg: LanChatMessage,
        respond: Sender<Response>,
    ) -> InternalMessage {
        InternalMessage { addr, msg, respond }
    }
}

#[derive(Debug)]
pub enum Response {
    Ack,
    HangUp,
}
