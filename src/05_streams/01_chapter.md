# `Stream` Trait（流特质）

`Stream` trait 与 `Future` 类似，但它可以在完成之前产生多个值，类似于标准库中的 `Iterator` trait：

```rust,ignore
{{#include ../../examples/05_01_streams/src/lib.rs:stream_trait}}
```

一个常见的 `Stream` 例子是 `futures` crate 中 channel 类型的 `Receiver`。每当 `Sender` 端发送一个值，它就会产生 `Some(val)`；而当 `Sender` 被 drop 且所有待处理的消息都已被接收后，它会产生 `None`：

```rust,edition2018,ignore
{{#include ../../examples/05_01_streams/src/lib.rs:channels}}
```
