#![cfg(test)]

mod stream_trait {
use futures::stream::Stream as RealStream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

// ANCHOR: stream_trait
trait Stream {
    /// 流所产生的值的类型。
    type Item;

    /// 尝试解析流中的下一个条目。
    /// 若尚未就绪则返回 `Poll::Pending`，若有值就绪则返回 `Poll::Ready(Some(x))`，
    /// 若流已结束则返回 `Poll::Ready(None)`。
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>>;
}
// ANCHOR_END: stream_trait

// 断言 `Stream` 与 `RealStream` 相匹配：
impl<I> Stream for dyn RealStream<Item = I> {
    type Item = I;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>>
    {
        RealStream::poll_next(self, cx)
    }
}
}

mod channels {
use futures::{
    channel::mpsc,
    prelude::*,
};

// ANCHOR: channels
async fn send_recv() {
    const BUFFER_SIZE: usize = 10;
    let (mut tx, mut rx) = mpsc::channel::<i32>(BUFFER_SIZE);

    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();
    drop(tx);

    // `StreamExt::next` 与 `Iterator::next` 类似，但返回的是一个
    // 实现了 `Future<Output = Option<T>>` 的类型。
    assert_eq!(Some(1), rx.next().await);
    assert_eq!(Some(2), rx.next().await);
    assert_eq!(None, rx.next().await);
}
// ANCHOR_END: channels

#[test]
fn run_send_recv() { futures::executor::block_on(send_recv()) }
}
