# 使用 `Waker` 唤醒任务

Future 在第一次被 `poll` 时往往无法完成。当这种情况发生时，future 需要确保自己在准备好取得更多进展时会被再次轮询。这可以通过 `Waker` 类型来实现。

每次轮询 future 时，都是把它作为某个"任务"的一部分来轮询的。任务是提交给执行器的顶层 future。

`Waker` 提供了一个 `wake()` 方法，可以用来告诉执行器应该唤醒与之关联的任务。当 `wake()` 被调用时，执行器就知道与该 `Waker` 关联的任务已经准备好取得进展，它的 future 应当被再次轮询。

`Waker` 还实现了 `clone()`，因此它可以被到处复制和存储。

让我们试着用 `Waker` 实现一个简单的定时器 future。

## 实践：构建一个定时器

为了演示这个例子，我们将在定时器创建时直接启动一个新线程，让它睡眠所需的时间，然后在时间窗口结束后通知定时器 future。

首先，使用 `cargo new --lib timer_future` 新建一个项目，并在 `src/lib.rs` 中添加我们开始所需的导入：

```rust
{{#include ../../examples/02_03_timer/src/lib.rs:imports}}
```

我们先来定义 future 类型本身。我们的 future 需要一种方式，让线程通知它定时器已经到期、future 应当完成。我们将使用一个共享的 `Arc<Mutex<..>>` 值在线程和future 之间进行通信。

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_decl}}
```

现在，让我们来真正编写 `Future` 的实现！

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:future_for_timer}}
```

相当简单，对吧？如果线程已经设置了 `shared_state.completed = true`，我们就完成了！否则，我们克隆当前任务的 `Waker`，并把它传递给 `shared_state.waker`，以便线程能够重新唤醒这个任务。

重要的是，我们必须在每次轮询 future 时都更新 `Waker`，因为这个 future 可能已经移动到了另一个具有不同 `Waker` 的任务中。当 future 被轮询之后在任务之间传递时，就会发生这种情况。

最后，我们需要一个真正构建定时器并启动线程的 API：

```rust,ignore
{{#include ../../examples/02_03_timer/src/lib.rs:timer_new}}
```

搞定！这就是构建一个简单定时器 future 所需的全部内容。现在，要是我们有一个执行器可以运行这个 future 就好了……
