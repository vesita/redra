//! 分组点云示例 — 按材质分组发送点云，不同组渲染为不同颜色
//!
//! 演示两种方式：
//! 1. `ShapeBuilder::point_cloud_grouped()` — 高层链式 API
//! 2. `send_point_cloud_grouped()` — 低层函数式 API
//!
//! 运行前先启动 redra 服务端：
//!   cargo run

use redra_client::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== 分组点云示例 ===\n");

    // ── 第 1 帧：ShapeBuilder 链式 API ─────────────────
    println!("帧 1: ShapeBuilder 链式 API（聚类色板）");

    // 模拟 4 个聚类
    let cluster_0 = generate_cluster([0.0, 0.0, 0.0], 1.0, 200);
    let cluster_1 = generate_cluster([5.0, 0.0, 0.0], 0.8, 150);
    let cluster_2 = generate_cluster([0.0, 0.0, 5.0], 1.2, 180);
    let cluster_3 = generate_cluster([5.0, 0.0, 5.0], 0.6, 120);

    ShapeBuilder::point_cloud_grouped()
        .group(cluster_0, defaults::cluster::C01_RED)
        .group(cluster_1, defaults::cluster::C07_CYAN)
        .group(cluster_2, defaults::cluster::C05_GREEN)
        .group(cluster_3, defaults::cluster::C10_VIOLET)
        .send()
        .await?;

    send_frame_end().await?;
    println!("  4 个聚类，共 650 个点\n");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── 第 2 帧：低层函数式 API ────────────────────────
    println!("帧 2: send_point_cloud_grouped（语义色）");

    let ground = generate_plane([-5.0, -1.0, -5.0], [5.0, -1.0, 5.0], 0.5, 400);
    let obstacles = generate_cluster([1.0, 0.0, 1.0], 0.3, 80);
    let noise = generate_cluster([-3.0, 2.0, -3.0], 2.0, 50);

    send_point_cloud_grouped(&[
        (&ground, defaults::semantic::GROUND),
        (&obstacles, defaults::semantic::ALERT),
        (&noise, defaults::semantic::NOISE),
    ])
    .await?;

    send_frame_end().await?;
    println!("  地面 + 障碍物 + 噪声，共 530 个点\n");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── 第 3 帧：循环使用 12 色聚类色板 ────────────────
    println!("帧 3: 12 色循环（大规模点云）");

    let mut builder = ShapeBuilder::point_cloud_grouped();
    for i in 0..12 {
        let center = [
            (i as f32 * 3.0 - 16.5).cos() * 6.0,
            0.0,
            (i as f32 * 3.0 - 16.5).sin() * 6.0,
        ];
        let points = generate_cluster(center, 1.0, 100);
        let color = defaults::cluster::ALL[i];
        builder = builder.group(points, color);
    }
    builder.send().await?;

    send_frame_end().await?;
    println!("  12 个聚类，共 1200 个点\n");

    println!("=== 完成！共 3 帧 ===");
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}

/// 在球形区域内生成随机点
fn generate_cluster(center: [f32; 3], radius: f32, count: usize) -> Vec<[f32; 3]> {
    (0..count)
        .map(|i| {
            let theta = (i as f32 * 2.399_96) % (2.0 * std::f32::consts::PI); // 黄金角
            let r = radius * ((i as f32 * 0.618) % 1.0).sqrt();
            let y = ((i as f32 * 0.381) % 1.0 - 0.5) * radius;
            [
                center[0] + r * theta.cos(),
                center[1] + y,
                center[2] + r * theta.sin(),
            ]
        })
        .collect()
}

/// 在矩形平面内生成均匀网格点
fn generate_plane(min: [f32; 3], max: [f32; 3], step: f32, _count_hint: usize) -> Vec<[f32; 3]> {
    let mut points = Vec::new();
    let mut z = min[2];
    while z <= max[2] {
        let mut x = min[0];
        while x <= max[0] {
            points.push([x, min[1], z]);
            x += step;
        }
        z += step;
    }
    points
}
