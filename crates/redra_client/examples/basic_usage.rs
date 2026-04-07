use redra_client::{ShapeConfig, send_point, send_segment, send_cube, send_sphere};
use redra_proto::proto::shape::Color;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("=== Redra Client 示例程序 ===\n");

    // 1. 连接到服务器（请确保服务器正在运行）
    let server_addr = "127.0.0.1:8080";
    println!("正在连接到服务器: {}", server_addr);
    
    match redra_client::init_global_client(server_addr).await {
        Ok(_) => println!("✓ 连接成功\n"),
        Err(e) => {
            eprintln!("✗ 连接失败: {}", e);
            eprintln!("提示: 请确保服务器正在运行在 {}", server_addr);
            return Err(e);
        }
    }

    // 2. 发送简单的点（使用默认配置）
    println!("1. 发送简单的点...");
    send_point(0.0, 0.0, 0.0).await?;
    send_point(1.0, 0.0, 0.0).await?;
    send_point(0.0, 1.0, 0.0).await?;
    println!("   ✓ 已发送 3 个点\n");

    // 3. 发送带配置的点（自定义颜色和名称）
    println!("2. 发送带配置的点...");
    let red_config = ShapeConfig::new()
        .with_color(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 })
        .with_name("red_origin")
        .with_tag("important");
    
    redra_client::send_point_with_config(2.0, 0.0, 0.0, red_config).await?;
    
    let green_config = ShapeConfig::new()
        .with_color(Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 })
        .with_name("green_point");
    
    redra_client::send_point_with_config(2.0, 1.0, 0.0, green_config).await?;
    println!("   ✓ 已发送 2 个带颜色的点\n");

    // 4. 发送线段
    println!("3. 发送线段...");
    send_segment([0.0, 0.0, 0.0], [5.0, 5.0, 0.0]).await?;
    
    let blue_line_config = ShapeConfig::new()
        .with_color(Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 })
        .with_name("blue_diagonal");
    
    redra_client::send_segment_with_config(
        [0.0, 0.0, 0.0],
        [5.0, 0.0, 5.0],
        blue_line_config
    ).await?;
    println!("   ✓ 已发送 2 条线段\n");

    // 5. 发送立方体
    println!("4. 发送立方体...");
    send_cube(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).await?;
    
    let yellow_cube_config = ShapeConfig::new()
        .with_color(Color { r: 1.0, g: 1.0, b: 0.0, a: 0.7 })
        .with_name("yellow_box")
        .with_tag("geometry");
    
    redra_client::send_cube_with_config(
        3.0, 0.0, 0.0,
        2.0, 1.5, 1.0,
        yellow_cube_config
    ).await?;
    println!("   ✓ 已发送 2 个立方体\n");

    // 6. 发送球体
    println!("5. 发送球体...");
    send_sphere(0.0, 3.0, 0.0, 0.5).await?;
    
    let purple_sphere_config = ShapeConfig::new()
        .with_color(Color { r: 0.5, g: 0.0, b: 1.0, a: 0.8 })
        .with_name("purple_ball");
    
    redra_client::send_sphere_with_config(
        3.0, 3.0, 0.0,
        1.0,
        purple_sphere_config
    ).await?;
    println!("   ✓ 已发送 2 个球体\n");

    // 7. 批量发送点
    println!("6. 批量发送点...");
    let points = vec![
        [0.0, 5.0, 0.0],
        [1.0, 5.0, 0.0],
        [2.0, 5.0, 0.0],
        [3.0, 5.0, 0.0],
        [4.0, 5.0, 0.0],
    ];
    
    let batch_config = ShapeConfig::new()
        .with_color(Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 })
        .with_name("batch_points");
    
    redra_client::send_points_with_config(&points, batch_config).await?;
    println!("   ✓ 已批量发送 {} 个点\n", points.len());

    // 8. 切换到下一个数据集
    println!("7. 切换到下一个数据集...");
    redra_client::next_set().await?;
    println!("   ✓ 数据集已切换\n");

    // 9. 在新数据集中发送形状
    println!("8. 在新数据集中发送形状...");
    send_point(10.0, 0.0, 0.0).await?;
    send_cube(10.0, 0.0, 0.0, 1.0, 1.0, 1.0).await?;
    println!("   ✓ 已在新数据集中发送形状\n");

    println!("=== 所有示例完成！===");
    println!("\n提示: 查看服务器端以确认接收到的数据");
    
    // 保持程序运行一段时间，让消息完全发送
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    Ok(())
}
