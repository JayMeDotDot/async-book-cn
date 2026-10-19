#![cfg(test)]

use futures::executor::block_on;

mod first {
// ANCHOR: hello_world
// `block_on` 会阻塞当前线程，直到提供的 future 运行完毕。其他执行器提供更复杂的行为，
// 例如将多个 future 调度到同一个线程上。
use futures::executor::block_on;

async fn hello_world() {
    println!("hello, world!");
}

fn main() {
    let future = hello_world(); // 没有任何输出
    block_on(future); // `future` 被运行，并打印出 "hello, world!"
}
// ANCHOR_END: hello_world

#[test]
fn run_main() { main() }
}

struct Song;
async fn learn_song() -> Song { Song }
async fn sing_song(_: Song) {}
async fn dance() {}

mod second {
use super::*;
// ANCHOR: block_on_each
fn main() {
    let song = block_on(learn_song());
    block_on(sing_song(song));
    block_on(dance());
}
// ANCHOR_END: block_on_each

#[test]
fn run_main() { main() }
}

mod third {
use super::*;
// ANCHOR: block_on_main
async fn learn_and_sing() {
    // 等到学会这首歌之后再开始唱。
    // 这里使用 `.await` 而不是 `block_on`，以避免阻塞线程，
    // 这样才有可能同时 `dance`。
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!` 类似于 `.await`，但可以同时等待多个 future。
    // 如果 `learn_and_sing` 这个 future 暂时被阻塞，`dance`
    // 这个 future 就会接管当前线程。如果 `dance` 被阻塞了，
    // `learn_and_sing` 也可以重新接管回来。如果两个 future
    // 都被阻塞，那么 `async_main` 就会被阻塞，并将控制权让给执行器。
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}
// ANCHOR_END: block_on_main

#[test]
fn run_main() { main() }
}
