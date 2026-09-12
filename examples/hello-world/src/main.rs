// 定义一个异步函数。
async fn say_hello() {
    println!("hello, world!");
}

#[tokio::main] //让我们写 `async fn main` 的模板，稍后我们会进行解释。
async fn main() {
    // 调用异步函数并等待其结果。
    say_hello().await;
}
