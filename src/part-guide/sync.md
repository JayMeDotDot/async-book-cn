# 通道、锁与同步

关于同步原语与运行时相关性的说明

为什么我们需要异步原语，而不是直接使用同步原语

## 通道

- 基本上与标准库的通道相同，但需要 await
- 用于任务之间通信 (同一线程或不同线程均可)（用于任务之间通信，同一线程或不同线程均可）
- one shot（单次发送通道）
- mpsc（多生产者、单消费者通道）
- 其他类型的通道
- 有界通道与无界通道

## 锁

  - async Mutex（异步互斥锁）
    - 参见 std::Mutex —— 可以跨 await 点持有（在 guard 中借用互斥锁，guard 是 Send 的，调度器感知？还是仅仅因为 lock 是异步的？），lock 是异步的（等待锁可用时不会阻塞线程）
      - 甚至有一个 clippy lint 用于检测跨 await 持有 guard 的情况 (https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock)
    - 因为可以跨 await 持有，所以开销更大
      - 尽量使用 std::Mutex
        - 可以使用 try_lock，或者预期该互斥锁不会存在竞争
    - 在让出执行权时锁并不会被神奇地释放（这恰恰是锁的意义所在！）
    - 由于跨 await 持有互斥锁而导致死锁
      - 任务死锁了，但其他任务仍然可以推进，所以在进程统计/工具/操作系统中可能看起来不像死锁
      - 常见建议：限制作用域、尽量减少锁、对锁排序、优先使用替代方案
    - 没有互斥锁中毒机制
    - lock_owned
    - blocking_lock
      - 不能在异步环境中使用
    - 同样适用于其他锁（上述内容是否应该移到专门讨论 mutex 之前？也许是）
  - RWLock（读写锁）
  - Semaphore（信号量）
  - yielding（让出执行权）

## 其他同步原语

- notify, barrier（通知、屏障）
- OnceCell（单次初始化单元）
- atomics（原子操作）
