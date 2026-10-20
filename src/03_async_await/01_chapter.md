# `async`/`.await`

在[第一章](../01_getting_started/04_async_await_primer.md)中，我们简要地了解了 `async`/`.await`。本章将更详细地讨论 `async`/`.await`，解释它是如何工作的，以及 `async` 代码与传统 Rust 程序的区别。

`async`/`.await` 是 Rust 中一些特殊的语法，它们让当前线程可以让出控制权而不是阻塞，从而在等待某个操作完成的同时，允许其他代码继续运行。

使用 `async` 主要有两种方式：`async fn` 和 `async` 块。它们都会返回一个实现了 `Future` trait 的值：

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_fn_and_block_examples}}
```

正如我们在第一章中看到的，`async` 函数体和其他 future 都是惰性的：在被运行之前它们什么也不做。运行一个 `Future` 最常见的方式是对它使用 `.await`。当对一个 `Future` 调用 `.await` 时，它会尝试将其运行至完成。如果该 `Future` 被阻塞，它就会让出当前线程的控制权。当可以取得进一步进展时，该 `Future` 会被执行器重新拾起并恢复运行，从而使 `.await` 得到解决。

## `async` 的生命周期

与传统函数不同，接受引用或其他非 `'static` 参数的 `async fn` 会返回一个受参数生命周期约束的 `Future`：

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:lifetimes_expanded}}
```

这意味着，从 `async fn` 返回的 future 必须在其非 `'static` 参数仍然有效的期间被 `.await`。在调用函数后立即 `.await` 该 future 的常见情形下（例如 `foo(&x).await`），这不是问题。然而，如果把 future 存储起来，或者把它发送到另一个任务或线程，这可能会成为问题。

要把一个以引用作为参数的 `async fn` 转换成 `'static` future，一个常见的变通方法是将参数连同对 `async fn` 的调用一起放进一个 `async` 块中：

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:static_future_with_borrow}}
```

通过把参数移动进 `async` 块，我们将其生命周期延长到与调用 `good` 所返回的 `Future` 的生命周期一致。

## `async move`

`async` 块和闭包也允许使用 `move` 关键字，这与普通闭包一样。`async move` 块会取得它所引用的变量的所有权，这使它能够活得比当前作用域更久，但也放弃了与其他代码共享这些变量的能力：

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_move_examples}}
```

## 在多线程执行器上进行 `.await`

注意，在使用多线程 `Future` 执行器时，一个 `Future` 可能在线程之间移动，因此 `async` 函数体中使用的任何变量都必须能够在线程之间传递，因为任何 `.await` 都可能导致切换到一个新的线程。


这意味着使用 `Rc`、`&RefCell` 或任何其他未实现 `Send` trait 的类型都是不安全的，包括对未实现 `Sync` trait 的类型的引用。

（注意：只要这些类型在对 `.await` 的调用期间不处于作用域内，仍然是可以使用它们的。）

同样地，在 `.await` 期间持有传统的、不感知 future 的锁也不是一个好主意，因为它可能导致线程池卡死：一个任务获取了锁，然后 `.await` 并让出给执行器，使得另一个任务尝试获取同一把锁，从而造成死锁。为了避免这种情况，请使用 `futures::lock` 中的 `Mutex`，而不是 `std::sync` 中的那个。
