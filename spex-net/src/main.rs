use tokio::time::{sleep, Duration};

async fn task_a() {
    for i in 1..=3 {
        println!("A: Sleep {i}");
        sleep(Duration::from_secs(1)).await;
    }
}

async fn task_b() {
    for i in  1..3{
        println!("A: sleep {i}");
        sleep(Duration::from_secs(1)).await;
    }
}
