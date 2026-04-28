//! 标签功能验证示例
//! 
//! 本示例演示了如何使用redra_client发送带标签的3D对象
//! 用于测试标签显示、样式配置和位置偏移功能

use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{ExObject, ExTransform, ExMesh, ExCommand, Point, Sphere, Cylinder, Cone, CommandType, Tag, TagStyle};
use redra_client::client::link::Link;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("启动 Redra 标签功能验证示例");
    println!("将发送包含标签的3D对象数据用于测试标签显示功能\n");
    
    // 直接创建连接，避免使用全局连接
    let link = Link::connect().await?;
    println!("客户端已连接到服务器\n");

    // 第1帧：基础标签 - 简单文本标签
    println!("=== 发送第 1 帧：基础标签 ===");
    send_object_with_tag(
        &link, 
        1, 
        "sphere", 
        [0.0, 0.0, 0.0], 
        [1.0, 1.0, 1.0], 
        "red",
        "球体对象",
        None, // 使用默认偏移
        None, // 使用默认样式
    ).await?;
    send_frame_end(&link).await?;
    println!("第 1 帧发送完成（1个带标签的对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第2帧：自定义偏移标签 - 标签位置偏移
    println!("=== 发送第 2 帧：自定义偏移标签 ===");
    let offset = Some(ExTransform {
        x: 0.0,
        y: 2.0,  // 向上偏移2个单位
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    });
    send_object_with_tag(
        &link, 
        1, 
        "sphere", 
        [0.0, 0.0, 0.0], 
        [1.0, 1.0, 1.0], 
        "red",
        "偏移标签",
        offset,
        None,
    ).await?;
    send_frame_end(&link).await?;
    println!("第 2 帧发送完成（标签向上偏移2个单位）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第3帧：自定义样式标签 - 不同颜色和字体大小
    println!("=== 发送第 3 帧：自定义样式标签 ===");
    let custom_style = Some(TagStyle {
        font_size: 20.0,
        bg_r: 0.8,
        bg_g: 0.2,
        bg_b: 0.2,
        bg_a: 0.9,
        text_r: 1.0,
        text_g: 1.0,
        text_b: 1.0,
        text_a: 1.0,
        corner_radius: 8.0,
    });
    send_object_with_tag(
        &link, 
        1, 
        "sphere", 
        [0.0, 0.0, 0.0], 
        [1.0, 1.0, 1.0], 
        "red",
        "红色背景大字体",
        None,
        custom_style,
    ).await?;
    send_frame_end(&link).await?;
    println!("第 3 帧发送完成（红色背景，20号字体）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第4帧：多个对象带标签 - 同时显示多个标签
    println!("=== 发送第 4 帧：多个对象带标签 ===");
    
    // 对象1：球体
    let style1 = Some(TagStyle {
        font_size: 16.0,
        bg_r: 0.2,
        bg_g: 0.6,
        bg_b: 0.2,
        bg_a: 0.85,
        text_r: 1.0,
        text_g: 1.0,
        text_b: 1.0,
        text_a: 1.0,
        corner_radius: 6.0,
    });
    send_object_with_tag(
        &link, 
        1, 
        "sphere", 
        [-2.0, 0.0, 0.0], 
        [0.8, 0.8, 0.8], 
        "green",
        "绿色球体",
        None,
        style1,
    ).await?;

    // 对象2：圆柱体
    let style2 = Some(TagStyle {
        font_size: 14.0,
        bg_r: 0.2,
        bg_g: 0.2,
        bg_b: 0.8,
        bg_a: 0.85,
        text_r: 1.0,
        text_g: 1.0,
        text_b: 1.0,
        text_a: 1.0,
        corner_radius: 4.0,
    });
    send_object_with_tag(
        &link, 
        2, 
        "cylinder", 
        [0.0, 0.0, 0.0], 
        [0.5, 1.5, 0.5], 
        "blue",
        "蓝色圆柱",
        None,
        style2,
    ).await?;

    // 对象3：圆锥
    let style3 = Some(TagStyle {
        font_size: 18.0,
        bg_r: 0.8,
        bg_g: 0.8,
        bg_b: 0.2,
        bg_a: 0.85,
        text_r: 0.0,
        text_g: 0.0,
        text_b: 0.0,
        text_a: 1.0,
        corner_radius: 10.0,
    });
    send_object_with_tag(
        &link, 
        3, 
        "cone", 
        [2.0, 0.0, 0.0], 
        [0.8, 1.2, 0.8], 
        "yellow",
        "黄色圆锥",
        None,
        style3,
    ).await?;

    send_frame_end(&link).await?;
    println!("第 4 帧发送完成（3个带不同样式标签的对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第5帧：复杂场景 - 混合使用各种标签配置
    println!("=== 发送第 5 帧：复杂场景 ===");
    
    // 对象1：带偏移和自定义样式
    let offset1 = Some(ExTransform {
        x: 1.5,
        y: 1.5,
        z: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    });
    let style_complex1 = Some(TagStyle {
        font_size: 22.0,
        bg_r: 0.9,
        bg_g: 0.1,
        bg_b: 0.9,
        bg_a: 0.9,
        text_r: 1.0,
        text_g: 1.0,
        text_b: 1.0,
        text_a: 1.0,
        corner_radius: 12.0,
    });
    send_object_with_tag(
        &link, 
        1, 
        "sphere", 
        [-3.0, -2.0, 0.0], 
        [1.2, 1.2, 1.2], 
        "magenta",
        "复杂标签1",
        offset1,
        style_complex1,
    ).await?;

    // 对象2：仅文本，无样式
    send_object_with_tag(
        &link, 
        2, 
        "cylinder", 
        [0.0, -2.0, 0.0], 
        [0.6, 1.8, 0.6], 
        "cyan",
        "简约标签",
        None,
        None,
    ).await?;

    // 对象3：小字体半透明背景
    let style_complex2 = Some(TagStyle {
        font_size: 12.0,
        bg_r: 0.0,
        bg_g: 0.0,
        bg_b: 0.0,
        bg_a: 0.5,
        text_r: 0.0,
        text_g: 1.0,
        text_b: 0.0,
        text_a: 1.0,
        corner_radius: 2.0,
    });
    send_object_with_tag(
        &link, 
        3, 
        "cone", 
        [3.0, -2.0, 0.0], 
        [0.7, 1.0, 0.7], 
        "white",
        "半透明黑底绿字",
        None,
        style_complex2,
    ).await?;

    send_frame_end(&link).await?;
    println!("第 5 帧发送完成（3个不同配置的标签）\n");

    println!("========================================");
    println!("标签功能验证示例执行完成！");
    println!("共发送 5 帧数据：");
    println!("  帧1: 1个对象 - 基础标签（默认样式）");
    println!("  帧2: 1个对象 - 自定义偏移标签");
    println!("  帧3: 1个对象 - 自定义样式标签（红色背景，大字体）");
    println!("  帧4: 3个对象 - 多个不同样式标签");
    println!("  帧5: 3个对象 - 复杂混合配置");
    println!("\n请在UI中测试：");
    println!("  - 标签是否正确显示在3D对象附近");
    println!("  - 标签偏移是否生效");
    println!("  - 不同样式配置（颜色、字体大小、圆角）是否正确应用");
    println!("  - 多个标签同时显示时的性能");
    println!("  - 相机移动时标签是否跟随对象");
    println!("  - 标签在视口外的裁剪行为");
    println!("========================================");

    // 保持连接一段时间以允许服务器处理数据
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

// 发送带标签对象的辅助函数
async fn send_object_with_tag(
    link: &Link,
    id: u64,
    mesh_type: &str,
    position: [f32; 3],
    scale: [f32; 3],
    material: &str,
    tag_text: &str,
    tag_offset: Option<ExTransform>,
    tag_style: Option<TagStyle>,
) -> Result<(), Box<dyn std::error::Error>> {
    let unit = create_test_unit_with_tag(
        id, 
        mesh_type, 
        position, 
        scale, 
        material.to_string(),
        tag_text.to_string(),
        tag_offset,
        tag_style,
    );
    send_unit(link, &unit).await?;
    Ok(())
}

// 发送帧结束命令
async fn send_frame_end(link: &Link) -> Result<(), Box<dyn std::error::Error>> {
    let unit = create_frame_end_unit();
    send_unit(link, &unit).await?;
    Ok(())
}

// 发送单元的辅助函数
async fn send_unit(link: &Link, unit: &Unit) -> Result<(), Box<dyn std::error::Error>> {
    match encode(unit) {
        Ok(buf) => {
            link.send(&buf).await.map_err(|e| Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )))?;
        },
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
    Ok(())
}

// 辅助函数：创建带标签的测试单元
fn create_test_unit_with_tag(
    id: u64,
    mesh_type: &str,
    position: [f32; 3],
    scale: [f32; 3],
    material: String,
    tag_text: String,
    tag_offset: Option<ExTransform>,
    tag_style: Option<TagStyle>,
) -> Unit {
    let mut unit = generate_unit();
    
    // 添加ID对象
    unit.objects.push(ExObject {
        u_object: Some(expto::rdmp::ex_object::UObject::Id(id)),
    });
    
    // 设置网格对象
    let mesh_obj = ExObject {
        u_object: Some(match mesh_type {
            "sphere" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Sphere(
                    Sphere { 
                        location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                        radius: 1.0 
                    })),
            }),
            "cube" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Sphere(
                    Sphere { 
                        location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                        radius: 1.0 
                    })),
            }),
            "cylinder" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Cylinder(
                    Cylinder { radius: 0.5, height: 2.0 })),
            }),
            "cone" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Cone(
                    Cone { radius: 0.8, height: 1.5 })),
            }),
            _ => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Sphere(
                    Sphere { 
                        location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                        radius: 1.0 
                    })),
            }),
        }),
    };
    unit.objects.push(mesh_obj);

    // 设置变换对象
    let transform_obj = ExObject {
        u_object: Some(expto::rdmp::ex_object::UObject::Transform(ExTransform {
            x: position[0],
            y: position[1],
            z: position[2],
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
            sx: scale[0],
            sy: scale[1],
            sz: scale[2],
        })),
    };
    unit.objects.push(transform_obj);
    
    // 设置材质对象
    let material_obj = ExObject {
        u_object: Some(expto::rdmp::ex_object::UObject::MaterialId(material)),
    };
    unit.objects.push(material_obj);

    // 设置标签对象
    let tag_obj = ExObject {
        u_object: Some(expto::rdmp::ex_object::UObject::Tag(Tag {
            text: tag_text,
            offset: tag_offset,
            style: tag_style,
        })),
    };
    unit.objects.push(tag_obj);

    unit
}

// 辅助函数：创建帧结束单元
fn create_frame_end_unit() -> Unit {
    let mut unit = generate_unit();
    unit.command = Some(ExCommand {
        u_command: CommandType::Frameend as i32,
    });
    unit
}
