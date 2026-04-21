use expto::prelude::*;
use expto::rdmp::auto::unit::generate_unit;
use expto::rdmp::{ExObject, ExMesh, Point, Cylinder, Cone};

use crate::client::link::get_link;

// 定义一个 trait 来扩展 Unit 的功能
pub trait AutoSend4Unit {
    async fn send(&self) -> Result<(), String>;
}

impl AutoSend4Unit for Unit { 
    async fn send(&self) -> Result<(), String> { 
        match encode(self) {
            Ok(buf) => {
                let link = get_link();
                link.send(&buf).await?;
            },
            Err(e) => return Err(format!("{}", e)),
        }
        Ok(())
    }
}


pub async fn send_point(
    x: f32,
    y: f32,
    z: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point: Point = (x, y, z).into();
    let mesh: ExMesh = point.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);

    unit.send().await?;
    Ok(())
}

pub async fn send_line(
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point1: Point = (x1, y1, z1).into();
    let point2: Point = (x2, y2, z2).into();
    let line: Line = (point1, point2).into();
    let mesh: ExMesh = line.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

pub async fn send_sphere(
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let point: Point = (x, y, z).into();
    let sphere: Sphere = (point, radius).into();
    let mesh: ExMesh = sphere.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

pub async fn send_cylinder(
    radius: f32,
    height: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let cylinder: Cylinder = (radius, height).into();
    let mesh: ExMesh = cylinder.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}

pub async fn send_cone(
    radius: f32,
    height: f32,
) -> Result<(), String> {
    let mut unit = generate_unit();
    let cone: Cone = (radius, height).into();
    let mesh: ExMesh = cone.into();
    let object: ExObject = mesh.into();
    let _ = unit.set_object(object);
    unit.send().await?;
    Ok(())
}
