#![cfg(test)]
#![recursion_limit="128"]

mod example {
// ANCHOR: example
use futures::{
    future::FutureExt, // 用于 `.fuse()`
    pin_mut,
    select,
};

async fn task_one() { /* ... */ }
async fn task_two() { /* ... */ }

async fn race_tasks() {
    let t1 = task_one().fuse();
    let t2 = task_two().fuse();

    pin_mut!(t1, t2);

    select! {
        () = t1 => println!("task one completed first"),
        () = t2 => println!("task two completed first"),
    }
}
// ANCHOR_END: example
}

mod default_and_complete {
// ANCHOR: default_and_complete
use futures::{future, select};

async fn count() {
    let mut a_fut = future::ready(4);
    let mut b_fut = future::ready(6);
    let mut total = 0;

    loop {
        select! {
            a = a_fut => total += a,
            b = b_fut => total += b,
            complete => break,
            default => unreachable!(), // 永远不会运行（两个 future 都已就绪，随后即完成）
        };
    }
    assert_eq!(total, 10);
}
// ANCHOR_END: default_and_complete

#[test]
fn run_count() {
    futures::executor::block_on(count());
}
}

mod fused_stream {
// ANCHOR: fused_stream
use futures::{
    stream::{Stream, StreamExt, FusedStream},
    select,
};

async fn add_two_streams(
    mut s1: impl Stream<Item = u8> + FusedStream + Unpin,
    mut s2: impl Stream<Item = u8> + FusedStream + Unpin,
) -> u8 {
    let mut total = 0;

    loop {
        let item = select! {
            x = s1.next() => x,
            x = s2.next() => x,
            complete => break,
        };
        if let Some(next_num) = item {
            total += next_num;
        }
    }

    total
}
// ANCHOR_END: fused_stream
}

mod fuse_terminated {
// ANCHOR: fuse_terminated
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    stream::{FusedStream, Stream, StreamExt},
    pin_mut,
    select,
};

async fn get_new_num() -> u8 { /* ... */ 5 }

async fn run_on_new_num(_: u8) { /* ... */ }

async fn run_loop(
    mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
    starting_num: u8,
) {
    let run_on_new_num_fut = run_on_new_num(starting_num).fuse();
    let get_new_num_fut = Fuse::terminated();
    pin_mut!(run_on_new_num_fut, get_new_num_fut);
    loop {
        select! {
            () = interval_timer.select_next_some() => {
                // 定时器已到时。如果尚未运行 `get_new_num_fut`，
                // 则启动一个新的。
                if get_new_num_fut.is_terminated() {
                    get_new_num_fut.set(get_new_num().fuse());
                }
            },
            new_num = get_new_num_fut => {
                // 一个新数字已到达 —— 启动一个新的 `run_on_new_num_fut`，
                // 并丢弃旧的。
                run_on_new_num_fut.set(run_on_new_num(new_num).fuse());
            },
            // 运行 `run_on_new_num_fut`
            () = run_on_new_num_fut => {},
            // 如果全部完成则 panic，因为 `interval_timer` 应该
            // 会无限期地持续产生值。
            complete => panic!("`interval_timer` completed unexpectedly"),
        }
    }
}
// ANCHOR_END: fuse_terminated
}

mod futures_unordered {
// ANCHOR: futures_unordered
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
    pin_mut,
    select,
};

async fn get_new_num() -> u8 { /* ... */ 5 }

async fn run_on_new_num(_: u8) -> u8 { /* ... */ 5 }

async fn run_loop(
    mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
    starting_num: u8,
) {
    let mut run_on_new_num_futs = FuturesUnordered::new();
    run_on_new_num_futs.push(run_on_new_num(starting_num));
    let get_new_num_fut = Fuse::terminated();
    pin_mut!(get_new_num_fut);
    loop {
        select! {
            () = interval_timer.select_next_some() => {
                // 定时器已到时。如果尚未运行 `get_new_num_fut`，
                // 则启动一个新的。
                if get_new_num_fut.is_terminated() {
                    get_new_num_fut.set(get_new_num().fuse());
                }
            },
            new_num = get_new_num_fut => {
                // 一个新数字已到达 —— 启动一个新的 `run_on_new_num_fut`。
                run_on_new_num_futs.push(run_on_new_num(new_num));
            },
            // 运行 `run_on_new_num_futs` 并检查是否有已完成的
            res = run_on_new_num_futs.select_next_some() => {
                println!("run_on_new_num_fut returned {:?}", res);
            },
            // 如果全部完成则 panic，因为 `interval_timer` 应该
            // 会无限期地持续产生值。
            complete => panic!("`interval_timer` completed unexpectedly"),
        }
    }
}

// ANCHOR_END: futures_unordered
}
