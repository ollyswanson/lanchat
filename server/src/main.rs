#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Server starting");
    server::run().await
}
