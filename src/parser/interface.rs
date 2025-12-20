use nalgebra::{Matrix4, Vector3};

use crate::{geometry::{base::*, pose::RDRPose, shape::*}, proto::{rdr::*, shape::*}};



pub fn position_rdr(position: &Position) -> RDRPosition {
    RDRPosition {
        x: position.x,
        y: position.y,
        z: position.z,
    }
}

pub fn rotate_rdr(rotate: &Rotation) -> RDRRotation {
    RDRRotation {
        rx: rotate.rx,
        ry: rotate.ry,
        rz: rotate.rz,
    }
}

pub fn scale_rdr(scale: &Scale) -> RDRScale {
    RDRScale {
        sx: scale.sx,
        sy: scale.sy,
        sz: scale.sz,
    }
}

pub fn point_rdr(point: &Point) -> RDPoint {
    RDPoint {
        position: Vector3::new(
            point.pos.unwrap().x,
            point.pos.unwrap().y,
            point.pos.unwrap().z
        ),
    }
}

pub fn sphere_rdr(sphere: &Sphere) -> RDSphere {
    RDSphere {
        pose: RDRPosVec {
            pos: Vector3::new(
                sphere.pos.unwrap().x,
                sphere.pos.unwrap().y,
                sphere.pos.unwrap().z
            ),
        },
        radius: sphere.radius,
    }
}

pub fn cube_rdr(cube: &Cube) -> RDCube {
    // 获取旋转矩阵，如果没有提供旋转信息，则使用单位矩阵
    let rot_mat = cube.rot.as_ref().map(|r| rotate_rdr(r).to_matrix()).unwrap_or_else(|| {
        RDRRotation { rx: 0.0, ry: 0.0, rz: 0.0 }.to_matrix()
    });
    
    // 创建带有位置和旋转的姿态矩阵
    let pose_matrix = Matrix4::new(
        rot_mat.m11, rot_mat.m12, rot_mat.m13, cube.pos.as_ref().map_or(0.0, |p| p.x),
        rot_mat.m21, rot_mat.m22, rot_mat.m23, cube.pos.as_ref().map_or(0.0, |p| p.y),
        rot_mat.m31, rot_mat.m32, rot_mat.m33, cube.pos.as_ref().map_or(0.0, |p| p.z),
        0.0, 0.0, 0.0, 1.0,
    );
    
    // 将scale转换为edges，如果没有提供scale信息，则使用默认值(1.0, 1.0, 1.0)
    let edges = Vector3::new(
        cube.scale.as_ref().map_or(1.0, |s| s.sx),
        cube.scale.as_ref().map_or(1.0, |s| s.sy),
        cube.scale.as_ref().map_or(1.0, |s| s.sz)
    );
    
    RDCube {
        pose: RDRPose {
            pose: pose_matrix,
        },
        edges: edges,
    }
}