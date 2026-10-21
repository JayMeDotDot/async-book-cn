# 递归

在内部，`async fn` 会创建一个状态机类型，其中包含每个被 `.await` 的子 `Future`。这使得递归的 `async fn` 有点棘手，因为最终生成的状态机类型必须包含它自身：

```rust,edition2018
# async fn step_one() { /* ... */ }
# async fn step_two() { /* ... */ }
# struct StepOne;
# struct StepTwo;
// 这个函数:
async fn foo() {
    step_one().await;
    step_two().await;
}
// 生成一个这样的类型:
enum Foo {
    First(StepOne),
    Second(StepTwo),
}

// 所以这个函数:
async fn recursive() {
    recursive().await;
    recursive().await;
}

// 会生成这样的类型:
enum Recursive {
    First(Recursive),
    Second(Recursive),
}
```

这是行不通的——我们创建了一个无限大小的类型！编译器会报错：

```
error[E0733]: recursion in an async fn requires boxing
 --> src/lib.rs:1:1
  |
1 | async fn recursive() {
  | ^^^^^^^^^^^^^^^^^^^^
  |
  = note: a recursive `async fn` call must introduce indirection such as `Box::pin` to avoid an infinitely sized future
```

为了允许这样做，我们必须使用 `Box` 引入一层间接。

在 Rust 1.77 之前，由于编译器的限制，仅仅把对 `recursive()` 的调用包装在 `Box::pin` 中是不够的。要使其正常工作，我们必须把 `recursive` 改写成一个非 `async` 函数，它返回一个经过 `.boxed()` 的 `async` 块：

```rust,edition2018
{{#include ../../examples/07_05_recursion/src/lib.rs:example}}
```

在新版本的 Rust 中，[那个编译器限制已被解除](https://github.com/rust-lang/rust/pull/117703/)。

自 Rust 1.77 起，借助分配间接（allocation indirection）实现的 `async fn` 递归支持[已成为稳定特性](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0.html#support-for-recursion-in-async-fn)，因此只要递归调用使用某种形式的间接来避免函数状态出现无限大小，就是允许的。

这意味着像下面这样的代码现在可以正常工作了：

```rust,edition2021
{{#include ../../examples/07_05_recursion/src/lib.rs:example_pinned}}
```
