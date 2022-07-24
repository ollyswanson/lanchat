use std::io;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Socket: {:?} Addr: {:?}", socket, addr.ip());

        tokio::spawn(async move {
            server::process_socket(socket, addr).await;
        });
    }
}
