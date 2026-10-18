# 取消与取消安全性

内部取消 vs 外部取消
线程 vs future
  drop 即取消
  只能在 await 点取消
  有用的特性
  但取消仍然有些突然，令人意外
其它取消机制
  abort（强制中止）
  取消令牌（cancellation tokens）

## 取消安全性

不是内存安全问题，也不是竞态条件
  而是数据丢失或其它逻辑错误
不同的定义/名称
  tokio 的定义
  通用定义/停机安全性（halt safety）
  应用"可复制 future"的思路
简单的数据丢失
恢复执行
在循环中使用 select 或类似结构的问题
状态被拆分在 future 与上下文（context）之间，是根本原因
