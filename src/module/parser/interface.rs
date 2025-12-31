use nalgebra::{Matrix4, Vector3};

use crate::{
    geometry::{
        base::*,
        pose::RDRPose,
        shape::{line::RDSegment, *},
    },
    proto::{
        shape::*,
        transform::{Rotation, Scale, Translation},
    },
};

/// 将protobuf中的Translation转换为内部的RDTranslation类型
/// 
/// # 参数
/// * `position` - protobuf定义的Translation对象
/// 
/// # 返回值
/// * `RDTranslation` - 内部定义的平移数据结构
pub fn position_rd(position: &Translation) -> RDTranslation {
    RDTranslation {
        x: position.x,
        y: position.y,
        z: position.z,
    }
}

/// 将protobuf中的Rotation转换为内部的RDRRotation类型
/// 
/// # 参数
/// * `rotate` - protobuf定义的Rotation对象
/// 
/// # 返回值
/// * `RDRRotation` - 内部定义的旋转数据结构
pub fn rotate_rd(rotate: &Rotation) -> RDRRotation {
    RDRRotation {
        rx: rotate.rx,
        ry: rotate.ry,
        rz: rotate.rz,
    }
}

/// 将protobuf中的Scale转换为内部的RDRScale类型
/// 
/// # 参数
/// * `scale` - protobuf定义的Scale对象
/// 
/// # 返回值
/// * `RDRScale` - 内部定义的缩放数据结构
pub fn scale_rd(scale: &Scale) -> RDRScale {
    RDRScale {
        sx: scale.sx,
        sy: scale.sy,
        sz: scale.sz,
    }
}

/// 将protobuf中的Point转换为内部的RDPoint类型
/// 
/// # 参数
/// * `point` - protobuf定义的Point对象
/// 
/// # 返回值
/// * `RDPoint` - 内部定义的点数据结构
pub fn point_rd(point: &Point) -> RDPoint {
    RDPoint {
        position: Vector3::new(
            point.pos.unwrap().x,
            point.pos.unwrap().y,
            point.pos.unwrap().z,
        ),
    }
}

/// 将protobuf中的Sphere转换为内部的RDSphere类型
/// 
/// # 参数
/// * `sphere` - protobuf定义的Sphere对象
/// 
/// # 返回值
/// * `RDSphere` - 内部定义的球体数据结构
pub fn sphere_rd(sphere: &Sphere) -> RDSphere {
    RDSphere {
        pose: RDRPosVec {
            pos: Vector3::new(
                sphere.pos.unwrap().x,
                sphere.pos.unwrap().y,
                sphere.pos.unwrap().z,
            ),
        },
        radius: sphere.radius,
    }
}

/// 将protobuf中的Cube转换为内部的RDCube类型
/// 
/// 该函数将protobuf定义的立方体信息转换为内部表示，包括位置、旋转和缩放
/// 
/// # 参数
/// * `cube` - protobuf定义的Cube对象
/// 
/// # 返回值
/// * `RDCube` - 内部定义的立方体数据结构
pub fn cube_rd(cube: &Cube) -> RDCube {
    // 获取旋转矩阵，如果没有提供旋转信息，则使用单位矩阵
    let rot_mat = cube
        .rotation
        .as_ref()
        .map(|r| rotate_rd(r).to_matrix())
        .unwrap_or_else(|| {
            RDRRotation {
                rx: 0.0,
                ry: 0.0,
                rz: 0.0,
            }
            .to_matrix()
        });

    // 创建带有位置和旋转的姿态矩阵
    let pose_matrix = Matrix4::new(
        rot_mat.m11,
        rot_mat.m12,
        rot_mat.m13,
        cube.translation.as_ref().map_or(0.0, |p| p.x),
        rot_mat.m21,
        rot_mat.m22,
        rot_mat.m23,
        cube.translation.as_ref().map_or(0.0, |p| p.y),
        rot_mat.m31,
        rot_mat.m32,
        rot_mat.m33,
        cube.translation.as_ref().map_or(0.0, |p| p.z),
        0.0,
        0.0,
        0.0,
        1.0,
    );

    // 将scale转换为edges，如果没有提供scale信息，则使用默认值(1.0, 1.0, 1.0)
    let edges = Vector3::new(
        cube.scale.as_ref().map_or(1.0, |s| s.sx),
        cube.scale.as_ref().map_or(1.0, |s| s.sy),
        cube.scale.as_ref().map_or(1.0, |s| s.sz),
    );

    RDCube {
        pose: RDRPose { pose: pose_matrix },
        edges: edges,
    }
}

/// 将protobuf中的Segment转换为内部的RDSegment类型
/// 
/// # 参数
/// * `segment` - protobuf定义的Segment对象
/// 
/// # 返回值
/// * `RDSegment` - 内部定义的线段数据结构
pub fn segment_rd(segment: &Segment) -> RDSegment {
    RDSegment {
        start: point_rd(segment.start.as_ref().unwrap()),
        end: point_rd(segment.end.as_ref().unwrap()),
    }
}