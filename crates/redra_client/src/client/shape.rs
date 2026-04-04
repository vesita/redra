use redra_proto::proto::{
    command::{Command, command::CmdPack},
    designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
    shape::{ShapePack, shape_pack, Color, Pose},
    transform::{Translation, Scale, Rotation},
    target::TargetId,
};

/// 创建点形状命令
pub fn create_point_command(
    x: f32,
    y: f32,
    z: f32,
    color: Option<Color>,
    target_id: Option<TargetId>,
    name: String,
) -> Command {
    use redra_proto::proto::shape::Point;
    
    let point = Point {
        pos: Some(Translation { x, y, z }),
        size: 1.0,
    };
    
    let spawn = Spawn {
        id: target_id,
        name: name.clone(),
        tags: vec!["shape".to_string(), "point".to_string()],
        initial_pose: Some(Pose {
            translation: Some(Translation { x, y, z }),
            rotation: None,
            scale: Some(Scale { sx: 1.0, sy: 1.0, sz: 1.0 }),
        }),
        data: Some(spawn::Data::ShapeData(ShapePack {
            name,
            tags: vec!["geometry".to_string()],
            color,
            data: Some(shape_pack::Data::Point(point)),
        })),
    };
    
    Command {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("cmd_point_{}_{}_{}", x, y, z),
        cmd_pack: Some(CmdPack::Designation(DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        })),
    }
}

/// 创建立方体形状命令
pub fn create_cube_command(
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    depth: f32,
    color: Option<Color>,
    target_id: Option<TargetId>,
    name: String,
) -> Command {
    use redra_proto::proto::shape::Cube;
    
    let cube = Cube {
        translation: Some(Translation { x, y, z }),
        rotation: Some(Rotation { rx: 0.0, ry: 0.0, rz: 0.0 }),
        scale: Some(Scale { sx: width, sy: height, sz: depth }),
        is_wireframe: false,
    };
    
    let spawn = Spawn {
        id: target_id,
        name: name.clone(),
        tags: vec!["shape".to_string(), "cube".to_string()],
        initial_pose: Some(Pose {
            translation: Some(Translation { x, y, z }),
            rotation: Some(Rotation { rx: 0.0, ry: 0.0, rz: 0.0 }),
            scale: Some(Scale { sx: width, sy: height, sz: depth }),
        }),
        data: Some(spawn::Data::ShapeData(ShapePack {
            name,
            tags: vec!["geometry".to_string()],
            color,
            data: Some(shape_pack::Data::Cube(cube)),
        })),
    };
    
    Command {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("cmd_cube_{}_{}_{}", x, y, z),
        cmd_pack: Some(CmdPack::Designation(DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        })),
    }
}

/// 创建球体形状命令
pub fn create_sphere_command(
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    color: Option<Color>,
    target_id: Option<TargetId>,
    name: String,
) -> Command {
    use redra_proto::proto::shape::Sphere;
    
    let sphere = Sphere {
        pos: Some(Translation { x, y, z }),
        radius,
        segments: 32,
    };
    
    let spawn = Spawn {
        id: target_id,
        name: name.clone(),
        tags: vec!["shape".to_string(), "sphere".to_string()],
        initial_pose: Some(Pose {
            translation: Some(Translation { x, y, z }),
            rotation: None,
            scale: Some(Scale { sx: 1.0, sy: 1.0, sz: 1.0 }),
        }),
        data: Some(spawn::Data::ShapeData(ShapePack {
            name,
            tags: vec!["geometry".to_string()],
            color,
            data: Some(shape_pack::Data::Sphere(sphere)),
        })),
    };
    
    Command {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("cmd_sphere_{}_{}_{}", x, y, z),
        cmd_pack: Some(CmdPack::Designation(DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        })),
    }
}

/// 创建线段形状命令
pub fn create_line_command(
    start_x: f32,
    start_y: f32,
    start_z: f32,
    end_x: f32,
    end_y: f32,
    end_z: f32,
    color: Option<Color>,
    target_id: Option<TargetId>,
    name: String,
) -> Command {
    use redra_proto::proto::shape::{Segment, Point};
    
    let segment = Segment {
        start: Some(Point {
            pos: Some(Translation { x: start_x, y: start_y, z: start_z }),
            size: 1.0,
        }),
        end: Some(Point {
            pos: Some(Translation { x: end_x, y: end_y, z: end_z }),
            size: 1.0,
        }),
        thickness: 1.0,
    };
    
    let spawn = Spawn {
        id: target_id,
        name: name.clone(),
        tags: vec!["shape".to_string(), "segment".to_string()],
        initial_pose: Some(Pose {
            translation: Some(Translation { x: start_x, y: start_y, z: start_z }),
            rotation: None,
            scale: Some(Scale { sx: 1.0, sy: 1.0, sz: 1.0 }),
        }),
        data: Some(spawn::Data::ShapeData(ShapePack {
            name,
            tags: vec!["geometry".to_string()],
            color,
            data: Some(shape_pack::Data::Segment(segment)),
        })),
    };
    
    Command {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        command_id: format!("cmd_segment_{}_{}_{}_{}_{}_{}", start_x, start_y, start_z, end_x, end_y, end_z),
        cmd_pack: Some(CmdPack::Designation(DesignCmd {
            cmd: Some(Cmd::Spawn(spawn)),
        })),
    }
}