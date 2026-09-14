# 更多 async/await 主题

## 单元测试

如何对异步代码进行单元测试？问题在于，你只能在异步上下文内部使用 `await`，而 Rust 中的单元测试本身并不是异步的。幸运的是，大多数运行时都为测试提供了一个便捷的属性（attribute），类似于 `async main` 所使用的那种。使用 Tokio 时，它看起来是这样的：

```rust,norun
#[tokio::test]
async fn test_something() {
    // 在这里编写测试，可以随意使用所有你需要的 `await`。
}
```

测试有多种配置方式，详情请参阅[文档](https://docs.rs/tokio/latest/tokio/attr.test.html)。

在异步代码测试方面还有一些更高级的主题（例如：测试竞态条件、死锁等），我们稍后会在本指南中介绍其中的[一部分]()。


## 阻塞与取消

在使用异步 Rust 编程时，阻塞和取消是两个需要牢记在心的重要概念。这些概念并不局限于某个特定的特性或函数，而是贯穿整个系统的普遍性质，你必须理解它们才能写出正确的代码。

### 阻塞式 IO

当一个线程（注意这里说的是操作系统线程，而不是异步任务）无法继续推进时，我们就说它被阻塞了。这通常是因为它在等待操作系统替它完成某项任务（通常是 I/O）。重要的是，当一个线程被阻塞时，操作系统知道不应该再调度它，从而让其他线程得以推进。在多线程程序中这没问题，因为它允许其他线程在被阻塞的线程等待的同时继续工作。然而，在异步程序中，还有其他任务本应被调度到同一个操作系统线程上，但操作系统并不了解这些任务的存在，而是让整个线程一直等待。这意味着，不再是单个任务等待其 I/O 完成（这没问题），而是许多任务都不得不等待（这就有问题了）。

我们很快就会讨论非阻塞/异步 I/O。现在只需知道，非阻塞 I/O 是异步运行时所感知到的 I/O，因此只有当前任务在等待；线程本身并没有被阻塞。在异步任务中只使用非阻塞 I/O、绝不使用阻塞 I/O（Rust 标准库只提供阻塞 I/O）这一点非常重要。

### 阻塞式计算

你也可以通过执行计算来阻塞线程（这和阻塞式 I/O 不完全相同，因为操作系统并未参与，但效果类似）。如果你有一段长时间运行的计算（无论是否涉及阻塞式 I/O）而不将控制权交还给运行时，那么这个任务就永远不会给运行时的调度器去调度其他任务的机会。请记住，异步编程使用的是协作式多任务。在这里，某个任务没有进行协作，因此其他任务就得不到执行工作的机会。我们稍后会讨论缓解这一问题的方法。

阻塞整个线程的方式还有很多，在本指南中我们会多次回到阻塞这个话题。

### 取消

取消（cancellation）指的是停止一个 future（或任务）的执行。由于在 Rust 中（与许多其他 async/await 系统不同），future 必须由外部力量（例如异步运行时）来驱动推进，因此如果一个 future 不再被驱动，它就不会再执行。如果一个 future 被 drop 掉（请记住，future 只是一个普通的 Rust 对象），那么它就永远无法再取得任何进展，也就被取消了。

取消可以通过以下几种方式发起：

- 直接 drop 一个 future（如果你拥有它的所有权）。
- 对一个任务的 `JoinHandle`（或 `AbortHandle`）调用 [`abort`](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html#method.abort)。
- 通过 [`CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)（这要求被取消的 future 注意到该令牌并自行协作地取消自己）。
- 隐式地，通过类似 [`select`](https://docs.rs/tokio/latest/tokio/macro.select.html) 的函数或宏。

中间两种方式是 Tokio 特有的，不过大多数运行时都提供了类似的机制。使用 `CancellationToken` 需要被取消的 future 予以配合，而其他方式则不需要。在这些其他情况下，被取消的 future 不会收到任何取消通知，也没有机会进行清理（除了它的析构函数）。注意，即使一个 future 拥有取消令牌，它仍然可能通过其他方式被取消，而这些方式不会触发取消令牌。

从编写异步代码（在异步函数、块、future 等）的角度来看，代码可能停在任何 `await` 点（包括宏中隐藏的 await 点）上，并且再也不继续执行。为了让你的代码正确（具体来说就是做到*取消安全*（*cancellation safe*）），无论它是正常完成，还是在任意一个 await 点终止，都必须正确运行[^cfThreads]。

```rust,norun
async fn some_function(input: Option<Input>) {
    let Some(input) = input else {
        return;           // 可能在这里终止 (`return`).
    };

    let x = foo(input)?;  // 可能在这里终止 (`?`).

    let y = bar(x).await; // 可能在这里终止 (`await`).

    // ...

    //                       可能在这里终止 (隐式 return).
}
```

举一个可能出问题的例子：假设一个异步函数将数据读入一个内部缓冲区，然后 await 下一个数据。如果读取数据是破坏性的（即无法从原始来源重新读取），而该异步函数又被取消了，那么内部缓冲区就会被 drop，其中的数据也会随之丢失。重要的是要考虑：取消一个 future、重启该 future，或者启动一个触及相同数据的全新 future，会对这个 future 及其涉及的任何数据产生怎样的影响。

在本指南中，我们会多次回到取消和取消安全这个话题，并且在参考部分有一整[章]()专门讨论这个主题。

[^cfThreads]: 将异步编程中的取消与取消线程做一个比较会很有意思。取消一个线程是可能的（例如在 C 中使用 `pthread_cancel`，Rust 中没有直接的方法），但这几乎总是一个非常非常糟糕的主意，因为被取消的线程可能在任何地方终止。相比之下，取消一个异步任务只能在 await 点发生。因此，在不终止整个进程的情况下去取消一个操作系统线程是非常罕见的，作为程序员，你通常不必担心这种情况发生。然而，在异步 Rust 中，取消确实是*可能*发生的事情。我们会在后续内容中讨论如何应对它。

## 异步块

一个普通的块（`{ ... }`）在源代码中将代码组织在一起，并为名称创建一个封装作用域。在运行时，块按顺序执行，并求值为其最后一个表达式的值（如果没有尾随表达式，则求值为单元类型 `()`）。

与异步函数类似，异步块是普通块的延迟执行版本。异步块将代码和名称组织在同一作用域中，但在运行时它不会立即执行，而是求值为一个 future。要执行该块并获取结果，必须对它进行 `await`。例如：

```rust,norun
let s1 = {
    let a = 42;
    format!("The answer is {a}")
};

let s2 = async {
    let q = question().await;
    format!("The question is {q}")
};
```

如果我们执行这段代码，`s1` 会是一个可以打印的字符串，但 `s2` 会是一个 future；`question()` 并不会被调用。要打印 `s2`，我们首先需要 `s2.await`。

异步块是启动异步上下文并创建 future 的最简单方式。它常被用于创建只在某处使用一次的小型 future。

遗憾的是，异步块的控制流有些古怪。因为异步块创建的是一个 future，而不是直接执行，所以它在控制流方面的行为更像是一个函数，而不是一个普通的块。`break` 和 `continue` 无法像在普通块中那样"穿透"异步块；你必须改用 `return`：

```rust,norun
loop {
    {
        if ... {
            // ok
            continue;
        }
    }

    async {
        if ... {
            // not ok
            // continue;

            // ok - 继续执行 `loop` 的下一次循环，但请注意，如果异步块之后
            // 还有代码，那些代码会先被执行。

            return;
        }
    }.await
}
```

要实现 `break`，你需要检查该块的值（一种常见的惯用法是用 [`ControlFlow`](https://doc.rust-lang.org/std/ops/enum.ControlFlow.html) 作为块的值，也允许使用 `?`）。

同样，异步块内的 `?` 在遇到错误时会终止该 future 的执行，使得被 `await` 的块取到该错误的值，但不会退出外层函数（不像普通块中的 `?` 那样）。为此，你需要在 `await` 之后再加一个 `?`：

```rust,norun
async {
    let x = foo()?;   // 这个 `?` 只会退出异步块，而不会退出外层函数。

    consume(x);
    Ok(())
}.await?
```

令人烦恼的是，这常常会让编译器感到困惑，因为（与函数不同）异步块的"返回"类型并没有被显式声明。你可能需要为变量添加一些类型标注，或者使用 turbofish 语法来使其工作，例如在上面的例子中用 `Ok::<_, MyError>(())` 代替 `Ok(())`。

一个返回异步块的函数与异步函数非常相似。写 `async fn foo() -> ... { ... }` 大致等价于 `fn foo() -> ... { async { ... } }`。事实上，从调用者的角度来看它们是等价的，从一种形式改为另一种形式并不构成破坏性变更。此外，在实现异步 trait 时，你可以用一种形式覆盖另一种形式（见下文）。不过，你确实需要调整类型，在异步块版本中显式写出 `Future`：`async fn foo() -> Foo` 变为 `fn foo() -> impl Future<Output = Foo>`（你可能还需要显式标注其他约束，例如 `Send` 和 `'static`）。

通常你会更倾向于使用异步函数版本，因为它更简单清晰。然而，异步块版本更加灵活，因为你可以在函数被调用时执行一些代码（写在异步块外），并在结果被 await 时执行另一些代码（写在异步块内的代码）。

## 异步闭包

- 闭包
  - 即将推出 (https://github.com/rust-lang/rust/pull/132706, https://blog.rust-lang.org/inside-rust/2024/08/09/async-closures-call-for-testing.html)
  - 闭包中的异步块 vs 异步闭包

## 生命周期与借用

- 上面提到了静态生命周期（'static）
- future 上的生命周期约束（`Future + '_` 等）
- 跨越 await 点的借用
- 我也不太确定，异步函数中肯定还有更多的生命周期问题……


## Future 上的 `Send + 'static` 约束

- 它们存在的原因，多线程运行时
- 使用 spawn local 来避免这些约束
- 是什么让一个异步函数成为 `Send + 'static`，以及如何修复相关 bug

## 异步 trait

- 语法
  - `Send + 'static` 问题及其解决办法
    - trait_variant
    - 显式 future
    - [返回类型表示法（return type notation）](https://blog.rust-lang.org/inside-rust/2024/09/26/rtn-call-for-testing.html)
- 覆盖
  - 方法中 future 表示法与 async 表示法的对比
- 对象安全（object safety）
- 捕获规则 (https://blog.rust-lang.org/2024/09/05/impl-trait-capture-rules.html)
- 历史与 async-trait crate

## 递归

- 允许（相对较新的特性），但需要显式地进行装箱（boxing）
  - 前向引用 future、pinning
  - [04_recursion](https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html)
  - [support-for-recursion-in-async-fn](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0.html#support-for-recursion-in-async-fn)
  - [async-recursion 宏](https://docs.rs/async-recursion/latest/async_recursion/)
