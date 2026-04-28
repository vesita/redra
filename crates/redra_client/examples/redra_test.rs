//! 数据管理功能验证示例
//! 
//! 本示例演示了如何使用redra_client进行数据管理功能的验证
//! 生成多帧数据以测试帧回放UI的功能

use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{ExObject, ExTransform, ExMesh, ExCommand, Point, Sphere, Cylinder, Cone, CommandType};
use redra_client::client::link::Link;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("启动 Redra 数据管理功能验证示例");
    println!("将发送 5 帧数据用于测试帧回放UI\n");
    
    // 直接创建连接，避免使用全局连接
    let link = Link::connect().await?;
    println!("客户端已连接到服务器\n");

    // 第1帧：初始状态 - 3个对象
    println!("=== 发送第 1 帧 ===");
    send_object(&link, 1, "sphere", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], "red").await?;
    send_object(&link, 2, "cylinder", [2.0, 0.0, 0.0], [0.5, 1.5, 0.5], "green").await?;
    send_object(&link, 3, "cone", [-2.0, 0.0, 0.0], [0.8, 1.2, 0.8], "blue").await?;
    send_frame_end(&link).await?;
    println!("第 1 帧发送完成（3个对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第2帧：对象移动 - 位置变化
    println!("=== 发送第 2 帧 ===");
    send_object(&link, 1, "sphere", [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], "red").await?;
    send_object(&link, 2, "cylinder", [2.0, 1.0, 0.0], [0.5, 1.5, 0.5], "green").await?;
    send_object(&link, 3, "cone", [-1.0, 1.0, 0.0], [0.8, 1.2, 0.8], "blue").await?;
    send_frame_end(&link).await?;
    println!("第 2 帧发送完成（3个对象，位置更新）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第3帧：添加新对象 - 增加到4个
    println!("=== 发送第 3 帧 ===");
    send_object(&link, 1, "sphere", [2.0, 2.0, 0.0], [1.0, 1.0, 1.0], "red").await?;
    send_object(&link, 2, "cylinder", [2.0, 2.0, 0.0], [0.5, 1.5, 0.5], "green").await?;
    send_object(&link, 3, "cone", [0.0, 2.0, 0.0], [0.8, 1.2, 0.8], "blue").await?;
    send_object(&link, 4, "sphere", [0.0, 0.0, 0.0], [0.6, 0.6, 0.6], "yellow").await?;
    send_frame_end(&link).await?;
    println!("第 3 帧发送完成（4个对象，新增1个）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第4帧：删除对象 - 减少到3个
    println!("=== 发送第 4 帧 ===");
    send_object(&link, 1, "sphere", [3.0, 3.0, 0.0], [1.0, 1.0, 1.0], "red").await?;
    send_object(&link, 3, "cone", [1.0, 3.0, 0.0], [0.8, 1.2, 0.8], "blue").await?;
    send_object(&link, 4, "sphere", [1.0, 1.0, 0.0], [0.6, 0.6, 0.6], "yellow").await?;
    send_frame_end(&link).await?;
    println!("第 4 帧发送完成（3个对象，删除ID=2）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第5帧：最终状态 - 2个对象
    println!("=== 发送第 5 帧 ===");
    send_object(&link, 1, "sphere", [4.0, 4.0, 0.0], [1.2, 1.2, 1.2], "red").await?;
    send_object(&link, 4, "sphere", [2.0, 2.0, 0.0], [0.8, 0.8, 0.8], "yellow").await?;
    send_frame_end(&link).await?;
    println!("第 5 帧发送完成（2个对象，删除ID=3）\n");

    println!("========================================");
    println!("数据管理功能验证示例执行完成！");
    println!("共发送 5 帧数据：");
    println!("  帧1: 3个对象（初始状态）");
    println!("  帧2: 3个对象（位置更新）");
    println!("  帧3: 4个对象（新增对象）");
    println!("  帧4: 3个对象（删除对象）");
    println!("  帧5: 2个对象（最终状态）");
    println!("\n请在UI中测试：");
    println!("  - 帧切换按钮（上一帧/下一帧）");
    println!("  - 时间轴拖动");
    println!("  - 播放/暂停功能");
    println!("  - 播放速度调节");
    println!("========================================");

    // 保持连接一段时间以允许服务器处理数据
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

// 发送对象的辅助函数
async fn send_object(
    link: &Link,
    id: u64,
    mesh_type: &str,
    position: [f32; 3],
    scale: [f32; 3],
    material: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let unit = create_test_unit(id, mesh_type, position, scale, material.to_string());
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

// 辅助函数：创建测试单元
fn create_test_unit(
    id: u64,
    mesh_type: &str,
    position: [f32; 3],
    scale: [f32; 3],
    material: String,
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