# IO 与阻塞问题

高效处理 IO（输入/输出）是异步编程的主要动机之一，大多数异步程序都会进行大量的 IO 操作。从根本上说，IO 的问题在于它所花费的时间比计算高出好几个数量级，因此仅仅等待 IO 完成而不去做其他工作是非常低效的。理想情况下，异步编程允许程序在等待 IO 的同时继续处理其他工作。

本章将介绍异步环境下的 IO。我们将讨论阻塞式 IO 与非阻塞式 IO 之间的重要区别，以及为什么阻塞式 IO 与异步编程难以混用（至少在不花点心思和力气的情况下是这样）。我们将介绍如何使用非阻塞式 IO，然后探讨 IO 与异步编程结合时可能出现的若干问题。我们还会了解操作系统是如何处理 IO 的，并简要了解诸如 io_uring 等其他 IO 方式。

最后，我们将讨论阻塞异步任务的其他一些方式（这可不是好事），以及如何正确地将异步编程与阻塞式 IO 或长时间运行的 CPU 密集型代码结合起来。


## 阻塞式 IO 与非阻塞式 IO

IO 由操作系统实现；IO 工作发生在独立的进程中和/或专用硬件上，无论是哪种情况，都处于程序的进程之外。IO 可以是同步的，也可以是异步的（分别又称为阻塞式和非阻塞式）。同步 IO 意味着程序（或至少是线程）在 IO 进行时会等待（即阻塞），直到 IO 完成并从操作系统接收到结果后才开始处理。异步 IO 意味着程序可以在 IO 进行时继续推进工作，并在稍后获取结果。这两种 IO 都有许多不同的操作系统 API，不过异步方面的种类更为丰富。

异步 IO 与异步编程本身并没有内在联系。然而，异步编程使得异步 IO 既好用又高效，这也是异步编程的一个重要动机。由同步 IO 引起的阻塞是异步编程性能问题的主要根源之一，我们必须小心避免（详见下文）。

Rust 的标准库包含了用于阻塞式 IO 的函数和 trait。对于非阻塞式 IO，你必须使用专门的库，这些库通常是异步运行时的一部分，例如 Tokio 的 [`io`](https://docs.rs/tokio/latest/tokio/io/index.html) 模块。

我们来看一个简单的例子（改编自 Tokio 文档）：

```rust
use tokio::{io::AsyncWriteExt, net::TcpStream};

async fn write_hello() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    stream.write_all(b"hello world!").await?;

    Ok(())
}
```

`write_all` 是一个异步 IO 方法，用于将数据写入 `stream`。它可能会立即完成，但更可能需要一些时间才能完成，因此 `stream.write_all(...).await` 会让当前任务暂停，等待操作系统处理写入。调度器会运行其他任务，当写入完成后，它会唤醒该任务并将其调度以继续执行。

然而，如果我们使用标准库中的写函数，异步调度器就不会参与其中，操作系统会在 IO 完成期间暂停整个线程，这意味着不仅当前任务被暂停，该线程也无法执行任何其他任务。如果运行时线程池中的所有线程都出现这种情况（在某些情况下线程池可能只有一个线程），那么整个程序就会停止，无法继续推进。这被称为阻塞线程（或阻塞程序），对性能极为不利。在异步程序中绝不能阻塞线程，因此你应该避免在异步任务中使用阻塞式 IO。

除了阻塞式 IO 之外，长时间运行的任务或等待锁的任务也会导致线程阻塞。我们会在[本章末尾](#other-blocking-operations)进一步讨论这个问题。

反复读取或写入是一种常见的模式，而流（stream）和汇（sink，又称异步迭代器）是实现这一操作的便捷机制。它们在[专门的章节](streams.md)中有所介绍。


## 读和写

TODO

- async Read and Write traits
  - part of the runtime
- how to use
- specific implementations
  - network vs disk
    - tcp, udp
    - file system is not really async, but io_uring (ref to that chapter)
  - practical examples
  - stdout, etc.
  - pipe, fd, etc.


## 内存管理

当我们读取数据时，需要将其放在某处；当我们写入数据时，在写入完成之前需要将其保存在某处。无论是哪种情况，如何管理这些内存都很重要。

TODO


- Issues with buffer management and async IO
- Different solutions and pros and cons
  - zero-copy approach
  - shared buffer approach
- Utility crates to help with this, Bytes, etc.

## IO 高级主题

TODO


- buf read/write
- Read + Write, split, join
- copy
- simplex and duplex
- cancelation
- what if we have to do sync IO? Spawn a thread or use spawn_blocking (see below)

## 操作系统视角的 IO

TODO

- Different kinds of IO and mechanisms, completion IO, reference to completion IO chapter in adv section
  - different runtimes can faciliate this
  - mio for low-level interface


## 其他阻塞操作

正如本章开头所述，不阻塞线程对异步程序的性能至关重要。各种类型的阻塞式 IO 是常见的阻塞方式，但通过进行大量计算或以异步调度器无法协调的方式进行等待，也可能导致阻塞。

等待最常见的原因是使用了非异步感知的同步机制，例如使用 `std::sync::Mutex` 而非异步互斥锁，或等待一个非异步的 channel。我们会在 [Channel、锁与同步](sync.md) 一章中讨论这个问题。还有其他可能导致阻塞式等待的情况，一般来说，你需要找到非阻塞或对异步友好的机制，例如使用异步的 `sleep` 函数而非标准库中的版本。等待也可能是忙等待（实际上就是在不做任何工作的情况下循环，又称自旋锁），你大概应该避免这种情况。

### CPU 密集型工作

执行长时间运行（即 CPU 密集型或 CPU 受限的）工作会阻止调度器运行其他任务。这*确实*是一种阻塞，但不像在 IO 上阻塞或等待那样糟糕，因为至少你的程序还在取得一些进展。然而（如果不加以注意和考虑），它在某些方面（例如尾延迟）可能并非最优，并且如果那些无法运行的任务需要在特定时间运行，还可能引发正确性问题。有一种说法是你根本不应该用异步 Rust（或像 Tokio 这样的通用异步运行时）来做 CPU 密集型工作，但这是一种过度简化。正确的说法是，你不能在不做特殊处理的情况下混合 IO 受限和 CPU 受限（更准确地说是长时间运行和对延迟敏感）的任务，并指望一切顺利。

在本节余下部分，我们假设你同时有对延迟敏感的任务和长时间运行的 CPU 密集型任务。如果你没有任何对延迟敏感的任务，情况会有所不同（通常会简单一些）。

运行长时间运行或阻塞任务基本上有三种解决方案：使用运行时的内置设施、使用单独的线程，或使用单独的运行时。

在 Tokio 中，你可以使用 [`spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) 来生成一个可能阻塞的任务。它的工作方式类似于用于生成任务的 [`spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)，但会在一个专为可能阻塞的任务优化的独立线程池中运行该任务（该任务很可能在它自己的线程上运行）。请注意，它运行的是常规的同步代码，而非异步任务。这意味着该任务无法被取消（即使它的 `JoinHandle` 有 `abort` 方法）。其他运行时也提供了类似的功能。

这个例子使用 `spawn_blocking` 通过调用标准库中的同步文件系统函数来执行阻塞式 IO。请注意，[`tokio::fs`](https://docs.rs/tokio/latest/tokio/fs/index.html) 也存在并提供异步文件系统 API；然而在底层，它同样使用包裹在 `spawn_blocking` 中的阻塞操作。

```rust,norun
use tokio;

#[tokio::main]
async fn main() {
    let contents = tokio::task::spawn_blocking(|| {
		std::fs::read_to_string("file.txt").unwrap()
    })
	.await
	.unwrap();

	// 对 contents 做处理
}
```

由于通过 `spawn_blocking` 生成的任务无法被中止，它适用于最终会完成的工作。可能无限期阻塞的任务（例如监听传入请求的服务器）最好在专用线程上运行，这样它们就不会长时间占用 Tokio 阻塞线程池中的线程。你可以使用 [`std::thread::spawn`](https://doc.rust-lang.org/stable/std/thread/fn.spawn.html) 或类似的 API 来创建。

如果你需要运行大量任务，你可能需要某种线程池或工作调度器。如果你不断生成线程，并且线程数远多于可用核心数，最终会牺牲吞吐量。[Rayon](https://github.com/rayon-rs/rayon) 是一个流行的选择，它能轻松运行和管理并行任务。使用更针对你的工作负载和/或对所运行任务有所了解的方案，可能会获得更好的性能。

下面是一个将 Rayon 与 Tokio 结合使用的例子。它利用 [`tokio::oneshot::channel`](https://docs.rs/tokio/latest/tokio/sync/oneshot/fn.channel.html) 在 Rayon 生成的任务与 Tokio 中的当前任务之间传递结果。

```rust,norun
use rayon::prelude::*;

#[tokio::main]
async fn main() {
    let data = 1..=10;

    let (send, recv) = tokio::sync::oneshot::channel();
    // 在 rayon 上生成一个任务，以避免阻塞当前任务
    std::thread::spawn(move || {
        // 使用 rayon 的并行迭代器来并行计算结果

        let results = data.into_par_iter().map(compute).collect::<Vec<_>>();
        // 发送结果给 Tokio.
        send.send(results).unwrap();
    });

    // 等待 rayon 任务并获取结果
    let results = recv.await.unwrap();
    println!("Results: {:?}", results);
}

fn compute(input: u64) -> u64 {
    // 通过对大量整数求和来
    // 模拟一个 CPU 密集型计算。

    let mut sum = 0u64;
    for i in 0..100_000_000 {
        sum = sum.wrapping_add(i * i);
    }
    sum % input
}
```

对于 CPU 受限型任务来说，这是一个不错的解决方案，因为它是唯一一种支持将长时间运行的任务作为异步任务来运行的方案。它也很灵活（因为运行时可以配置为最适合其所运行任务的类型；实际上，要获得最佳性能，必须在运行时配置上花些心思），并且让你能够从使用 Tokio 这样成熟、设计良好的子系统中受益。你甚至可以使用两个不同的异步运行时。无论哪种情况，运行时都必须在不同的线程上运行。

你可以为延迟敏感型任务和长时间运行的任务分别使用一个独立的异步运行时实例。这适用于 CPU 受限型任务，但即使是在用于长时间运行任务的运行时上，你仍然不应该使用阻塞式 IO。对于 CPU 受限型任务来说，这是一个不错的解决方案，因为它是唯一一种支持将长时间运行的任务作为异步任务来运行的方案。它也很灵活（因为运行时可以配置为最适合其所运行任务的类型；实际上，要获得最佳性能，必须在运行时配置上花些心思），并且让你能够从使用 Tokio 这样成熟、设计良好的子系统中受益。你甚至可以使用两个不同的异步运行时。无论哪种情况，运行时都必须在不同的线程上运行。

另一方面，你确实需要多动些脑筋：你必须确保在正确的运行时上运行任务（这可能比听起来更难），而且任务之间的通信可能很复杂。我们接下来会讨论同步上下文与异步上下文之间的同步问题，但在多个异步运行时之间，情况可能更加棘手。每个运行时都是自己独立的小宇宙，调度器之间完全独立。Tokio 的 channel 和锁*可以*从不同的运行时中使用（甚至是非 Tokio 的运行时），但其他运行时的原语可能无法以这种方式工作。

由于每个运行时中的调度器对其他运行时一无所知（而操作系统对任何异步调度器也不知情），因此调度之间没有协调或共享的优先级，工作也无法在运行时之间被窃取。因此，任务调度可能不是最优的（尤其是当运行时没有很好地针对其工作负载进行调优时）。此外，由于所有调度都是协作式的，长时间运行的任务仍可能被剥夺资源，延迟也会受到影响。关于如何让长时间运行的任务更加协作，请参见[下一节](#yielding)。

作为一个纯粹的调度器，使用 Tokio 来做 CPU 工作可能比专用的同步工作线程池有稍高的开销。考虑到支持异步编程所需的额外工作，这并不令人意外。对于大多数用户来说，这在实践中不太可能成为问题，但如果你的代码对性能极其敏感，可能值得考虑。

对于上述任何一种解决方案，你都会有运行在不同上下文中（同步与异步，或不同的异步运行时）的任务。如果你需要在任务之间通信，就必须注意使用正确的同步与异步原语（channel、互斥锁等）组合，以及在这些原语上使用正确的（阻塞或非阻塞）方法。对于互斥锁和类似的锁，如果你需要跨 await 点持有锁或保护 IO 资源，你可能应该使用异步版本（通过使用阻塞式锁方法，它应该也能在同步上下文中使用）；如果是保护数据或锁不需要跨 await 点持有，则使用同步版本。Tokio 的异步 channel 可以通过阻塞方法在同步上下文中使用，但关于何时使用同步或异步 channel 的更多细节，请参见[这些文档](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code)。

那么，你应该如何选择上述方案呢？

- 如果你正在做阻塞式 IO，你可能应该使用 `spawn_blocking`。你不能使用第二个运行时或其他线程池（至少在你需要最佳性能的情况下）。
- 如果你有一个会永远运行的线程，你应该使用 `std::thread::spawn`，而不是使用任何类型的线程池（因为它会耗尽线程池中的一个线程）。
- 如果你正在做*大量*的 CPU 工作，那么你应该使用线程池，无论是专用的线程池还是第二个异步运行时。
- 如果你需要运行长时间运行的异步代码，那么你应该使用第二个运行时。
- 你可能会选择使用专用线程或 `spawn_blocking`，因为它简单且性能令人满意，尽管更复杂的方案才是最优的。


### Yielding

长时间运行的代码之所以是个问题，是因为它不给调度器调度其他任务的机会。异步并发是协作式的：调度器无法抢占一个任务去运行另一个任务。如果长时间运行的任务不让出控制权给调度器，调度器就无法停止它。然而，如果长时间运行的代码确实让出控制权给调度器，那么其他任务就可以被调度，任务长时间运行就不再是问题了。这可以作为一种替代方案，用于替代使用另一个线程来处理 CPU 密集型工作，或用于在自身运行时上处理 CPU 密集型工作以（可能）提升性能。

让出控制权很简单，只需调用运行时的 yield 函数。在 Tokio 中，这个函数是 [`yield_now`](https://docs.rs/tokio/latest/tokio/task/fn.yield_now.html)。请注意，它既不同于标准库的 [`yield_now`](https://doc.rust-lang.org/stable/std/thread/fn.yield_now.html)，也不同于用于从协程中让出控制权的 `yield` 关键字。如果当前 future 正在 `select` 或 `join` 中运行，调用 `yield_now` 不会让出控制权给调度器（参见[并发组合 future](concurrency-primitives.md) 一章）；这可能是也可能不是你想要的结果。

知道何时需要让出控制权要棘手一些。首先，你需要知道你的程序是否在隐式地让出控制权。这只能发生在 `.await` 处，所以如果你没有在 `await`，你就没有让出控制权。但 await 并不会自动让出控制权给调度器。只有在被 `await` 的叶子 future 处于 pending（未就绪）状态，或者调用栈中某处有显式的 `yield` 时，才会让出控制权。Tokio 和大多数异步运行时会在它们的 IO 和同步函数中这样做，但一般来说，你无法在不进行调试或查看源代码的情况下知道一个 `await` 是否会让出控制权。

一个很好的经验法则是，代码不应在超过 10-100 微秒的时间内运行而不遇到一个可能的让出点。

### 参考资料

- [Tokio docs on CPU-bound tasks and blocking code](https://docs.rs/tokio/latest/tokio/index.html#cpu-bound-tasks-and-blocking-code)
- [Blog post: What is Blocking?](https://ryhl.io/blog/async-what-is-blocking/)
- [Blog post: Using Rustlang's Async Tokio Runtime for CPU-Bound Tasks](https://thenewstack.io/using-rustlangs-async-tokio-runtime-for-cpu-bound-tasks/)
