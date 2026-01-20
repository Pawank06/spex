use tokio::time::{sleep, Duration};

async fn task_a() {
    for i in 1..=3 {
        println!("A: Sleep {i}");
        sleep(Duration::from_secs(1)).await;
    }
}
