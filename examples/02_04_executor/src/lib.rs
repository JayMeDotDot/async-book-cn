#![cfg(test)]

// ANCHOR: imports
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};
// 我们在上一节编写的定时器：
use timer_future::TimerFuture;
// ANCHOR_END: imports

// ANCHOR: executor_decl
/// 任务执行器，从 channel 中接收任务并运行它们。
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` 把新的 future 生成到任务 channel 上。
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// 一个能够重新调度自己、以被 `Executor` 轮询的 future。
struct Task {
    /// 正在进行中的、应当被推进至完成的 future。
    ///
    /// 从正确性的角度来说，`Mutex` 并非必需，因为我们同一时刻
    /// 只有一个线程在执行任务。然而，Rust 并不够聪明，无法知道
    /// `future` 只会被一个线程修改，所以我们需要用 `Mutex`
    /// 来证明线程安全。生产环境的执行器不需要这样做，
    /// 可以改用 `UnsafeCell`。
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// 用于把任务本身重新放回任务队列的句柄。
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // channel 中允许同时排队的任务数上限。
    // 这只是为了让 `sync_channel` 满意，真实的执行器
    // 中并不会存在这个限制。
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}
// ANCHOR_END: executor_decl

// ANCHOR: spawn_fn
impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.try_send(task).expect("too many tasks queued");
    }
}
// ANCHOR_END: spawn_fn

// ANCHOR: arcwake_for_task
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // 通过把这个任务重新发送到任务 channel 上来实现 `wake`，
        // 这样它就会被执行器再次轮询。
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .try_send(cloned)
            .expect("too many tasks queued");
    }
}
// ANCHOR_END: arcwake_for_task

// ANCHOR: executor_run
impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // 取出这个 future，如果它尚未完成（仍是 Some），
            // 就轮询它以尝试使其完成。
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // 从任务本身创建一个 `LocalWaker`
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // `BoxFuture<T>` 是
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`
                // 的类型别名。我们可以通过调用 `Pin::as_mut` 方法，
                // 从中得到一个 `Pin<&mut dyn Future + Send + 'static>`。
                if future.as_mut().poll(context).is_pending() {
                    // future 还没有处理完，所以把它放回所属的任务中，
                    // 留待以后再次运行。
                    *future_slot = Some(future);
                }
            }
        }
    }
}
// ANCHOR_END: executor_run

// ANCHOR: main
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // 生成一个任务，在等待定时器前后各打印一行。
    spawner.spawn(async {
        println!("howdy!");
        // 等待我们的定时器 future 在两秒后完成。
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // 丢弃 spawner，让执行器知道已经结束，
    // 不会再收到新的任务来运行。
    drop(spawner);

    // 运行执行器，直到任务队列为空。
    // 这会先打印 "howdy!"，停顿一下，然后打印 "done!"。
    executor.run();
}
// ANCHOR_END: main

#[test]
fn run_main() {
    main()
}
