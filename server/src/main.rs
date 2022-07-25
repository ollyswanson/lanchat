#[tokio::main]
async fn main() -> Result<(), server::BoxedError> {
    server::run().await
}
