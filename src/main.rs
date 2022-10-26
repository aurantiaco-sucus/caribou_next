mod caribou;

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    test().await
}

async fn test() {
    sleep(Duration::from_secs(1)).await;
}