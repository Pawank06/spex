use tokio::time::{sleep, Duration};

pub mod protocol;

pub async fn async_hello() -> String {
    "hello from async".to_string()
}

pub async fn async_wait_and_hello_world() -> String {
    println!("waiting...");
    sleep(Duration::from_secs(2)).await;
    "hello world after waiting".to_string()
}