# `async`/`.await` 入门

`async`/`.await` 是 Rust 的内置工具，用于编写看起来像同步代码的异步函数。`async` 会把一段代码块转换成一个实现了 `Future` trait 的状态机。在同步方法中调用阻塞函数会阻塞整个线程，而被阻塞的 `Future` 则会让出对线程的控制，允许其他`Future` 继续运行。

让我们往 `Cargo.toml` 文件中添加一些依赖：

```toml
{{#include ../../examples/01_04_async_await_primer/Cargo.toml:9:10}}
```

要创建异步函数，你可以使用 `async fn` 语法：

```rust,edition2018
async fn do_something() { /* ... */ }
```

`async fn` 返回的值是一个 `Future`。要让任何事情真正发生，
这个 `Future` 需要在执行器（executor）上运行。

```rust,edition2018
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:hello_world}}
```

在 `async fn` 内部，你可以使用 `.await` 来等待另一个实现了 `Future` trait
的类型的完成，例如另一个 `async fn` 的输出。与 `block_on` 不同，
`.await` 不会阻塞当前线程，而是异步地等待 future 完成——当该 future
当前无法取得进展时，允许其他任务运行。

例如，假设我们有三个 `async fn`：`learn_song`、`sing_song`
和 `dance`：

```rust,ignore
async fn learn_song() -> Song { /* ... */ }
async fn sing_song(song: Song) { /* ... */ }
async fn dance() { /* ... */ }
```

学习、唱歌和跳舞的一种方式是逐个地对它们进行阻塞：

```rust,ignore
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:block_on_each}}
```

然而，这种方式并没有发挥出最佳性能——我们同一时刻只在做一个事情！显然，我们必须先学会这首歌才能唱它，但是可以一边学歌和唱歌、一边跳舞。要做到这一点，我们可以创建两个单独的、可以并发运行的 `async fn`：

```rust,ignore
{{#include ../../examples/01_04_async_await_primer/src/lib.rs:block_on_main}}
```

In this example, learning the song must happen before singing the song, but
both learning and singing can happen at the same time as dancing. If we used
`block_on(learn_song())` rather than `learn_song().await` in `learn_and_sing`,
the thread wouldn't be able to do anything else while `learn_song` was running.
This would make it impossible to dance at the same time. By `.await`-ing
the `learn_song` future, we allow other tasks to take over the current thread
if `learn_song` is blocked. This makes it possible to run multiple futures
to completion concurrently on the same thread.

在这个例子中，学歌必须发生在唱歌之前，但是学歌和唱歌都可以与跳舞同时进行。如果我们在 `learn_and_sing` 中使用 `block_on(learn_song())` 而不是 `learn_song().await`，那么在 `learn_song` 运行期间，线程就无法做任何其他事情。这样就不可能同时跳舞了。通过 `.await` 这个 `learn_song` future，当 `learn_song` 被阻塞时，我们允许其他任务接管当前线程。这使得多个 future 能够并发地在同一个线程上运行直至完成。
