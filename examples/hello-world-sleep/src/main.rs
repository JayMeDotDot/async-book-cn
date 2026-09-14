use std::io::{stdout, Write};
use tokio::time::{sleep, Duration};

async fn say_hello() {
    print!("hello, ");
    // 刷新 stdout，这样我们就能立即看到上面 `print` 的效果。
    stdout().flush().unwrap();
}

async fn say_world() {
    println!("world!");
}

#[tokio::main]
async fn main() {
    say_hello().await;
    // 一个异步休眠函数, 让当前任务休眠 1s.
    sleep(Duration::from_millis(1000)).await;
    say_world().await;
}
