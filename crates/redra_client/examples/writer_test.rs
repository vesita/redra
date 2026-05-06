//! RdraWriter 测试 — 生成 .rdra 文件
//!
//! 生成 5 帧数据的 .rdra 文件，可用 Redra 打开验证。

use redra_client::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut writer = RdraWriter::new();

    // 帧 1：初始场景 — 3 个对象
    writer.spawn(spawn_sphere([0.0, 0.0, 0.0], 1.0, "red").id(1).tag("球体"));
    writer.spawn(spawn_cylinder([3.0, 0.0, 0.0], 0.5, 1.5, "green").id(2).tag("圆柱"));
    writer.spawn(spawn_cone([-3.0, 0.0, 0.0], 0.8, 1.2, "blue").id(3).tag("圆锥"));
    writer.end_frame();
    println!("帧 1: {} 个实体", 3);

    // 帧 2：球体移动 + 新增立方体
    writer.spawn(spawn_sphere([1.5, 1.0, 0.0], 1.0, "red").id(1));
    writer.spawn(spawn_cube(vec![
        (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0),
        (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0),
        (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
    ], "yellow").id(4).tag("立方体"));
    writer.end_frame();
    println!("帧 2: {} 个实体", writer.entity_count());

    // 帧 3：删除圆锥，圆柱变色
    writer.destroy(3);
    writer.spawn(spawn_cylinder([3.0, 0.5, 0.0], 0.5, 1.5, "cyan").id(2));
    writer.end_frame();
    println!("帧 3: {} 个实体（删除圆锥）", writer.entity_count());

    // 帧 4：缩放 + 旋转
    writer.spawn(spawn_sphere([1.5, 1.0, 0.0], 1.0, "magenta").id(1).scale_uniform(1.5));
    writer.spawn(spawn_cylinder([3.0, 0.5, 0.0], 0.5, 1.5, "cyan").id(2).rotation_deg(0.0, 0.0, 45.0));
    writer.end_frame();
    println!("帧 4: {} 个实体（缩放+旋转）", writer.entity_count());

    // 帧 5：最终状态
    writer.spawn(spawn_sphere([0.0, 2.0, 0.0], 0.8, "white").id(1));
    writer.spawn(spawn_line([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], "glow").id(5).tag("连接线"));
    writer.end_frame();
    println!("帧 5: {} 个实体", writer.entity_count());

    // 保存
    let path = "temp/writer_test_output.rdra";
    writer.save(path)?;
    println!("\n已保存 {} 帧到 {}", writer.frame_count(), path);
    println!("用 Redra 打开验证: cargo run -- {}", path);

    Ok(())
}
