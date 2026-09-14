use tokio::{spawn, time::{sleep, Duration}};

async fn say_hello() {
    // 等待一会儿再打印，使竞争更有趣。
    sleep(Duration::from_millis(100)).await;
    println!("hello");
}

async fn say_world() {
    sleep(Duration::from_millis(100)).await;
    println!("world!");
}

#[tokio::main]
async fn main() {
    spawn(say_hello());
    spawn(say_world());
    // 等待一会儿，给任务一些运行时间。
    sleep(Duration::from_millis(1000)).await;
}
