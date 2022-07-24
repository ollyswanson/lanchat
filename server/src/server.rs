use protocol::{command::Command, message::LanChatMessage};
use tokio::sync::{broadcast::Sender, mpsc::Receiver};

pub(crate) async fn run_server(mut recv: Receiver<LanChatMessage>, msg_broadcast: Sender<String>) {
    while let Some(msg) = recv.recv().await {
        match msg.command {
            Command::Message(_) => {
                // TODO: Improve error handling.
                let x = msg_broadcast.send("foo".to_owned());
            }
            Command::Nick(_) => {
                unreachable!("Currently handled by connection, might change!");
            }
        }
    }
}
