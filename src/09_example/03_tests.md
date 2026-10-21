# 测试 TCP 服务器

接下来我们来测试 `handle_connection` 函数。

首先，我们需要一个可用的 `TcpStream`。在端到端测试或集成测试中，我们可能想建立真实的 TCP 连接来测试代码。一种做法是在 `localhost` 的 0 号端口上启动监听器。端口 0 并不是一个有效的 UNIX 端口，但它可以用于测试——操作系统会为我们挑选一个空闲的 TCP 端口。

在本例中，我们改为为连接处理函数编写单元测试，检查它对相应输入返回正确的响应。为了让单元测试保持隔离且结果确定，我们将用一个 mock 替换 `TcpStream`。

首先，我们修改 `handle_connection` 的签名，使其更易于测试。`handle_connection` 实际上并不要求传入 `async_std::net::TcpStream`；它只要求传入任何实现了 `async_std::io::Read`、`async_std::io::Write` 和 `marker::Unpin` 的结构体。相应地修改类型签名后，我们就可以在测试时传入 mock。

```rust,ignore
use async_std::io::{Read, Write};

async fn handle_connection(mut stream: impl Read + Write + Unpin) {
```

接下来，我们来构建一个实现了这些 trait 的 mock `TcpStream`。先实现 `Read` trait，它只有一个方法 `poll_read`。我们的 mock `TcpStream` 会包含一些数据，这些数据会被复制到读取缓冲区中，然后我们返回 `Poll::Ready` 表示读取完成。

```rust,ignore
{{#include ../../examples/09_05_final_tcp_server/src/main.rs:mock_read}}
```

我们对 `Write` 的实现与此非常类似，只不过需要编写三个方法：`poll_write`、`poll_flush` 和 `poll_close`。`poll_write` 会把输入数据复制进 mock `TcpStream`，完成后返回 `Poll::Ready`。flush 和 close 这个 mock `TcpStream` 不需要做任何事，所以 `poll_flush` 和 `poll_close` 直接返回 `Poll::Ready` 即可。

```rust,ignore
{{#include ../../examples/09_05_final_tcp_server/src/main.rs:mock_write}}
```

最后，我们的 mock 还需要实现 `Unpin`，表明它在内存中的位置可以安全地移动。关于固定（pinning）和 `Unpin` trait 的更多信息，请参见固定（pinning）一节。

```rust,ignore
{{#include ../../examples/09_05_final_tcp_server/src/main.rs:unpin}}
```

现在我们可以测试 `handle_connection` 函数了。在搭建好包含一些初始数据的 `MockTcpStream` 之后，我们可以使用 `#[async_std::test]` 属性来运行 `handle_connection`，就像之前使用 `#[async_std::main]` 一样。为了确保 `handle_connection` 按预期工作，我们将检查写入 `MockTcpStream` 的数据是否与初始内容所对应的正确结果一致。

```rust,ignore
{{#include ../../examples/09_05_final_tcp_server/src/main.rs:test}}
```
