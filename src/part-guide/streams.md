# 异步迭代器（曾用名：流）

- 流作为一种异步迭代器或多个 future
- WIP
  - 当前状态
  - futures 与 Tokio 的 Stream trait
  - nightly trait
- 像同步迭代器一样惰性求值
- 固定（pinning）与流（前向引用至 pinning 章节）
- 融合流（fused streams）

## 消费异步迭代器

- while let with async next
- for_each、for_each_concurrent
- collect
- into_future、buffered

## 流组合子（combinators）

- 接受一个 future 而非闭包
- 一些组合子示例
- 无序（unordered）变体
- StreamGroup

### 使用流进行 join/select/race

- 在循环中使用 select 的隐患
- 融合（fusing）
- 与普通 future 的区别
- 它们的替代方案
  - Stream::merge 等

## 实现异步迭代器

- 实现 trait
- 实践细节与工具函数
- async_iter stream 宏

## Sink（接收端）

- https://docs.rs/futures/latest/futures/sink/index.html

## 后续工作

- 当前状态
  - https://rust-lang.github.io/rfcs/2996-async-iterator.html
- async next 与 poll 的对比
- 异步迭代语法
- （异步）生成器
- 借贷式迭代器（lending iterators）
