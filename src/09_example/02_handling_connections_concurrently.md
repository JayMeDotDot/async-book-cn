# 并发地处理连接

我们目前代码的问题在于，`listener.incoming()` 是一个阻塞式迭代器。当 `listener` 等待传入连接时，执行器无法运行其他 future，而且在上一条连接处理完毕之前，我们也无法处理新连接。

为了解决这个问题，我们要把 `listener.incoming()` 从阻塞式 Iterator 转换为非阻塞式 Stream。Stream 与 Iterator 类似，但可以异步地消费。更多信息请参见[Stream 章节](../05_streams/01_chapter.md)。

让我们用非阻塞的 `async_std::net::TcpListener` 替换阻塞的 `std::net::TcpListener`，并更新连接处理函数，使其接受 `async_std::net::TcpStream`：
```rust,ignore
{{#include ../../examples/09_04_concurrent_tcp_server/src/main.rs:handle_connection}}
```

`TcpListener` 的异步版本为 `listener.incoming()` 实现了 `Stream` trait，这一改动带来两个好处。第一，`listener.incoming()` 不再阻塞执行器。当没有需要处理的传入 TCP 连接时，执行器现在可以让出给其他待处理的 future。

第二个好处是可以选择并发地处理 Stream 中的元素，方法是使用 Stream 的 `for_each_concurrent` 方法。这里，我们将利用该方法并发地处理每个传入的请求。我们需要从 `futures` crate 导入 `Stream` trait，因此 Cargo.toml 现在如下所示：
```diff
+[dependencies]
+futures = "0.3"

 [dependencies.async-std]
 version = "1.6"
 features = ["attributes"]
```

现在，我们可以通过闭包函数传入 `handle_connection`，从而并发地处理每条连接。该闭包取得每个 `TcpStream` 的所有权，并且一旦有新的 `TcpStream` 可用就立即运行。只要 `handle_connection` 不阻塞，一个慢请求就不再会妨碍其他请求完成。
```rust,ignore
{{#include ../../examples/09_04_concurrent_tcp_server/src/main.rs:main_func}}
```
# 并行地服务请求

到目前为止，我们的示例基本上把协作式多任务并发（使用异步代码）作为抢占式多任务（使用线程）的替代方案来介绍。然而，异步代码与线程并不互斥。在我们的示例中，`for_each_concurrent` 并发地处理每条连接，但都是在同一线程上进行的。`async-std` crate 也允许我们把任务派生到单独的线程上。由于 `handle_connection` 既是 `Send` 的又是非阻塞的，将它与 `async_std::task::spawn` 一起使用是安全的。代码如下所示：
```rust
{{#include ../../examples/09_05_final_tcp_server/src/main.rs:main_func}}
```

现在，我们同时使用了协作式多任务并发和抢占式多任务来同时处理多个请求！更多信息请参见[多线程执行器](../08_ecosystem/00_chapter.md#single-threading-vs-multithreading)一节。
