# 定时器与信号处理

## 时间与定时器

- 运行时集成，不要使用 thread::sleep 等

- 标准库的 Instant 和 Duration

- sleep

- interval

- timeout
  - 特殊 Future 与 select/race 的对比

## 信号处理

- 什么是信号处理，以及为什么它是一个异步问题？

- 高度依赖操作系统

- 参见 Tokio 文档
