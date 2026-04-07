use redra_client::{send_point, send_cube};

/// 最简单的使用示例 - 快速测试连接和基本功能
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("正在连接到服务器...");
    
    // 初始化客户端
    redra_client::init_global_client("127.0.0.1:8080").await?;
    println!("✓ 连接成功\n");

    // 发送一些基本形状
    println!("发送测试形状...");
    send_point(0.0, 0.0, 0.0).await?;
    send_point(1.0, 1.0, 1.0).await?;
    send_cube(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).await?;
    
    println!("✓ 完成！");
    
    Ok(())
}
