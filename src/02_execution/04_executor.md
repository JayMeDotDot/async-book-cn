# 实践：构建一个执行器

Rust 的 `Future` 是惰性的：除非被主动驱动至完成，否则它们什么也不会做。驱动 future 完成的一种方式是在 `async` 函数中 `.await` 它，但这只是把问题向上推了一层：谁来运行顶层 `async` 函数返回的 future 呢？答案是，我们需要一个 `Future` 执行器。

`Future` 执行器接收一组顶层 `Future`，并在 `Future` 能够取得进展时调用 `poll`，把它们运行至完成。通常，执行器会先对一个 future 进行一次 `poll` 来启动它。当 `Future` 通过调用 `wake()` 表示自己已准备好取得进展时，它就会被重新放回队列，再次调用 `poll`，如此往复，直到 `Future` 完成。

在本节中，我们将编写一个自己的简单执行器，它能够并发地把大量顶层 future运行至完成。

在这个例子中，我们依赖 `futures` crate 提供的 `ArcWake` trait，它提供了一种简便的方式来构造 `Waker`。编辑 `Cargo.toml`，添加一个新的依赖：

```toml
[package]
name = "timer_future"
version = "0.1.0"
authors = ["XYZ Author"]
edition = "2021"

[dependencies]
futures = "0.3"
```

接下来，我们需要在 `src/main.rs` 顶部添加以下导入：

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:imports}}
```

我们的执行器将采用这样的工作方式：通过 channel 发送要运行的任务。执行器从 channel 中取出事件并运行它们。当一个任务准备好做更多工作（被唤醒）时，它可以通过把自己重新放回 channel 来安排自己再次被轮询。

在这种设计中，执行器本身只需要任务 channel 的接收端。用户会拿到发送端，以便生成新的 future。任务本身只是能够重新调度自己的 future，因此我们将把它们存储为一个 future，并配上一个发送端，任务可以用它把自己重新放回队列。

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:executor_decl}}
```

我们再给 spawner 添加一个方法，让生成新 future 变得更容易。这个方法接收一个 future 类型，把它装箱，并创建一个包含它的 `Arc<Task>`，然后就可以把它放入执行器的队列中了。

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:spawn_fn}}
```

要轮询 future，我们需要创建一个 `Waker`。正如在[任务唤醒一节](./03_wakeups.md)中讨论的， `Waker` 负责在 `wake` 被调用后安排任务再次被轮询。请记住，`Waker` 会确切地告诉执行器哪个任务已经就绪，从而让执行器只轮询那些准备好取得进展的 future。创建新 `Waker` 的最简单方式是实现 `ArcWake` trait，然后使用 `waker_ref` 或 `.into_waker()` 函数把 `Arc<impl ArcWake>` 转换为 `Waker`。让我们为任务实现 `ArcWake`，使它们能够被转换为 `Waker` 并被唤醒：

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:arcwake_for_task}}
```

当 `Waker` 由 `Arc<Task>` 创建时，对它调用 `wake()` 会使一份 `Arc` 的副本被发送到任务 channel 上。然后，我们的执行器需要取出这个任务并轮询它。让我们来实现这一点：

```rust,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:executor_run}}
```

恭喜！我们现在已经有了一个可以工作的 future 执行器。我们甚至可以用它来运行 `async/.await` 代码和自定义 future，比如我们之前编写的 `TimerFuture`：

```rust,edition2018,ignore
{{#include ../../examples/02_04_executor/src/lib.rs:main}}
```
