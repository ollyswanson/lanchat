use std::{collections::HashMap, net::SocketAddr};

use protocol::{
    command::Command,
    message::{LanChatMessage, Prefix},
};
use tokio::sync::{broadcast::Sender, mpsc::Receiver, oneshot};

use crate::response::Response;

pub(crate) async fn run_server(
    mut recv: Receiver<(SocketAddr, LanChatMessage, oneshot::Sender<Response>)>,
    msg_broadcast: Sender<String>,
) {
    let mut prefixes: HashMap<SocketAddr, Prefix> = HashMap::new();

    while let Some((addr, mut msg, respond)) = recv.recv().await {
        let prefix = prefixes.get(&addr).cloned().unwrap_or_else(|| Prefix {
            nick: "unknown".to_string(),
        });

        match msg.command {
            Command::Msg(_) => {
                msg.prefix = Some(prefix);
                let _ = msg_broadcast.send(msg.to_string());
            }
            Command::Nick(nick) => {
                prefixes.insert(addr, Prefix { nick });
            }
            Command::Quit => {
                prefixes.remove(&addr);
                let _ = respond.send(Response::HangUp);
            }
        }
    }
}
