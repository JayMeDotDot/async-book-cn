# 运行时与运行时相关的问题

## 运行异步代码

- 显式启动 vs async main
- tokio 上下文的概念
- block_on
- 代码中体现的运行时（Runtime、Handle）
- 运行时关闭

## 线程与任务

- 默认采用工作窃取（work stealing）的多线程模式
  - 重新审视 Send + 'static 约束
- yield
- spawn-local
- spawn-blocking（回顾）、block-in-place
- tokio 特有的关于向其他线程让出执行权、本地队列与全局队列等内容

## 配置选项

- 线程池大小
- 单线程、每核心一个线程等

## 其他运行时

- 为什么你会想要使用不同的运行时或自己实现一个
- 高层设计中存在哪些差异
- 前向引用进阶章节
