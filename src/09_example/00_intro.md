# 最终项目：用 Async Rust 构建并发 Web 服务器

在本章中，我们将使用异步 Rust 来改造 Rust 书籍中的[单线程 web 服务器](https://doc.rust-lang.org/book/ch20-01-single-threaded.html)，使其能够并发地服务请求。

## 回顾

下面是这一课结束时代码的样子。

`src/main.rs`:
```rust
{{#include ../../examples/09_01_sync_tcp_server/src/main.rs}}
```

`hello.html`:
```html
{{#include ../../examples/09_01_sync_tcp_server/hello.html}}
```

`404.html`:
```html
{{#include ../../examples/09_01_sync_tcp_server/404.html}}
```

如果你用 `cargo run` 运行服务器，并在浏览器中访问 `127.0.0.1:7878`，Ferris 会向你送上一段友好的问候！
