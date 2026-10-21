# 运行异步代码

HTTP 服务器应该能够并发地服务多个客户端；也就是说，在处理当前请求之前，它不应该等待先前的请求完成。原书通过创建一个线程池[解决了这个问题](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#turning-our-single-threaded-server-into-a-multithreaded-server)，每条连接都在各自的线程上处理。在这里，我们不再通过增加线程来提高吞吐量，而是用异步代码来达到同样的效果。

让我们把 `handle_connection` 声明为 `async fn`，使其返回一个 future：
```rust,ignore
{{#include ../../examples/09_02_async_tcp_server/src/main.rs:handle_connection_async}}
```

在函数声明中加上 `async`，会把它的返回类型从单元类型 `()` 变为实现 `Future<Output=()>` 的类型。

如果我们尝试编译这段代码，编译器会警告我们这样行不通：
```console
$ cargo check
    Checking async-rust v0.1.0 (file:///projects/async-rust)
warning: unused implementer of `std::future::Future` that must be used
  --> src/main.rs:12:9
   |
12 |         handle_connection(stream);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_must_use)]` on by default
   = note: futures do nothing unless you `.await` or poll them
```

由于我们既没有 `await` 也没有 `poll` `handle_connection` 的结果，它永远不会运行。如果你运行服务器并在浏览器中访问 `127.0.0.1:7878`，会发现连接被拒绝；我们的服务器没有处理任何请求。

在纯粹的同步代码中，我们无法 `await` 或 `poll` future。我们需要一个异步运行时来负责调度，并把 future 运行至完成。关于异步运行时、执行器与反应器的更多信息，请参阅[选择运行时](../08_ecosystem/00_chapter.md)一节。其中列出的任何运行时都适用于本项目，但在这些示例中，我们选择使用 `async-std` crate。

## 添加异步运行时

下面的示例将演示如何重构同步代码以使用异步运行时；这里使用 `async-std`。`async-std` 提供的 `#[async_std::main]` 属性让我们可以编写异步的 main 函数。要使用它，需要在 `Cargo.toml` 中启用 `async-std` 的 `attributes` feature：
```toml
[dependencies.async-std]
version = "1.6"
features = ["attributes"]
```

第一步，我们改用异步的 main 函数，并 `await` 异步版 `handle_connection` 返回的 future。然后，我们测试服务器的响应情况。代码如下所示：
```rust
{{#include ../../examples/09_02_async_tcp_server/src/main.rs:main_func}}
```

现在，让我们测试服务器能否并发地处理连接。仅仅把 `handle_connection` 改成异步，并不意味着服务器就能同时处理多条连接，我们很快会看到原因。

为了说明这一点，我们来模拟一个慢请求。当客户端向 `127.0.0.1:7878/sleep` 发起请求时，我们的服务器会休眠 5 秒：

```rust,ignore
{{#include ../../examples/09_03_slow_request/src/main.rs:handle_connection}}
```

这与原书中的[慢请求模拟](https://doc.rust-lang.org/book/ch20-02-multithreaded.html#simulating-a-slow-request-in-the-current-server-implementation)非常相似，但有一个重要区别：我们使用的是非阻塞的 `async_std::task::sleep` 函数，而不是阻塞的 `std::thread::sleep` 函数。需要记住的是，即使一段代码在 `async fn` 中运行并被 `await`，它仍可能阻塞。要测试服务器是否能并发地处理连接，我们需要确保 `handle_connection` 是非阻塞的。

如果你运行服务器，会看到对 `127.0.0.1:7878/sleep` 的请求会让其他所有新进请求阻塞 5 秒！这是因为在我们 `await` `handle_connection` 的结果时，没有其他并发任务能够取得进展。下一节我们将看到如何用异步代码并发地处理连接。
