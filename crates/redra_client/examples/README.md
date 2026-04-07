# Redra Client 示例

本目录包含 `redra_client` 的使用示例，帮助你快速上手。

## 前置要求

在运行示例之前，请确保：

1. **Redra 服务器正在运行**
   ```bash
   # 在另一个终端中启动服务器
   cargo run -p redra_server
   ```

2. **设置日志级别**（可选，用于查看详细日志）
   ```bash
   export RUST_LOG=info
   ```

## 示例列表

### 1. simple_test.rs - 简单测试

最简单的使用示例，演示基本的连接和形状发送。

**运行方式：**
```bash
cargo run --example simple_test
```

**功能：**
- 连接到服务器
- 发送 2 个点和 1 个立方体
- 快速验证基本功能

### 2. basic_usage.rs - 完整功能演示

展示 `redra_client` 的所有主要功能和 API 用法。

**运行方式：**
```bash
cargo run --example basic_usage
```

**功能：**
- ✓ 基本形状发送（点、线段、立方体、球体）
- ✓ 使用 `ShapeConfig` 配置颜色、名称和标签
- ✓ 批量发送形状
- ✓ 数据集切换（`next_set()`）
- ✓ 自定义配置示例

## API 使用说明

### 基本形状发送

```rust
// 发送点
send_point(x, y, z).await?;

// 发送线段
send_segment([x1, y1, z1], [x2, y2, z2]).await?;

// 发送立方体
send_cube(x, y, z, width, height, depth).await?;

// 发送球体
send_sphere(x, y, z, radius).await?;
```

### 带配置的形状发送

```rust
use redra_client::ShapeConfig;
use redra_proto::proto::shape::Color;

let config = ShapeConfig::new()
    .with_color(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }) // 红色
    .with_name("my_shape")
    .with_tag("important");

send_point_with_config(x, y, z, config).await?;
```

### 批量发送

```rust
let points = vec![
    [0.0, 0.0, 0.0],
    [1.0, 1.0, 1.0],
    [2.0, 2.0, 2.0],
];

send_points(&points).await?;
```

### 数据集管理

```rust
// 切换到下一个数据集
next_set().await?;
```

## 常见问题

### Q: 连接失败怎么办？

**A:** 确保服务器正在运行并且地址正确：
```bash
# 检查服务器是否在运行
netstat -tlnp | grep 8080

# 或尝试 ping 服务器
ping 127.0.0.1
```

### Q: 看不到发送的形状？

**A:** 
1. 检查服务器日志确认数据已接收
2. 确保使用了正确的坐标系
3. 检查查看器是否正确渲染了形状

### Q: 如何调试？

**A:** 设置详细的日志级别：
```bash
RUST_LOG=debug cargo run --example basic_usage
```

## 下一步

- 查看 [`redra_client`](../src/lib.rs) 的文档了解完整的 API 参考
- 探索 [`redra_proto`](../../redra_proto/) 了解底层协议
- 创建你自己的示例来测试特定功能
