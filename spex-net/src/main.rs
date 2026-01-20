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

#[tokio::main]
async fn main() {
    let a = tokio::spawn(task_a());
    let b = tokio::spawn(task_b());
    
    let _ = tokio::join!(a, b);
    
    println!("Both tasks completed");
}