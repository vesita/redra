use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use expto::rdmp::{ExMesh, ExTransform};

/// 将协议网格转为 Bevy Mesh3d。
pub fn proto_mesh_to_bevy(meshes: &mut Assets<Mesh>, proto_mesh: &ExMesh) -> Option<Mesh3d> {
    use expto::rdmp::mesh::ex_mesh::UMesh;
    let bevy_mesh = match &proto_mesh.u_mesh {
        Some(UMesh::Sphere(sphere)) => Mesh3d(meshes.add(Sphere::new(sphere.radius))),
        Some(UMesh::Point(_point)) => {
            let mut mesh = Mesh::new(PrimitiveTopology::PointList, default());
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]]);
            Mesh3d(meshes.add(mesh))
        }
        Some(UMesh::Line(line)) => {
            let start = line.start.as_ref()?;
            let end = line.end.as_ref()?;
            let length = Vec3::new(end.x - start.x, end.y - start.y, end.z - start.z).length();
            if length < 0.001 { return None; }
            Mesh3d(meshes.add(Cylinder::new(0.02, length)))
        }
        Some(UMesh::Cylinder(cylinder)) => Mesh3d(meshes.add(Cylinder::new(cylinder.radius, cylinder.height))),
        Some(UMesh::Cone(cone)) => Mesh3d(meshes.add(Cone::new(cone.radius, cone.height))),
        Some(UMesh::Cube(cube)) => {
            if cube.vertices.len() < 8 { return None; }
            let mut min = [f32::MAX, f32::MAX, f32::MAX];
            let mut max = [f32::MIN, f32::MIN, f32::MIN];
            for v in &cube.vertices {
                min[0] = min[0].min(v.x); min[1] = min[1].min(v.y); min[2] = min[2].min(v.z);
                max[0] = max[0].max(v.x); max[1] = max[1].max(v.y); max[2] = max[2].max(v.z);
            }
            let w = max[0] - min[0];
            let h = max[1] - min[1];
            let d = max[2] - min[2];
            if w < 0.001 || h < 0.001 || d < 0.001 { return None; }
            Mesh3d(meshes.add(Cuboid::new(w, h, d)))
        }
        None => return None,
    };
    Some(bevy_mesh)
}

pub fn proto_transform_to_bevy(transform: &ExTransform) -> Transform {
    Transform::from_translation(Vec3::new(transform.x, transform.y, transform.z))
        .with_rotation(Quat::from_euler(EulerRot::XYZ, transform.rx, transform.ry, transform.rz))
        .with_scale(Vec3::new(transform.sx, transform.sy, transform.sz))
}
