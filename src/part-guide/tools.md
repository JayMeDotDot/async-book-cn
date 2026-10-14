# 异步编程工具

- 为什么我们需要专门的异步工具
- 是否还有其他工具需要介绍
  - loom

## 监控

- [Tokio console](https://github.com/tokio-rs/console)

## 追踪与日志

- 异步追踪存在的问题
- tracing crate (https://github.com/tokio-rs/tracing)

## 调试

- 理解异步回溯（RUST_BACKTRACE 与调试器中的回溯）
- 调试异步代码的技巧
- 使用 Tokio console 进行调试
- 调试器支持（WinDbg？）

## 性能分析

- 异步如何破坏火焰图
- 如何对异步 IO 进行性能分析
- 深入了解运行时
  - Tokio metrics
