use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(10);
    
    tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(format!("message {i}")).await.unwrap();
            sleep(Duration::from_secs(1)).await;
        }
    });
    
    while let Some(msg) = rx.recv().await {
        println!("recived: {msg}");
    }
    
    println!("channel closed")
}