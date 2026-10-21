# `Send` 的近似判定

有些 `async fn` 的状态机可以安全地跨线程发送，而另一些则不行。一个 `async fn` 的 `Future` 是否为 `Send`，取决于是否有非 `Send` 的类型被跨 `.await` 点持有。编译器会尽力近似判断值何时可能被跨 `.await` 点持有，但目前这种分析在许多地方都过于保守。

例如，考虑一个简单的非 `Send` 类型，比如一个包含 `Rc` 的类型：

```rust
use std::rc::Rc;

#[derive(Default)]
struct NotSend(Rc<()>);
```

`NotSend` 类型的变量可以作为临时值短暂地出现在 `async fn` 中，即使该 `async fn` 返回的 `Future` 类型必须是 `Send`：

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
async fn bar() {}
async fn foo() {
    NotSend::default();
    bar().await;
}

fn require_send(_: impl Send) {}

fn main() {
    require_send(foo());
}
```

然而，如果我们修改 `foo`，把 `NotSend` 存入一个变量，这个例子就无法编译了：

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
# async fn bar() {}
async fn foo() {
    let x = NotSend::default();
    bar().await;
}
# fn require_send(_: impl Send) {}
# fn main() {
#    require_send(foo());
# }
```

```
error[E0277]: `std::rc::Rc<()>` cannot be sent between threads safely
  --> src/main.rs:15:5
   |
15 |     require_send(foo());
   |     ^^^^^^^^^^^^ `std::rc::Rc<()>` cannot be sent between threads safely
   |
   = help: within `impl std::future::Future`, the trait `std::marker::Send` is not implemented for `std::rc::Rc<()>`
   = note: required because it appears within the type `NotSend`
   = note: required because it appears within the type `{NotSend, impl std::future::Future, ()}`
   = note: required because it appears within the type `[static generator@src/main.rs:7:16: 10:2 {NotSend, impl std::future::Future, ()}]`
   = note: required because it appears within the type `std::future::GenFuture<[static generator@src/main.rs:7:16: 10:2 {NotSend, impl std::future::Future, ()}]>`
   = note: required because it appears within the type `impl std::future::Future`
   = note: required because it appears within the type `impl std::future::Future`
note: required by `require_send`
  --> src/main.rs:12:1
   |
12 | fn require_send(_: impl Send) {}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: aborting due to previous error

For more information about this error, try `rustc --explain E0277`.
```

这个错误是正确的。如果我们将 `x` 存入一个变量，它直到 `.await` 之后才会被销毁（drop），而那时 `async fn` 可能正运行在另一个线程上。由于 `Rc` 不是 `Send` 的，让它跨线程传递将是不健全的（unsound）。一个简单的解决办法是在 `.await` 之前 `drop` 掉这个 `Rc`，但遗憾的是，如今这种做法行不通。

要成功绕过这个问题，你可能需要引入一个块作用域，把所有非 `Send` 的变量封装在其中。这样编译器就能更容易地判断出这些变量并没有跨 `.await` 点存活。

```rust,edition2018
# use std::rc::Rc;
# #[derive(Default)]
# struct NotSend(Rc<()>);
# async fn bar() {}
async fn foo() {
    {
        let x = NotSend::default();
    }
    bar().await;
}
# fn require_send(_: impl Send) {}
# fn main() {
#    require_send(foo());
# }
```
