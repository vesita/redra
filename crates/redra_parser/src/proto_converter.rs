use redra_proto::proto::{shape, designation, command, pointcloud, formats};
use crate::core::{
    RDPack, InternalShapePack, InternalPointCloudPack, 
    InternalFormatPack, InternalFormatData,
    InternalImageData, InternalTextData, InternalModelData,
    InternalAudioData, InternalVideoData
};
use bevy::prelude::{Mesh, Transform, Vec3, Sphere, Cuboid};

/// 将 Protobuf ShapePack 转换为内部 InternalShapePack
pub fn convert_shape_pack(proto_pack: &shape::ShapePack) -> Result<InternalShapePack, String> {
    let data = proto_pack.data.as_ref()
        .ok_or("ShapePack 数据为空")?;

    match data {
        shape::shape_pack::Data::Point(point) => {
            let pos = point.pos.as_ref().unwrap_or(&redra_proto::proto::transform::Translation { x: 0.0, y: 0.0, z: 0.0 });
            let transform = Transform::from_translation(Vec3::new(pos.x, pos.y, pos.z));

            Ok(InternalShapePack {
                mesh: std::sync::Arc::new(Mesh::from(Sphere { radius: 0.05 })),
                transform,
                material: "point_material".to_string(),
                source: Some("point".to_string()),
            })
        }
        shape::shape_pack::Data::Sphere(sphere) => {
            let pos = sphere.pos.as_ref().unwrap_or(&redra_proto::proto::transform::Translation { x: 0.0, y: 0.0, z: 0.0 });
            let transform = Transform::from_translation(Vec3::new(pos.x, pos.y, pos.z))
                .with_scale(Vec3::splat(sphere.radius));

            let mesh = std::sync::Arc::new(Mesh::from(Sphere { radius: sphere.radius }));

            Ok(InternalShapePack {
                mesh,
                transform,
                material: "sphere_material".to_string(),
                source: Some("sphere".to_string()),
            })
        }
        shape::shape_pack::Data::Cube(cube) => {
            let translation = cube.translation.as_ref().unwrap_or(&redra_proto::proto::transform::Translation { x: 0.0, y: 0.0, z: 0.0 });
            let scale = cube.scale.as_ref().unwrap_or(&redra_proto::proto::transform::Scale { sx: 1.0, sy: 1.0, sz: 1.0 });
            let transform = Transform::from_translation(Vec3::new(translation.x, translation.y, translation.z))
                .with_scale(Vec3::new(scale.sx, scale.sy, scale.sz));

            Ok(InternalShapePack {
                mesh: std::sync::Arc::new(Mesh::from(Cuboid::new(scale.sx, scale.sy, scale.sz))),
                transform,
                material: "cube_material".to_string(),
                source: Some("cube".to_string()),
            })
        }
        // ... 其他形状
        _ => Err("不支持的形状类型".to_string()),
    }
}

/// 将 Protobuf FormatPack 转换为内部 InternalFormatPack
fn convert_format_pack(proto_pack: &formats::FormatPack) -> Result<InternalFormatPack, String> {
    let data = proto_pack.data.as_ref()
        .ok_or("FormatPack 数据为空")?;

    let internal_data = match data {
        formats::format_pack::Data::Image(image) => {
            InternalFormatData::Image(InternalImageData {
                data: image.data.clone(),
                mime_type: image.mime_type.clone(),
                width: image.width,
                height: image.height,
            })
        }
        formats::format_pack::Data::Text(text) => {
            InternalFormatData::Text(InternalTextData {
                content: text.content.clone(),
                language: text.language.clone(),
                encoding: text.encoding.clone(),
            })
        }
        formats::format_pack::Data::Model(model) => {
            InternalFormatData::Model(InternalModelData {
                data: model.data.clone(),
                format: model.format.clone(),
                textures: model.textures.clone(),
            })
        }
        formats::format_pack::Data::Audio(audio) => {
            InternalFormatData::Audio(InternalAudioData {
                data: audio.data.clone(),
                format: audio.format.clone(),
                sample_rate: audio.sample_rate,
                channels: audio.channels,
            })
        }
        formats::format_pack::Data::Video(video) => {
            InternalFormatData::Video(InternalVideoData {
                data: video.data.clone(),
                format: video.format.clone(),
                width: video.width,
                height: video.height,
                duration: video.duration,
            })
        }
    };

    Ok(InternalFormatPack {
        data: internal_data
    })
}

/// 处理 Command 并转换为 RDPack
pub fn process_command(cmd: command::Command) -> Result<Vec<RDPack>, String> {
    let mut packs = Vec::new();

    match cmd.cmd_pack {
        Some(command::command::CmdPack::Designation(design_cmd)) => {
            if let Some(designation::design_cmd::Cmd::Spawn(spawn)) = design_cmd.cmd {
                match spawn.data {
                    Some(designation::spawn::Data::ShapeData(shape_pack)) => {
                        let rd_shape = convert_shape_pack(&shape_pack)?;
                        packs.push(RDPack::SpawnShape(Box::new(rd_shape)));
                    }
                    Some(designation::spawn::Data::FormatData(format_pack)) => {
                        let rd_format = convert_format_pack(&format_pack)?;
                        packs.push(RDPack::SpawnFormat(Box::new(rd_format)));
                    }
                    _ => {
                        // 处理其他数据类型或忽略
                    }
                }
            }
        }
        Some(command::command::CmdPack::PointCloud(pc)) => {
            let points = pc.points.iter()
                .map(|p| (p.x as f32, p.y as f32, p.z as f32))
                .collect();

            packs.push(RDPack::PointCloud(InternalPointCloudPack {
                frame_id: pc.frame_id,
                timestamp: pc.timestamp as f64,  // 转换u64到f64
                points,
            }));
        }
        // ... 其他命令类型
        _ => {}
    }

    Ok(packs)
}

/// 从 RDPack 中提取几何数据
pub fn extract_geometry(rd_pack: &RDPack) -> Option<&InternalShapePack> {
    match rd_pack {
        RDPack::SpawnShape(shape_pack) => Some(shape_pack.as_ref()),
        _ => None,
    }
}