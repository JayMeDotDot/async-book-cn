# trait 中的 `async`

目前，在 Rust 的稳定版中，`async fn` 还不能用于 trait。自 2022 年 11 月 17 日起，async-fn-in-trait 的 MVP 已在编译器工具链的 nightly 版本上可用，[详情请见这里](https://blog.rust-lang.org/inside-rust/2022/11/17/async-fn-in-trait-nightly.html)。

与此同时，稳定版工具链上有一个变通方案，即使用 [crates.io 上的 async-trait crate](https://github.com/dtolnay/async-trait)。

请注意，使用这些 trait 方法会导致每次函数调用都进行一次堆分配。对于绝大多数应用程序来说，这并不是什么显著的开销，但如果一个预计每秒会被调用数百万次的底层函数要将其用于公开 API，那么在决定是否使用该功能时就应当考虑到这一点。

最新进展：https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html
