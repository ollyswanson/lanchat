//! Internal messages.
//!
//! This module defines the different types used for passing messages from a client connection to
//! the main actor orchestrating the server.
use std::net::SocketAddr;

use protocol::message::LanChatMessage;
use tokio::sync::oneshot::Sender;

/// A type for sending messages from a connection to the main actor.
#[derive(Debug)]
pub struct InternalMessage {
    /// The address of the connected client.
    pub addr: SocketAddr,
    /// The message sent from the client to the server.
    pub msg: LanChatMessage,
    /// The sending half of a oneshot channel used to send a `Response` from the server actor back
    /// to the client once `msg` has been processed.
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

/// A response that the main actor sends back to the task handling a connection.
#[derive(Debug)]
pub enum Response {
    /// A straightforward acknowledgment that the command has been processed.
    Ack,
    /// A command telling the connection task to hang up, is issues after the client has sent a
    /// QUIT command.
    HangUp,
}
