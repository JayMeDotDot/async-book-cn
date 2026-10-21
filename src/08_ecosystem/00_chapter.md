# 异步生态

Rust 目前仅为编写异步代码提供了最基本的支持。重要的是，执行器（executor）、任务（task）、反应器（reactor）、组合器（combinator）以及底层 I/O 的 future 和 trait尚未被纳入标准库。在此期间，由社区提供的异步生态填补了这些空白。

异步基础工作组（Async Foundations Team）有兴趣扩展《Async Book》中的示例，使其覆盖多个运行时。如果你有兴趣为这个项目做贡献，请通过 [Zulip](https://rust-lang.zulipchat.com/#narrow/stream/201246-wg-async-foundations.2Fbook) 联系我们。

## 异步运行时

异步运行时（runtime）是用于执行异步应用程序的库。运行时通常将一个 *reactor* 与一个或多个 *executor* 捆绑在一起。反应器为外部事件提供订阅机制，例如异步 I/O、进程间通信和定时器。在异步运行时中，订阅者通常是代表底层 I/O 操作的 future。执行器负责任务的调度与执行。它们跟踪正在运行和已挂起的任务，轮询 future 直至其完成，并在任务可以取得进展时将其唤醒。"executor" 一词经常与 "runtime" 混用。在这里，我们使用 "ecosystem" 一词来描述与兼容的 trait 和特性捆绑在一起的运行时。

## 社区提供的异步 crate

### futures crate

[`futures` crate](https://docs.rs/futures/) 包含对编写异步代码有用的 trait 和函数。其中包括 `Stream`、`Sink`、`AsyncRead` 和 `AsyncWrite` trait，以及诸如组合器之类的实用工具。这些实用工具和 trait 最终可能会成为标准库的一部分。

`futures` 自带执行器，但没有自带反应器，因此它不支持执行异步 I/O 或定时器 future。出于这个原因，它不被视为一个完整的运行时。常见的做法是将 `futures` 的实用工具与来自其他 crate 的执行器搭配使用。

### 常见的异步运行时

标准库中没有异步运行时，官方也没有推荐任何运行时。以下 crate 提供了常见的运行时。

- [Tokio](https://docs.rs/tokio/)：一个流行的异步生态，拥有 HTTP、gRPC 和 tracing 框架。

- [async-std](https://docs.rs/async-std/)：一个为标准库组件提供异步对应实现的 crate。

- [smol](https://docs.rs/smol/)：一个小巧、精简的异步运行时。
它提供了 `Async` trait，可用于包装 `UnixStream` 或 `TcpListener` 之类的结构体。

- [fuchsia-async](https://fuchsia.googlesource.com/fuchsia/+/master/src/lib/fuchsia-async/)：
一个用于 Fuchsia OS 的执行器。

## 判断生态兼容性

并非所有异步应用程序、框架和库都彼此兼容，或与所有操作系统和平台兼容。大多数异步代码可以在任何生态中使用，但某些框架和库要求使用特定的生态。生态约束并不总是有文档说明，但有几条经验法则可以用来判断某个库、trait 或函数是否依赖于特定的生态。

任何与异步 I/O、定时器、进程间通信或任务交互的异步代码，通常都依赖于特定的异步执行器或反应器。所有其他异步代码，例如异步表达式、组合器、同步类型和流（stream），通常是生态无关的，前提是其中嵌套的 future 也都是生态无关的。在开始一个项目之前，建议先调研相关的异步框架和库，以确保它们与你选择的运行时兼容，并且彼此之间也兼容。

值得注意的是，`Tokio` 使用 `mio` 反应器，并定义了自己的异步 I/O trait 版本，包括 `AsyncRead` 和 `AsyncWrite`。就其本身而言，它与 `async-std` 和 `smol` 并不兼容，后者依赖 [`async-executor` crate](https://docs.rs/async-executor)，以及 `futures` 中定义的 `AsyncRead` 和 `AsyncWrite` trait。

冲突的运行时需求有时可以通过兼容层来解决，兼容层允许你在另一个运行时中调用为某个运行时编写的代码。例如，[`async_compat` crate](https://docs.rs/async_compat) 提供了 `Tokio` 与其他运行时之间的兼容层。

暴露异步 API 的库不应依赖于特定的执行器或反应器，除非它们需要生成任务，或定义自己的异步 I/O 或定时器 future。理想情况下，只有二进制可执行程序才应该负责任务的调度和运行。

## 单线程执行器与多线程执行器

异步执行器可以是单线程的，也可以是多线程的。例如，`async-executor` crate 既有单线程的 `LocalExecutor`，也有多线程的 `Executor`。

多线程执行器可以同时推进多个任务。对于包含大量任务的工作负载，它可以大幅加快执行速度，但在任务之间同步数据通常开销更大。在单线程运行时和多线程运行时之间做选择时，建议先对你的应用程序进行性能测量。

任务既可以在创建它的线程上运行，也可以在单独的线程上运行。
异步运行时通常提供将任务生成到单独线程上的功能。
即使任务在单独的线程上执行，它们也仍然应该是非阻塞的。
为了在多线程执行器上调度任务，任务还必须实现 `Send`。
一些运行时提供了生成非 `Send` 任务的函数，
这能确保每个任务都在生成它的线程上执行。
它们还可能提供将阻塞任务生成到专用线程的函数，
这对于运行来自其他库的阻塞式同步代码很有用。
