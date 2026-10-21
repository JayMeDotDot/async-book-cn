# 同时执行多个 Future

到目前为止，我们主要使用 `.await` 来执行 future，它会阻塞当前任务，直到某个特定的 `Future` 完成。然而，真实的异步应用往往需要并发地执行多个不同的操作。

在本章中，我们将介绍一些同时执行多个异步操作的方法：

- `join!`：等待所有 future 完成
- `select!`：等待多个 future 中的某一个完成
- 生成任务：创建一个顶层任务，在后台自动运行某个 future 直到完成
- `FuturesUnordered`：一组 future，会产出每个子 future 的结果
