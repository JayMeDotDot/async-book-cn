# 生成任务（Spawning）

生成任务（spawning）允许你在后台运行一个新的异步任务。这样我们就可以在它运行的同时继续执行其它代码。

假设我们有一个 web 服务器，希望在不阻塞主线程的情况下接受连接。为此，我们可以使用 `async_std::task::spawn` 函数来创建并运行一个处理连接的新任务。该函数接收一个 future 并返回一个 `JoinHandle`，可用于在任务完成后等待其结果。

```rust,edition2018
{{#include ../../examples/06_04_spawning/src/lib.rs:example}}
```

`spawn` 返回的 `JoinHandle` 实现了 `Future` trait，因此我们可以对它调用 `.await` 来获取任务的结果。这会阻塞当前任务，直到被生成的任务完成为止。如果不 await 该任务，程序会继续执行而不会等待它；如果函数在任务完成之前就结束了，该任务会被取消。

```rust,edition2018
{{#include ../../examples/06_04_spawning/src/lib.rs:join_all}}
```

要在主任务和被生成的任务之间通信，我们可以使用异步运行时提供的通道（channel）。
