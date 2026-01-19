use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        loop {
            println!("tick");
            sleep(Duration::from_secs(1)).await;
        }
    });

    let msg = spex_net::async_wait_and_hello_world().await;
    println!("{msg}");
    println!("done");
}
