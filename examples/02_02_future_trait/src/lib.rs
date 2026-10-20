// ANCHOR: simple_future
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
// ANCHOR_END: simple_future

struct Socket;
impl Socket {
    fn has_data_to_read(&self) -> bool {
        // 检查该 socket 当前是否可读
        true
    }
    fn read_buf(&self) -> Vec<u8> {
        // 从 socket 中读入数据
        vec![]
    }
    fn set_readable_callback(&self, _wake: fn()) {
        // 将 `_wake` 注册到某个会在 socket
        // 变为可读时调用它的机制上，例如基于
        // `epoll` 的事件循环。
    }
}

// ANCHOR: socket_read
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // socket 中有数据了 —— 读入缓冲区并返回。
            Poll::Ready(self.socket.read_buf())
        } else {
            // socket 中还没有数据。
            //
            // 安排好让 `wake` 在数据可用时被调用。
            // 当数据可用时，`wake` 会被调用，而这个
            // `Future` 的使用者就知道要再次调用 `poll`
            // 并接收数据了。
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}
// ANCHOR_END: socket_read

// ANCHOR: join
/// 一个并发地运行另外两个 future 直至完成的 SimpleFuture。
///
/// 并发是这样实现的：对各个 future 的 `poll` 调用可以交错进行，
/// 从而让每个 future 都能按自己的节奏推进。
pub struct Join<FutureA, FutureB> {
    // 每个字段都可能包含一个应当运行至完成的 future。
    // 如果 future 已经完成，该字段会被设为 `None`。
    // 这可以避免我们在 future 完成之后再去轮询它——那
    // 会违反 `Future` trait 的契约。
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // 尝试完成 future `a`。
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // 尝试完成 future `b`。
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // 两个 future 都已完成 —— 我们可以成功返回了
            Poll::Ready(())
        } else {
            // 一个或两个 future 返回了 `Poll::Pending`，且仍有
            // 工作要做。当可以取得进展时，它们会调用 `wake()`。
            Poll::Pending
        }
    }
}
// ANCHOR_END: join

// ANCHOR: and_then
/// 一个依次运行两个 future 直至完成的 SimpleFuture。
//
// 注意：在这个简单示例中，`AndThenFut` 假定第一个和第二个 future
// 在创建时就都是可用的。真正的 `AndThen` 组合子允许基于第一个
// future 的输出来创建第二个 future，例如
// `get_breakfast.and_then(|food| eat(food))`。
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // 第一个 future 完成了 —— 移除它，接着开始第二个！
                Poll::Ready(()) => self.first.take(),
                // 第一个 future 还无法完成。
                // 注意，我们用 `return` 语句中断了 `poll` 函数的执行流。
                Poll::Pending => return Poll::Pending,
            };
        }
        // 第一个 future 完成之后，接着尝试完成第二个。
        self.second.poll(wake)
    }
}
// ANCHOR_END: and_then

mod real_future {
use std::{
    future::Future as RealFuture,
    pin::Pin,
    task::{Context, Poll},
};

// ANCHOR: real_future
trait Future {
    type Output;
    fn poll(
        // 注意这里从 `&mut self` 变成了 `Pin<&mut Self>`：
        self: Pin<&mut Self>,
        // 以及这里从 `wake: fn()` 变成了 `cx: &mut Context<'_>`：
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>;
}
// ANCHOR_END: real_future

// ensure that `Future` matches `RealFuture`:
impl<O> Future for dyn RealFuture<Output = O> {
    type Output = O;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RealFuture::poll(self, cx)
    }
}
}
