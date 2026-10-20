// ANCHOR: imports
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};
// ANCHOR_END: imports

// ANCHOR: timer_decl
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// future 和等待线程之间共享的状态
struct SharedState {
    /// 睡眠时间是否已经结束
    completed: bool,

    /// `TimerFuture` 所运行任务的 waker。
    /// 线程在设置 `completed = true` 之后，可以用它来
    /// 唤醒 `TimerFuture` 的任务，让它看到 `completed = true`
    /// 并继续向前推进。
    waker: Option<Waker>,
}
// ANCHOR_END: timer_decl

// ANCHOR: future_for_timer
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 查看共享状态，看看定时器是否已经完成。
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // 设置 waker，以便线程在定时器结束后可以唤醒当前任务，
            // 确保 future 会被再次轮询，并看到 `completed = true`。
            //
            // 人们很容易在这里只设置一次，而不是每次都克隆一遍
            // waker。然而，`TimerFuture` 可能会在执行器的任务之间
            // 移动，这可能导致一个指向错误任务的过期 waker，
            // 使 `TimerFuture` 无法被正确唤醒。
            //
            // 注意：可以用 `Waker::will_wake` 函数来检查这一点，
            // 但为了保持简单，我们在这里省略了这一步。
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
// ANCHOR_END: future_for_timer

// ANCHOR: timer_new
impl TimerFuture {
    /// 创建一个新的 `TimerFuture`，它将在给定的超时时间之后完成。
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // 启动新线程
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // 发出定时器已完成的信号，并唤醒最后一次轮询该 future
            // 的任务（如果存在的话）。
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}
// ANCHOR_END: timer_new

#[test]
fn block_on_timer() {
    futures::executor::block_on(async {
        TimerFuture::new(Duration::from_secs(1)).await
    })
}
