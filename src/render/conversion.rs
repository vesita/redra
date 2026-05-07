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
            if length < 0.001 {
                log::warn!("Line 长度退化 ({:.6})，跳过渲染。建议检查起终点是否重合。", length);
                return None;
            }
            Mesh3d(meshes.add(Cylinder::new(0.02, length)))
        }
        Some(UMesh::Cylinder(cylinder)) => Mesh3d(meshes.add(Cylinder::new(cylinder.radius, cylinder.height))),
        Some(UMesh::Cone(cone)) => Mesh3d(meshes.add(Cone::new(cone.radius, cone.height))),
        Some(UMesh::Cube(cube)) => {
            if cube.vertices.len() < 8 {
                log::warn!("Cube 顶点数不足 ({}/8)，跳过渲染。", cube.vertices.len());
                return None;
            }
            let raw: Vec<[f32; 3]> = cube.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();

            // AABB 退化检查
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for p in &raw {
                for i in 0..3 { min[i] = min[i].min(p[i]); max[i] = max[i].max(p[i]); }
            }
            let dims = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
            if dims[0] < 0.001 || dims[1] < 0.001 || dims[2] < 0.001 {
                log::warn!(
                    "Cube 维度退化 (w={:.4}, h={:.4}, d={:.4})，跳过渲染。\
                     常见原因：点云共面/共线/单点聚类。建议改用 Sphere 或 Point。",
                    dims[0], dims[1], dims[2]
                );
                return None;
            }

            Mesh3d(meshes.add(build_obb_mesh(&raw)))
        }
        None => return None,
    };
    Some(bevy_mesh)
}

/// 从 8 个角点构建 OBB 三角面 mesh（顶点相对于质心）。
fn build_obb_mesh(raw: &[[f32; 3]]) -> Mesh {
    // 质心（与客户端 tx/ty/tz 一致）
    let cx: f32 = raw.iter().map(|p| p[0]).sum::<f32>() / 8.0;
    let cy: f32 = raw.iter().map(|p| p[1]).sum::<f32>() / 8.0;
    let cz: f32 = raw.iter().map(|p| p[2]).sum::<f32>() / 8.0;
    let center = [cx, cy, cz];

    // 居中顶点
    let verts: Vec<[f32; 3]> = raw.iter()
        .map(|p| [p[0] - center[0], p[1] - center[1], p[2] - center[2]])
        .collect();

    // 从质心到首点的向量，衍生 3 轴
    let d0 = [verts[0][0] - cx, verts[0][1] - cy, verts[0][2] - cz];
    let mut ax = d0;
    let ref2 = [verts[1][0] - cx, verts[1][1] - cy, verts[1][2] - cz];
    let mut ay = cross(ax, ref2);
    if dot(ay, ay) < 1e-10 {
        let ref3 = [verts[2][0] - cx, verts[2][1] - cy, verts[2][2] - cz];
        ay = cross(ax, ref3);
    }
    let mut az = cross(ax, ay);
    if dot(az, az) < 1e-10 {
        az = [0.0, 1.0, 0.0];
        ax = cross(ay, az);
    }

    // 按符号分类 8 个角
    let mut corner = [0usize; 8];
    for (i, v) in verts.iter().enumerate() {
        let dv = [v[0] - cx, v[1] - cy, v[2] - cz];
        let s0 = if dot(dv, ax) >= 0.0 { 1 } else { 0 };
        let s1 = if dot(dv, ay) >= 0.0 { 1 } else { 0 };
        let s2 = if dot(dv, az) >= 0.0 { 1 } else { 0 };
        corner[s0 * 4 + s1 * 2 + s2] = i;
    }

    // 6 面 × 2 三角 = 12 三角
    #[rustfmt::skip]
    let indices: &[u32] = &[
        // x+
        corner[7] as u32, corner[3] as u32, corner[1] as u32,
        corner[7] as u32, corner[1] as u32, corner[5] as u32,
        // x-
        corner[0] as u32, corner[2] as u32, corner[6] as u32,
        corner[0] as u32, corner[6] as u32, corner[4] as u32,
        // y+
        corner[6] as u32, corner[2] as u32, corner[3] as u32,
        corner[6] as u32, corner[3] as u32, corner[7] as u32,
        // y-
        corner[0] as u32, corner[4] as u32, corner[5] as u32,
        corner[0] as u32, corner[5] as u32, corner[1] as u32,
        // z+
        corner[5] as u32, corner[7] as u32, corner[6] as u32,
        corner[5] as u32, corner[6] as u32, corner[4] as u32,
        // z-
        corner[0] as u32, corner[1] as u32, corner[3] as u32,
        corner[0] as u32, corner[3] as u32, corner[2] as u32,
    ];

    let normals = compute_face_normals(&verts, indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices.to_vec()));
    mesh
}

/// 按面计算平滑法线（每个三角的 3 个顶点共享面法线）。
fn compute_face_normals(verts: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; verts.len()];
    for tri in indices.chunks(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let e1 = sub(verts[i1], verts[i0]);
        let e2 = sub(verts[i2], verts[i0]);
        let n = cross(e1, e2);
        for &i in &[i0, i1, i2] {
            normals[i] = add(normals[i], n);
        }
    }
    for n in &mut *normals {
        let len = dot(*n, *n).sqrt();
        if len > 1e-10 {
            n[0] /= len; n[1] /= len; n[2] /= len;
        }
    }
    normals
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn proto_transform_to_bevy(transform: &ExTransform) -> Transform {
    Transform::from_translation(Vec3::new(transform.x, transform.y, transform.z))
        .with_rotation(Quat::from_euler(EulerRot::XYZ, transform.rx, transform.ry, transform.rz))
        .with_scale(Vec3::new(transform.sx, transform.sy, transform.sz))
}
