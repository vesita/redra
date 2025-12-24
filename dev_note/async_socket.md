# 开发笔记

## 问题：首次连接握手失败

### 问题描述

在 Redra 项目中，首次建立网络连接时，[linker](../src/net/listener.rs#L23-L23) 和 [forwarder](../src/net/listener.rs#L24-L24) 任务无法完成握手过程，导致数据传输失败。虽然后续连接可以工作，但首次连接总是失败。

### 问题症状

- 日志显示 [forwarder](../src/net/listener.rs#L24-L24) 等待握手信号超时
- [linker](../src/net/listener.rs#L23-L23) 任务没有输出预期的调试日志
- 握手过程失败，无法进入数据传输阶段

### 问题原因

根本原因是使用了阻塞的 `socket2::Socket` 而不是异步的 socket。在异步环境中使用阻塞的 socket 操作会：

1. 阻塞整个异步运行时，影响其他任务的调度
2. [linker](../src/net/listener.rs#L23-L23) 任务在尝试读取 socket 数据时被阻塞，导致无法执行到握手阶段
3. 因此 [forwarder](../src/net/listener.rs#L24-L24) 无法收到握手消息，导致握手失败

### 解决方案

将底层的 socket 操作从 `socket2::Socket` 替换为 `tokio::net::TcpStream` 和 `tokio::net::TcpListener`：

1. 修改 [linker.rs](../src/net/linker.rs) 中的 [socket](../src/net/listener.rs#L33-L33) 字段类型
2. 使用 `tokio::io::AsyncReadExt` 的异步读取方法
3. 修改 [listener.rs](../src/net/listener.rs) 使用 `TokioTcpListener`
4. 确保整个网络操作链都是异步的

### 代码修改

- [linker.rs](../src/net/linker.rs): 使用 `tokio::net::TcpStream` 和异步读取
- [listener.rs](../src/net/listener.rs): 使用 `tokio::net::TcpListener`

### 结果

修改后握手过程和数据传输都能正常工作，每个连接的 [linker](../src/net/listener.rs#L23-L23) 和 [forwarder](../src/net/listener.rs#L24-L24) 任务可以正确地并行运行。

### 关键点

在 Rust 异步编程中，必须确保所有 I/O 操作都是异步的，使用阻塞操作会破坏异步运行时的正常调度。
