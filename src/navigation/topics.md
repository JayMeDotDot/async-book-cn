# 主题索引

## 并发与并行

- [简介](../part-guide/concurrency.md#并发与并行)
- [使用 `spawn` 并行运行异步任务](../part-guide/async-await.md#派生任务spawning-tasks)
- [使用 `join` 和 `select` 并发运行 future](../part-guide/concurrency-primitives.md)
- [混合同步与异步并发](../part-guide/io.md#其他阻塞操作)


## 正确性与安全性

- 取消
  - [简介](../part-guide/more-async-await.md#取消)
  - [在 `select` 和 `try_join` 中](../part-guide/concurrency-primitives.md)


## 性能

- 阻塞
  - [简介](../part-guide/more-async-await.md#阻塞与取消)
  - [阻塞与非阻塞 IO](../part-guide/io.md)
  - [CPU 密集型代码](../part-guide/io.md#其他阻塞操作)


## 测试

- [单元测试语法](../part-guide/more-async-await.md#单元测试)
