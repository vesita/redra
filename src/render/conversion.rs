use bevy::prelude::*;
use expto::rdmp::{ExMesh, ExTransform};

pub fn proto_mesh_to_bevy(meshes: &mut Assets<Mesh>, proto_mesh: &ExMesh) -> Option<Mesh3d> {
    use expto::rdmp::mesh::ex_mesh::UMesh;
    let bevy_mesh = match &proto_mesh.u_mesh {
        Some(UMesh::Sphere(sphere)) => Mesh3d(meshes.add(Sphere::new(sphere.radius))),
        Some(UMesh::Point(_point)) => Mesh3d(meshes.add(Sphere::new(0.05))),
        Some(UMesh::Line(line)) => {
            let start = line.start.as_ref()?;
            let end = line.end.as_ref()?;
            let length = Vec3::new(end.x - start.x, end.y - start.y, end.z - start.z).length();
            if length < 0.001 { return None; }
            Mesh3d(meshes.add(Cylinder::new(0.02, length)))
        }
        Some(UMesh::Cylinder(cylinder)) => Mesh3d(meshes.add(Cylinder::new(cylinder.radius, cylinder.height))),
        Some(UMesh::Cone(cone)) => Mesh3d(meshes.add(Cone::new(cone.radius, cone.height))),
        None => return None,
    };
    Some(bevy_mesh)
}

pub fn proto_transform_to_bevy(transform: &ExTransform) -> Transform {
    Transform::from_translation(Vec3::new(transform.x, transform.y, transform.z))
        .with_rotation(Quat::from_euler(EulerRot::XYZ, transform.rx, transform.ry, transform.rz))
        .with_scale(Vec3::new(transform.sx, transform.sy, transform.sz))
}
