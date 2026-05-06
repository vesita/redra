//! ShapeBuilder API 综合测试
//!
//! 测试所有形状 + 位置 + 材质 + 标签 + 样式

use redra_client::*;
use expto::rdmp::TagStyle;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("ShapeBuilder API 综合测试\n");

    // ── 第1帧：6种形状，一行排开 ──────────────────────
    println!("=== 第 1 帧：基础形状 ===");

    spawn_sphere([-5.0, 0.0, 0.0], 0.8, "red")
        .id(1).tag("Sphere").send().await?;

    spawn_cylinder([-3.0, 0.0, 0.0], 0.4, 1.5, "green")
        .id(2).tag("Cylinder").send().await?;

    spawn_cone([-1.0, 0.0, 0.0], 0.5, 1.2, "blue")
        .id(3).tag("Cone").send().await?;

    spawn_point([1.0, 0.0, 0.0], "yellow")
        .id(4).tag("Point").send().await?;

    spawn_line([2.5, -0.5, 0.0], [4.0, 0.8, 0.0], "cyan")
        .id(5).tag("Line").send().await?;

    spawn_cube(vec![
        (5.0, -0.5, -0.5), (6.0, -0.5, -0.5),
        (6.0,  0.5, -0.5), (5.0,  0.5, -0.5),
        (5.0, -0.5,  0.5), (6.0, -0.5,  0.5),
        (6.0,  0.5,  0.5), (5.0,  0.5,  0.5),
    ], "magenta").id(6).tag("Cube").send().await?;

    send_frame_end().await?;
    println!("  6 个基础形状\n");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // ── 第2帧：标签样式展示 ──────────────────────────
    println!("=== 第 2 帧：标签样式 ===");

    let red_tag = TagStyle::default_style()
        .with_font_size(18.0)
        .with_bg_color(0.8, 0.1, 0.1, 0.9)
        .with_corner_radius(8.0);
    spawn_sphere([-3.0, 0.0, 0.0], 0.8, "red")
        .id(1).tag(Tag::new("红底白字").with_style(red_tag)).send().await?;

    let green_tag = TagStyle::default_style()
        .with_font_size(14.0)
        .with_bg_color(0.1, 0.6, 0.1, 0.85)
        .with_text_color(1.0, 1.0, 0.0, 1.0)
        .with_corner_radius(4.0);
    spawn_sphere([0.0, 0.0, 0.0], 0.8, "green")
        .id(2).tag(Tag::new("绿底黄字").with_style(green_tag)).send().await?;

    let offset = ExTransform {
        x: 0.0, y: 2.0, z: 0.0,
        rx: 0.0, ry: 0.0, rz: 0.0,
        sx: 1.0, sy: 1.0, sz: 1.0,
    };
    spawn_sphere([3.0, 0.0, 0.0], 0.8, "blue")
        .id(3).tag(Tag::new("向上偏移").with_offset(offset)).send().await?;

    send_frame_end().await?;
    println!("  3 种标签样式\n");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // ── 第3帧：变换（缩放 + 旋转） ───────────────────
    println!("=== 第 3 帧：变换 ===");

    spawn_sphere([-3.0, 0.0, 0.0], 0.5, "red")
        .id(1).scale_uniform(1.0).tag("1x").send().await?;

    spawn_sphere([0.0, 0.0, 0.0], 0.5, "green")
        .id(2).scale_uniform(2.0).tag("2x").send().await?;

    spawn_sphere([3.5, 0.0, 0.0], 0.5, "blue")
        .id(3).scale(1.0, 2.0, 0.5).tag("非等比").send().await?;

    spawn_cylinder([-3.0, 2.5, 0.0], 0.3, 1.0, "yellow")
        .id(4).rotation_deg(0.0, 0.0, 45.0).tag("旋转45°").send().await?;

    spawn_cylinder([0.0, 2.5, 0.0], 0.3, 1.0, "cyan")
        .id(5).rotation_deg(0.0, 0.0, 90.0).tag("旋转90°").send().await?;

    send_frame_end().await?;
    println!("  缩放 + 旋转变换\n");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // ── 第4帧：材质效果 ──────────────────────────────
    println!("=== 第 4 帧：材质 ===");

    for (i, mat) in ["red", "green", "blue", "yellow", "cyan", "magenta", "white"].iter().enumerate() {
        let x = -6.0 + i as f32 * 2.0;
        spawn_sphere([x, 0.0, 0.0], 0.6, *mat)
            .id(i as u64 + 1).tag(*mat).send().await?;
    }

    for (i, mat) in ["metal", "glass", "glow", "matte", "plastic", "wood"].iter().enumerate() {
        let x = -5.0 + i as f32 * 2.0;
        spawn_sphere([x, 2.5, 0.0], 0.5, *mat)
            .id(i as u64 + 10).tag(*mat).send().await?;
    }

    send_frame_end().await?;
    println!("  基础色 + 效果材质\n");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // ── 第5帧：混合场景 ──────────────────────────────
    println!("=== 第 5 帧：混合场景 ===");

    // 中心大球
    spawn_sphere([0.0, 0.0, 0.0], 1.5, "metal")
        .id(1).tag("中心").send().await?;

    // 环绕的小球
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_4;
        let x = angle.cos() * 3.0;
        let z = angle.sin() * 3.0;
        let colors = ["red", "green", "blue", "yellow", "cyan", "magenta", "white", "metal"];
        spawn_sphere([x, 0.0, z], 0.4, colors[i])
            .id(i as u64 + 10)
            .tag(format!("环绕 #{}", i))
            .send().await?;
    }

    // 连接线
    spawn_line([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], "glow")
        .id(100).send().await?;

    send_frame_end().await?;
    println!("  中心球 + 8 环绕球 + 连接线\n");

    println!("========================================");
    println!("测试完成！共 5 帧");
    println!("========================================");

    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}
