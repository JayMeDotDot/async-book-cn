# Futures

在前面的章节中，我们已经多次讨论过 future；它是 Rust 异步编程故事中的关键组成部分！在本章中，我们将深入探讨 future 究竟是什么、它是如何工作的，以及一些可以直接用于处理 future 的库。

## `Future` 与 `IntoFuture` trait

- Future
  - Output 关联类型
  - 这里不展开细节，轮询（polling）在下一节讨论，可参考关于 Pin、执行器/唤醒器（executors/wakers）的进阶章节
- IntoFuture
  - 用法——通用场景、在 await 中、异步构建器模式（async builder pattern）（以及其使用的优缺点）
- 装箱（boxing）future，`Box<dyn Future>`，以及它过去为何既常见又必要、而如今大多不再需要（除递归等少数情况外）

## 轮询（Polling）

- 它是什么、由谁执行，Poll 类型
  - ready 是最终状态
- 它与 await 的关系
- drop = 取消（cancel）
  - 对 future 以及由此对 task 均适用
  - 对异步编程整体的含义
  - 参见关于取消安全性（cancellation safety）的章节

### 熔合（Fusing）

## futures-rs crate

- 历史与目的
  - 参见 stream 章节
  - 用于编写执行器或其他底层 future 相关内容的辅助工具
    - 固定（pinning）与装箱（boxing）
  - 执行器作为一种部分运行时（参见参考部分中的替代运行时）
- TryFuture
- 便利 future：pending、ready、ok/err 等
- FutureExt 上的组合子（combinator）函数
- Tokio 相关内容的替代方案
  - 函数
  - IO trait

## futures-concurrency crate

https://docs.rs/futures-concurrency/latest/futures_concurrency/
