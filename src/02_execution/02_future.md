# `Future` Trait

`Future` trait 是 Rust 异步编程的核心。`Future` 是一个可以产生值的异步计算（尽管这个值可能为空，例如 `()`）。该 trait 的一个 *简化* 版本大致如下：

```rust
{{#include ../../examples/02_02_future_trait/src/lib.rs:simple_future}}
```

Future 可以通过调用 `poll` 函数来推进，该函数会尽可能地把 future 向完成状态推进。如果 future 完成了，它会返回 `Poll::Ready(result)`；如果 future 还无法完成，它会返回 `Poll::Pending`，并安排在 `Future` 可以取得更多进展时调用 `wake()` 函数。当 `wake()` 被调用时，驱动该 `Future` 的执行器会再次调用 `poll`，让 `Future` 继续取得进展。

如果没有 `wake()`，执行器将无从得知某个特定的 future 何时能够取得进展，只能不断地轮询每一个 future。有了 `wake()`，执行器就能确切地知道哪些 future 已经准备好被 `poll` 了。

举个例子，假设我们想从一个 socket 中读取数据，而 socket 中可能已有数据，也可能还没有。如果有数据，我们可以读入并返回 `Poll::Ready(data)`；但如果没有数据就绪，我们的 future 就会被阻塞，无法继续取得进展。当没有数据可用时，我们必须注册 `wake`，以便当 socket 上的数据就绪时被调用，从而告诉执行器我们的 future 已经准备好继续推进了。一个简单的 `SocketRead` future 大致如下：

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:socket_read}}
```

这种 `Future` 模型使得我们可以把多个异步操作组合在一起，而不需要中间分配。同时运行多个 future，或者将多个 future 链接起来，都可以通过免分配的状态机来实现，就像这样：

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:join}}
```

这展示了如何同时运行多个 future 而无需单独的内存分配，从而编写出更高效的异步程序。类似地，多个顺序执行的 future 也可以一个接一个地运行，就像这样：

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:and_then}}
```

这些例子展示了如何使用 `Future` trait 来表达异步控制流，而不需要多个已分配的对象和深度嵌套的回调。基本的控制流讲完了，接下来让我们谈谈真正的 `Future` trait，以及它有什么不同。

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:real_future}}
```

你会注意到的第一个变化是，我们的 `self` 类型不再是 `&mut Self`，而是变成了 `Pin<&mut Self>`。我们会在后面的章节中详细讨论固定（pinning），现在只需知道它允许我们创建不可移动的 future。不可移动的对象可以在其字段之间存储指针，例如 `struct MyFut { a: i32, ptr_to_a: *const i32 }`。固定是实现 async/await 所必需的。

其次，`wake: fn()` 变成了 `&mut Context<'_>`。在 `SimpleFuture` 中，我们通过调用一个函数指针（`fn()`）来告诉 future 执行器应当轮询这个 future。然而，由于 `fn()` 只是一个函数指针，它无法存储任何关于是 *哪个* `Future` 调用了 `wake` 的信息。

在真实场景中，像 web 服务器这样的复杂应用可能有成千上万个不同的连接，它们的唤醒都需要被分别管理。`Context` 类型通过提供对 `Waker` 类型值的访问来解决了这个问题，而 `Waker` 可以用来唤醒特定的任务。
