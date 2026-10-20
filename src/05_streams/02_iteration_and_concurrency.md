# 迭代与并发

与同步的 `Iterator` 类似，迭代和处理 `Stream` 中的值有许多不同的方式。既有 `map`、`filter` 和 `fold` 这类组合子风格的方法，也有它们在出错时提前退出的表亲 `try_map`、`try_filter` 和 `try_fold`。

遗憾的是，`for` 循环无法用于 `Stream`，但对于命令式风格的代码，可以使用 `while let` 以及 `next`/`try_next` 函数：

```rust,edition2018,ignore
{{#include ../../examples/05_02_iteration_and_concurrency/src/lib.rs:nexts}}
```

然而，如果我们只是一次处理一个元素，就可能错失并发带来的机会——毕竟，这正是我们一开始编写异步代码的原因。要并发地处理流中的多个条目，可以使用 `for_each_concurrent` 和 `try_for_each_concurrent` 方法：

```rust,edition2018,ignore
{{#include ../../examples/05_02_iteration_and_concurrency/src/lib.rs:try_for_each_concurrent}}
```
