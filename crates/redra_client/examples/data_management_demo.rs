//! 数据管理功能验证示例
//! 
//! 本示例演示了如何使用redra_client进行数据管理功能的验证

use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{ExObject, ExTransform, ExMesh, ExCommand, Point, Sphere, Cylinder, Cone, CommandType};
use redra_client::client::link::Link;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 启动 Redra 数据管理功能验证示例");
    
    // 直接创建连接，避免使用全局连接
    let link = Link::connect().await?;
    println!("✅ 客户端已连接到服务器");

    // 模拟发送一些数据单元进行验证
    println!("📝 开始发送测试数据...");
    
    // 发送第一个对象
    let unit1: Unit = create_test_unit(
        1,
        "sphere",
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        "red".to_string(),
    );
    send_unit(&link, &unit1).await?;
    println!("✅ 发送了第一个对象");

    // 等待一小段时间
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 发送第二个对象
    let unit2: Unit = create_test_unit(
        2,
        "cube",
        [2.0, 0.0, 0.0],
        [1.5, 1.5, 1.5],
        "blue".to_string(),
    );
    send_unit(&link, &unit2).await?;
    println!("✅ 发送了第二个对象");

    // 等待一段时间再发送第三个对象
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 发送第三个对象
    let unit3: Unit = create_test_unit(
        3,
        "cylinder",
        [0.0, 2.0, 0.0],
        [1.2, 2.0, 1.2],
        "green".to_string(),
    );
    send_unit(&link, &unit3).await?;
    println!("✅ 发送了第三个对象");

    // 发送帧结束命令
    let frame_end_unit: Unit = create_frame_end_unit();
    send_unit(&link, &frame_end_unit).await?;
    println!("✅ 发送了帧结束命令");

    // 再发送一些对象到下一帧
    tokio::time::sleep(Duration::from_millis(100)).await;

    let unit4: Unit = create_test_unit(
        4,
        "cone",
        [-2.0, 0.0, 0.0],
        [1.0, 1.5, 1.0],
        "yellow".to_string(),
    );
    send_unit(&link, &unit4).await?;
    println!("✅ 发送了第四帧的对象");

    let frame_end_unit2: Unit = create_frame_end_unit();
    send_unit(&link, &frame_end_unit2).await?;
    println!("✅ 发送了第二帧结束命令");

    println!("🎉 数据管理功能验证示例执行完成！");
    println!("📊 数据已发送到服务器，等待渲染器处理...");

    // 保持连接一段时间以允许服务器处理数据
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(())
}

// 发送单元的辅助函数
async fn send_unit(link: &Link, unit: &Unit) -> Result<(), Box<dyn std::error::Error>> {
    match encode(unit) {  // 使用 prelude 中导出的编码函数
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
            // 对于立方体，我们使用球体作为替代，因为UMesh枚举中没有Box类型
            "cube" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Sphere(
                    Sphere { 
                        location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                        radius: 1.0 
                    })),
            }),
            "cylinder" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Cylinder(
                    Cylinder { radius: 1.0, height: 2.0 })),
            }),
            "cone" => expto::rdmp::ex_object::UObject::Mesh(ExMesh {
                u_mesh: Some(expto::rdmp::ex_mesh::UMesh::Cone(
                    Cone { radius: 1.0, height: 1.5 })),
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