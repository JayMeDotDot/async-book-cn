# 并发地组合 Future

在本章中，我们将介绍更多组合 future 的方式。特别是一些让 future 并发（但非并行）执行的新方式。从表面上看，本章介绍的新函数/宏相当简单，但底层概念可能相当微妙。我们将从对 future、并发和并行的回顾开始，不过你可能也想重温一下前面比较[并发与并行](concurrency.md#concurrency-and-parallelism)的小节。

future 是一种延迟计算。可以通过 `await` 来推进一个 future，`await` 会将控制权交给运行时，使当前任务等待计算结果。如果 `a` 和 `b` 是 future，那么可以通过先 `await` 一个再 `await` 另一个的方式将它们顺序组合（即组合成一个先执行完 `a` 再执行完 `b` 的 future）：`async { a.await; b.await}`。

我们还见过使用 `spawn` 对 future 进行并行组合：`async { let a = spawn(a); let b = spawn(b); (a.await, b.await)}` 会并行运行这两个 future。注意，元组中的 `await` 并非在等待 future 本身，而是在等待 `JoinHandle`，以便在 future 完成时获取它们的结果。

本章我们将介绍两种在不使用并行的情况下并发组合 future 的方式：`join` 和 `select`/`race`。在这两种情况下，future 都通过时间切片并发运行；每个被组合的 future 轮流执行，然后轮到下一个。这一切是*在不涉及异步运行时*的情况下完成的（因此不涉及多个操作系统线程，也不存在任何并行的可能）。组合构造在本地交错执行这些 future。你可以把这些构造想象成迷你执行器，在单个异步任务内执行其组成 future。

join 和 select/race 之间的根本区别在于它们如何处理 future 完成工作：join 在所有 future 都完成时结束，而 select/race 在任一 future 完成时结束（其余的都会被取消）。两者也都有用于处理错误的变体。

这些构造（或类似概念）经常与流（stream）一起使用，我们在下文会稍有提及，但会在[流那一章](streams.md)中更详细地讨论。

如果你想要并行（或者你没有明确表示不要并行），那么派生（spawn）任务通常是比这些组合构造更简单的替代方案。派生任务通常更不容易出错、更通用，且性能更可预测。另一方面，派生本身天生就缺乏[结构化](../part-reference/structured.md)，结构化可能使生命周期和资源管理更难推理。

性能问题值得更深入地考虑一下。并发组合潜在的性能问题在于时间共享的公平性。如果你的程序中有 100 个任务，那么通常最优的资源分配方式是每个任务获得 1% 的处理器时间（或者如果所有任务都在等待，那么每个任务被唤醒的机会相同）。如果你派生了 100 个任务，这通常就是会发生的情况（大致上）。然而，如果你派生了两个任务并在其中一个任务上 join 了 99 个 future，那么调度器只会知道两个任务，一个任务会获得 50% 的时间，而那 99 个 future 每个只能获得 0.5%。

通常任务的分布不会如此偏斜，而且我们经常将 join/select 等用于超时之类的场景，在这种情况下这种行为其实是符合预期的。但仍值得思考一下，以确保你的程序具有你所期望的性能特征。


## Join（合并）

Tokio 的 [`join` 宏](https://docs.rs/tokio/latest/tokio/macro.join.html) 接受一个 future 列表，并发地运行它们直至全部完成（并以元组形式返回所有结果）。当所有 future 都完成时它才会返回。这些 future 始终在同一个线程上执行（是并发而非并行）。

下面是一个简单的例子：

```rust,norun
async fn main() {
  let (result_1, result_2) = join!(do_a_thing(), do_a_thing());
  // Use `result_1` and `result_2`.
}
```

在这里，两次 `do_a_thing` 的执行是并发进行的，当两者都完成时结果就绪。注意，我们不需要用 `await` 来获取结果。`join!` 会隐式地 await 其 future 并产出一个值，它本身不会创建 future。但你仍需在异步上下文中使用它（例如在异步函数内）。

虽然在上面的例子中看不出，`join!` 接受的是求值为 future 的表达式[^into]。`join` 不会在其函数体中创建异步上下文，你不应该对传给 `join` 的 future 进行 `await`（否则它们会在被 join 的 future 之前就被求值执行）。

由于所有 future 都在同一个线程上执行，如果某个 future 阻塞了线程，那么其他 future 都无法推进。如果使用了互斥锁或其他锁，当某个 future 等待另一个 future 持有的锁时，很容易导致死锁。

[`join`](https://docs.rs/tokio/latest/tokio/macro.join.html) 并不关心 future 的结果。特别是，如果某个 future 被取消或返回错误，不会影响其他 future——它们会继续执行。如果你想要"快速失败"的行为，请使用 [`try_join`](https://docs.rs/tokio/latest/tokio/macro.try_join.html)。`try_join` 的工作方式与 `join` 类似，但如果任一 future 返回 `Err`，则所有其他 future 都会被取消，`try_join` 会立即返回该错误。

回到之前关于 [async/await](async-await.md) 的章节，我们曾用"join"这个词来谈论合并派生出的任务。顾名思义，合并 future 与合并任务是相关的：join 意味着我们并发地执行多个 future，并等待结果后再继续。语法上有所不同：使用 `JoinHandle` 还是 `join` 宏，但思路是相似的。关键区别在于：合并任务时，任务是并发且并行执行的，而使用 `join!` 时，future 是并发但非并行执行的。此外，派生的任务由运行时的调度器调度，而使用 `join!` 时，future 是在本地"调度"的（在同一个任务上、在宏执行的时间范围内）。另一个区别是：如果派生的任务 panic，panic 会被运行时捕获；而如果 `join` 中的某个 future panic，那么整个任务都会 panic。


### 替代方案

并发地运行 future 并收集它们的结果是一个常见需求。除非你有充分理由不这么做（即你明确不想要并行，即便如此你也可能更倾向于使用 [`spawn_local`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_local.html)），否则你大概应该使用 `spawn` 和 `JoinHandle`。[`JoinSet`](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) 抽象可以像 `join!` 那样来管理这些派生出的任务。

大多数运行时（以及 [futures.rs](https://docs.rs/futures/latest/futures/macro.join.html)）都有与 Tokio 的 `join` 宏等价的实现，且行为大体相同。也有一些 `join` 函数，它们与宏类似但灵活性稍差。例如，futures.rs 提供了用于合并两个 future 的 [`join`](https://docs.rs/futures/latest/futures/future/fn.join.html)、用于合并相应数量 future 的 [`join3`](https://docs.rs/futures/latest/futures/future/fn.join3.html)、[`join4`](https://docs.rs/futures/latest/futures/future/fn.join4.html) 和 [`join5`](https://docs.rs/futures/latest/futures/future/fn.join5.html)，以及用于合并一组 future 的 [join_all](https://docs.rs/futures/latest/futures/future/fn.join_all.html)（它们各自也都有 `try_` 变体）。

[Futures-concurrency](https://docs.rs/futures-concurrency/latest) 也提供了 join（以及 try_join）功能。在 futures-concurrency 的风格中，这些操作是作用于 future 组（如元组、`Vec` 或数组）上的 trait 方法。例如，要合并两个 future，你可以写 `(fut1, fut2).join().await`（注意这里的 `await` 是显式的）。

如果你想要合并的 future 集合是动态变化的（例如，随着网络输入的到来不断创建新的 future），或者你想在每个 future 完成时就获取结果而不是等所有 future 都完成，那么你就需要使用流以及 [`FuturesUnordered`](https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html) 或 [`FuturesOrdered`](https://docs.rs/futures/latest/futures/stream/struct.FuturesOrdered.html) 功能。我们会在[流](streams.md)一章中介绍这些。


[^into]: 这些表达式必须具有实现了 `IntoFuture` 的类型。表达式会被宏求值并转换为 future。也就是说，它们实际上不必求值为 future，而是可以转换为 future 的东西，不过这是一个相当细微的区分。表达式本身会在任何结果 future 执行之前被顺序求值。


## Race/select（竞速/选择）

与 join future 相对的是对 future 进行 race（也称为 select）。使用 race/select 时，future 是并发执行的，但我们不是等待所有 future 完成，而是只等待第一个完成，然后取消其余的。虽然这听起来与 join 类似，但它要有趣得多（有时也更容易出错），因为现在我们必须考虑取消（cancellation）的问题。

下面是一个使用 Tokio 的 [`select`](https://docs.rs/tokio/latest/tokio/macro.select.html) 宏的例子：

```rust,norun
async fn main() {
  select! {
    result = do_a_thing() => {
      println!("computation completed and returned {result}");
    }
    _ = timeout() => {
      println!("computation timed-out");
    }
  }
}
```

你会注意到，情况已经比 `join` 宏更有意思了，因为我们在 `select` 宏内部处理 future 的结果。它看起来有点像 `match` 表达式，但在 `select` 中，所有分支是并发运行的，最先完成的分支的函数体会带着其结果被执行（其他分支不会被执行，其 future 会通过 `drop` 被取消）。在上面的例子中，`do_a_thing` 和 `timeout` 并发执行，最先完成的那个会执行其代码块（即只会运行一个 `println`），另一个 future 会被取消。与 `join` 宏一样，对 future 的 await 是隐式的。

Tokio 的 `select` 宏支持许多特性：

- pattern matching（模式匹配）：每个分支中 `=` 左侧的语法可以是一个模式，只有当 future 的结果匹配该模式时才会执行代码块。如果模式不匹配，则该 future 不再被轮询（但其他 future 仍会被轮询）。这对于可选返回值的 future 很有用，例如 `Some(x) = do_a_thing() => { ... }`。
- `if` guards（if 守卫）：每个分支可以有一个 `if` 守卫。当 `select` 宏运行时，在将每个表达式求值为 future 之后，会求值 `if` 守卫，只有当守卫为真时才会轮询该 future。例如，`x = = do_a_thing() if false => { ... }` 永远不会被轮询。注意，`if` 守卫不会在轮询期间被重新求值，只在宏初始化时求值一次。
- 所有 future 都已停止且没有任何代码块被执行时，会执行此分支。如果这种情况发生时没有 `else` 分支，`select` 会 panic。

`select!` 宏的值是所执行分支的值（就像 `match` 一样），因此所有分支必须具有相同的类型。例如，如果我们想在上面的例子中在 `select` 之外使用结果，我们可以这样写：

```rust,norun
async fn main() {
  let result = select! {
    result = do_a_thing() => {
      Some(result)
    }
    _ = timeout() => {
      None
    }
  };

  // Use `result`
}
```

与 `join!` 一样，`select!` 不会以任何特殊方式处理 `Result`（除了前面提到的模式匹配），如果某个分支以错误完成，那么所有其他分支都会被取消，该错误将作为 select 的结果（与分支成功完成时的处理方式相同）。

`select` 宏本质上使用了取消，因此如果你想避免程序中的取消，就必须避免使用 `select!`。事实上，`select` 通常是异步程序中取消的主要来源。正如在[其他地方](../part-reference/cancellation.md)所讨论的，取消有许多微妙的问题，可能导致 bug。特别要注意，`select` 通过简单地丢弃（drop）future 来取消它们。这不会通知被丢弃的 future，也不会触发任何取消令牌等。

`select!` 常被用在循环中来处理流或其他 future 序列。这增加了一层额外的复杂性和出 bug 的机会。在简单情况下，如果在循环的每次迭代中创建一个新的、独立的 future，事情并不会复杂太多。然而，这很少是我们需要的。通常我们希望在迭代之间保留一些状态。在循环中使用 `select` 处理流是很常见的，循环的每次迭代处理流中的一个结果。例如：

```rust,norun
async fn main() {
  let mut stream = ...;

  loop {
    select! {
      result = stream.next() => {
        match result {
          Some(x) => println!("received: {x}"),
          None => break,
        }
      }
      _ = timeout() => {
        println!("time out!");
        break;
      }
    }
  }
}
```

在这个例子中，我们从 `stream` 中读取值并打印，直到没有剩余数据或等待结果超时。在超时的情况下，流中剩余的数据会如何处理取决于流的实现（可能会丢失！或者被重复处理！）。这是一个例子，说明在面对取消时的行为可能很重要（也很棘手）。

我们可能希望在多次迭代之间复用一个 future，而不仅仅是流。例如，我们可能想要与一个超时 future 竞速，而该超时适用于所有迭代，而不是为每次迭代应用一个新的超时。这可以通过在循环外部创建 future 并引用它来实现：

```rust,norun
async fn main() {
  let mut stream = ...;
  let mut timeout = timeout();

  loop {
    select! {
      result = stream.next() => {
        match result {
          Some(x) => println!("received: {x}"),
          None => break,
        }
      }
      // 创建对 `timeout` 的引用，而不是移动它。
      _ = &mut timeout => {
        println!("time out!");
        break;
      }
    }
  }
}
```

在循环中使用 `select!` 且 future 或流是在 `select!` 外部创建时，有几个重要的细节。这些是 `select` 工作机制的必然结果，因此我将以上一个例子中的 `timeout` 为例，逐步剖析 `select` 的细节来介绍它们。

- `timeout` 在循环外部创建，并初始化为某个倒计时时间。
- 在循环的每次迭代中，`select` 创建对 `timeout` 的引用，但不会改变其状态。
- 随着 `select` 的执行，它会轮询 `timeout`，当还有剩余时间时返回 `Pending`，时间耗尽时返回 `Ready`，此时其代码块会被执行。

在上面的例子中，当 `timeout` 就绪时，我们 `break` 跳出循环。但如果我们不这样做呢？那样的话，`select` 会再次轮询 `timeout`，而 `Future` 的[文档](https://doc.rust-lang.org/std/future/trait.Future.html#tymethod.poll)说这种情况不应该发生！`select` 也无能为力，它没有任何状态（在迭代之间）来决定是否应该轮询 `timeout`。根据 `timeout` 的实现方式，这可能导致 panic、逻辑错误或某种崩溃。

你可以通过几种方式来防止这类 bug：

- 使用 [fused](futures.md#fusing)（融合）的 [future](https://docs.rs/futures/latest/futures/future/trait.FutureExt.html#method.fuse) 或 [stream](https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.fuse)，使得重复轮询是安全的。
- 确保你的代码结构使得 future 永远不会被重复轮询，例如通过跳出循环（如前一个例子所示），或通过使用 `if` 守卫。

现在，让我们考虑 `&mut timeout` 的类型。假设 `timeout()` 返回一个实现了 `Future` 的类型，它可能是异步函数产生的匿名类型，也可能是像 `Timeout` 这样的命名类型。我们假设是后者，因为这会让例子更容易理解（但逻辑在两种情况下都适用）。既然 `Timeout` 实现了 `Future`，那么 `&mut Timeout` 会实现 `Future` 吗？不一定！有一个[ blanket impl](https://doc.rust-lang.org/std/future/trait.Future.html#impl-Future-for-%26mut+F) 使其成立，但前提是 `Timeout` 实现了 `Unpin`。并非所有 future 都满足这一条件，因此编写像上一个例子那样的代码时经常会遇到类型错误。不过这种错误很容易修复，只需使用 `pin` 宏，例如 `let mut timeout = pin!(timeout());`

在循环中使用 select 引发的取消往往会导致各种难以察觉的 bug。这些 bug 通常发生在 future 包含某些涉及数据的状态但又不包含数据本身的情况下。当 future 因取消而被 drop 时，那些状态会丢失，但底层数据并未被更新。这可能导致数据丢失或被多次处理。


### 替代方案

Futures.rs 有自己的 [`select` 宏](https://docs.rs/futures/latest/futures/macro.select.html)，futures-concurrency 有一个 [Race trait](https://docs.rs/futures-concurrency/latest/futures_concurrency/future/trait.Race.html)，它们都是 Tokio 的 `select` 宏的替代方案。这两者都具有相同的核心语义：并发地竞速多个 future，处理第一个完成的结果并取消其余的，但它们的语法不同，细节上也有所差异。

Futures.rs 的 `select` 在表面上与 Tokio 的相似；总结一下差异，在 futures.rs 版本中：

- Future 必须始终是融合的（由类型检查强制要求）。
- `select` 有 `default` 和 `complete` 分支，而不是 `else` 分支。
- `select` 不支持 `if` 守卫。

Futures-concurrency 的 `Race` 有着非常不同的语法，类似于它版本的 `join`，例如 `(future_a, future_b).race().await`（它适用于 `Vec` 和数组以及元组）。其语法不如宏灵活，但与大多数异步代码配合得很好。注意，如果你在循环中使用 `race`，仍然可能遇到与 `select` 相同的问题。

与 `join` 一样，派生任务并让它们并行执行通常是使用 `select` 的一个好替代方案。但是，在第一个任务完成后取消剩余任务需要一些额外的工作。这可以通过通道或取消令牌来实现。无论哪种方式，取消都需要被取消的任务采取一些行动，这意味着该任务可以进行一些清理或其他优雅关闭。

`select` 的一个常见用途（尤其是在循环中）是处理流。有一些流组合器方法可以替代 `select` 的某些用法。例如，futures-concurrency 中的 [`merge`](https://docs.rs/futures-concurrency/latest/futures_concurrency/stream/trait.Merge.html) 是将多个流合并在一起的好替代方案。


## 结语

在本节中，我们讨论了两种并发运行一组 future 的方式。Join future 意味着等待它们全部完成；select（又称 race）future 意味着等待第一个完成。与派生任务相比，这些组合方式不使用并行。

`join` 和 `select` 都作用于事先已知的 future 集合（通常是在编写程序时就确定的，而非运行时）。有时，要被组合的 future 并非事先已知的——future 必须在执行过程中被添加到被组合的 future 集合中。为此我们需要[流](streams.md)，它有自己的组合操作。

值得重申的是，虽然这些组合操作符功能强大且表现力强，但使用任务和派生通常更容易也更合适：并行通常是符合预期的，你不太可能遇到取消或阻塞相关的 bug，而且资源分配通常更公平（或至少更简单）且更可预测。
