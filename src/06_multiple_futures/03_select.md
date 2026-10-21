# `select!`

`futures::select` 宏同时运行多个 future，使得一旦任何一个 future 完成，就能立即做出响应。

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:example}}
```

上面的函数会并发地运行 `t1` 和 `t2`。当 `t1` 或 `t2` 中任意一个完成时，对应的处理分支就会调用 `println!`，然后该函数就会结束，而不会去完成剩余的任务。

`select` 的基本语法是 `<pattern> = <expression> => <code>,`，需要 `select` 多少个 future，就重复多少次。

## `default => ...` 与 `complete => ...`

`select` 还支持 `default` 和 `complete` 分支。

如果被 `select` 的所有 future 都尚未完成，就会运行 `default` 分支。因此，带 `default` 分支的 `select` 总是会立即返回，因为在其它 future 都未就绪时会运行 `default`。

`complete` 分支可用于处理被 `select` 的所有 future 都已完成、不再有新进展的情况。在循环中使用 `select!` 时，这通常很有用。

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:default_and_complete}}
```

## 与 `Unpin` 和 `FusedFuture` 的交互

你可能已经注意到，在上面的第一个例子中，我们必须对两个 `async fn` 返回的 future 调用 `.fuse()`，并用 `pin_mut` 将它们固定。这两个调用都是必需的，因为 `select` 中使用的 future 必须同时实现 `Unpin` trait 和 `FusedFuture` trait。

需要 `Unpin` 是因为 `select` 使用的 future 不是按值传入的，而是通过可变引用传入的。由于不获取 future 的所有权，未完成的 future 在 `select` 调用之后还可以再次使用。

类似地，`FusedFuture` trait 也是必需的，因为 `select` 不能在某个 future 完成之后再轮询它。`FusedFuture` 由那些会跟踪自己是否已完成的 future 实现。这使得我们可以在循环中使用 `select`，只轮询那些尚未完成的 future。在上面的例子中就能看到这一点：第二轮循环时，`a_fut` 或 `b_fut` 已经完成了。由于 `future::ready` 返回的 future 实现了 `FusedFuture`，它能够告诉 `select` 不要再次轮询它。

注意，流（stream）有对应的 `FusedStream` trait。实现了该 trait 或者用 `.fuse()` 包装过的流，其 `.next()` / `.try_next()` 组合子产生的 future 都实现了 `FusedFuture`。

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:fused_stream}}
```

## 在 `select` 循环中使用 `Fuse` 与 `FuturesUnordered` 处理并发任务

有一个不太容易发现但很实用的函数 `Fuse::terminated()`，它可以构造一个已经终止的空 future，之后可以再用需要运行的 future 来填充它。

当有一个任务需要在 `select` 循环期间运行、但它又是在 `select` 循环内部创建的时候，这会非常有用。

注意 `.select_next_some()` 函数的用法。它可以与 `select` 配合使用，只为流返回的 `Some(_)` 值运行对应分支，而忽略 `None`。

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:fuse_terminated}}
```

当需要同时运行同一个 future 的多个副本时，可以使用 `FuturesUnordered` 类型。下面的例子与上面的类似，但它会让每个 `run_on_new_num_fut` 副本都运行至完成，而不是在创建新副本时中止旧的副本。它还会打印出 `run_on_new_num_fut` 返回的值。

```rust,edition2018
{{#include ../../examples/06_03_select/src/lib.rs:futures_unordered}}
```
