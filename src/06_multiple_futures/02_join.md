# `join!`

`futures::join` 宏使得可以在并发执行多个不同 future 的同时，等待它们全部完成。

## `join!`

在执行多个异步操作时，很容易想到直接用 `.await` 依次等待它们：

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:naiive}}
```

然而，这会比必要的速度更慢，因为只有等 `get_book` 完成之后才会开始尝试 `get_music`。在某些其它语言中，future 会自动在后台运行直至完成，因此可以先调用每个 `async fn` 来启动 future、然后再同时 await 它们，从而并发地执行两个操作：

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:other_langs}}
```

然而，Rust 的 future 在被主动 `.await` 之前不会做任何工作。这意味着上面两个代码片段都会串行地运行 `book_future` 和 `music_future`，而不是并发运行。要正确地并发运行这两个 future，请使用 `futures::join!`：

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:join}}
```

`join!` 返回的值是一个元组，其中包含传入的每个 `Future` 的输出。

## `try_join!`

对于返回 `Result` 的 future，可以考虑使用 `try_join!` 而不是 `join!`。由于 `join!` 只有在所有子 future 都完成后才会完成，因此即使其中某个子 future 已经返回了 `Err`，它仍会继续处理其它 future。

与 `join!` 不同，只要有一个子 future 返回错误，`try_join!` 就会立即完成。

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:try_join}}
```

注意，传给 `try_join!` 的所有 future 必须具有相同的错误类型。可以使用 `futures::future::TryFutureExt` 中的 `.map_err(|e| ...)` 和 `.err_into()` 函数来统一错误类型：

```rust,edition2018,ignore
{{#include ../../examples/06_02_join/src/lib.rs:try_join_map_err}}
```
