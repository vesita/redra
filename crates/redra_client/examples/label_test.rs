//! 标签功能验证示例
//!
//! 使用 ShapeBuilder API 发送带标签的 3D 对象，测试标签显示、样式和偏移功能

use redra_client::*;
use expto::rdmp::TagStyle;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("启动 Redra 标签功能验证示例");
    println!("将发送包含标签的3D对象数据用于测试标签显示功能\n");

    // 第1帧：基础标签 - 简单文本标签
    println!("=== 发送第 1 帧：基础标签 ===");
    spawn_sphere([0.0, 0.0, 0.0], 1.0, "red")
        .id(1)
        .tag("球体对象")
        .send().await?;
    send_frame_end().await?;
    println!("第 1 帧发送完成（1个带标签的对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第2帧：自定义偏移标签
    println!("=== 发送第 2 帧：自定义偏移标签 ===");
    let offset = ExTransform {
        x: 0.0, y: 2.0, z: 0.0,
        rx: 0.0, ry: 0.0, rz: 0.0,
        sx: 1.0, sy: 1.0, sz: 1.0,
    };
    spawn_sphere([0.0, 0.0, 0.0], 1.0, "red")
        .id(1)
        .tag(Tag::new("偏移标签").with_offset(offset))
        .send().await?;
    send_frame_end().await?;
    println!("第 2 帧发送完成（标签向上偏移2个单位）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第3帧：自定义样式标签
    println!("=== 发送第 3 帧：自定义样式标签 ===");
    let style = TagStyle::default_style()
        .with_font_size(20.0)
        .with_bg_color(0.8, 0.2, 0.2, 0.9)
        .with_text_color(1.0, 1.0, 1.0, 1.0)
        .with_corner_radius(8.0);
    spawn_sphere([0.0, 0.0, 0.0], 1.0, "red")
        .id(1)
        .tag(Tag::new("红色背景大字体").with_style(style))
        .send().await?;
    send_frame_end().await?;
    println!("第 3 帧发送完成（红色背景，20号字体）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第4帧：多个对象带标签
    println!("=== 发送第 4 帧：多个对象带标签 ===");
    let green_style = TagStyle::default_style()
        .with_font_size(16.0)
        .with_bg_color(0.2, 0.6, 0.2, 0.85)
        .with_corner_radius(6.0);
    spawn_sphere([-2.0, 0.0, 0.0], 0.8, "green")
        .id(1)
        .tag(Tag::new("绿色球体").with_style(green_style))
        .send().await?;

    let blue_style = TagStyle::default_style()
        .with_font_size(14.0)
        .with_bg_color(0.2, 0.2, 0.8, 0.85)
        .with_corner_radius(4.0);
    spawn_cylinder([0.0, 0.0, 0.0], 0.5, 1.5, "blue")
        .id(2)
        .tag(Tag::new("蓝色圆柱").with_style(blue_style))
        .send().await?;

    let yellow_style = TagStyle::default_style()
        .with_font_size(18.0)
        .with_bg_color(0.8, 0.8, 0.2, 0.85)
        .with_text_color(0.0, 0.0, 0.0, 1.0)
        .with_corner_radius(10.0);
    spawn_cone([2.0, 0.0, 0.0], 0.8, 1.2, "yellow")
        .id(3)
        .tag(Tag::new("黄色圆锥").with_style(yellow_style))
        .send().await?;

    send_frame_end().await?;
    println!("第 4 帧发送完成（3个带不同样式标签的对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第5帧：复杂场景
    println!("=== 发送第 5 帧：复杂场景 ===");

    let complex_offset = ExTransform {
        x: 1.5, y: 1.5, z: 0.0,
        rx: 0.0, ry: 0.0, rz: 0.0,
        sx: 1.0, sy: 1.0, sz: 1.0,
    };
    let complex_style = TagStyle::default_style()
        .with_font_size(22.0)
        .with_bg_color(0.9, 0.1, 0.9, 0.9)
        .with_corner_radius(12.0);
    spawn_sphere([-3.0, -2.0, 0.0], 1.2, "magenta")
        .id(1)
        .tag(Tag::new("复杂标签1").with_offset(complex_offset).with_style(complex_style))
        .send().await?;

    spawn_cylinder([0.0, -2.0, 0.0], 0.6, 1.8, "cyan")
        .id(2)
        .tag("简约标签")
        .send().await?;

    let green_text_style = TagStyle::default_style()
        .with_font_size(12.0)
        .with_bg_color(0.0, 0.0, 0.0, 0.5)
        .with_text_color(0.0, 1.0, 0.0, 1.0)
        .with_corner_radius(2.0);
    spawn_cone([3.0, -2.0, 0.0], 0.7, 1.0, "white")
        .id(3)
        .tag(Tag::new("半透明黑底绿字").with_style(green_text_style))
        .send().await?;

    send_frame_end().await?;
    println!("第 5 帧发送完成（3个不同配置的标签）\n");

    println!("========================================");
    println!("标签功能验证示例执行完成！");
    println!("共发送 5 帧数据：");
    println!("  帧1: 1个对象 - 基础标签（默认样式）");
    println!("  帧2: 1个对象 - 自定义偏移标签");
    println!("  帧3: 1个对象 - 自定义样式标签（红色背景，大字体）");
    println!("  帧4: 3个对象 - 多个不同样式标签");
    println!("  帧5: 3个对象 - 复杂混合配置");
    println!("========================================");

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
