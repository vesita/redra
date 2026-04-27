use std::collections::HashMap;

use bevy::prelude::*;
use expto::rdmp::{CommandType, Unit, ex_object::UObject};
use serde::{Serialize, Deserialize};

use crate::manager::{data::frame::{Inpto, KeyFrame}, data_flow::parser::{e2i_transform, parse_command, extract_id, extract_material_id, extract_tag}};


impl KeyFrame {
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            ids: HashMap::new(),
            packs: Vec::new(),
        }
    }

    pub fn update(&mut self, unit: &Unit) {
        // 如果没有命令，默认当作 Spawn 处理（兼容无命令的 Unit）
        let command = parse_command(unit).unwrap_or(CommandType::Spawn);
        self.match_command(&command, unit);
    }

    pub fn match_command(&mut self, command: &CommandType, unit: &Unit) {
        match command {
            CommandType::Unknown => {},
            CommandType::Spawn => {
                self.react_spawn(unit);
            },
            CommandType::Update => {
                self.react_update(unit);
            },
            CommandType::Destroy => {
                self.react_destroy(unit);
            },
            CommandType::Frameend => {},
        }
    }

    pub fn react_spawn(&mut self, unit: &Unit) {
        // 基于类型识别提取对象，而非依赖固定位置
        let mut id: Option<u64> = None;
        let mut mesh: Option<expto::rdmp::ExMesh> = None;
        let mut transform: Option<expto::rdmp::ExTransform> = None;
        
        for obj in &unit.objects {
            if let Some(u_object) = &obj.u_object {
                match u_object {
                    UObject::Id(obj_id) => id = Some(*obj_id),
                    UObject::Mesh(mesh_data) => mesh = Some(*mesh_data),
                    UObject::Transform(transform_data) => transform = Some(*transform_data),
                    _ => {} // 忽略 MaterialId 和 Tag，通过辅助函数提取
                }
            }
        }
        
        // 必须包含 ID 和 Mesh 才能创建实体
        if let (Some(entity_id), Some(mesh_data)) = (id, mesh) {
            let material_id = extract_material_id(unit).unwrap_or_default();
            let tag = extract_tag(unit);
            let bevy_transform = transform
                .map(|t| e2i_transform(t))
                .unwrap_or(Transform::default());
            
            self.ids.insert(entity_id, self.packs.len());
            let mut inpto = Inpto::new(mesh_data, material_id, bevy_transform);
            if let Some(tag_data) = tag {
                inpto.tag = Some(tag_data);
            }
            self.packs.push(inpto);
        }
    }
    
    /// 从 Unit 中提取 material_id（如果存在）
    fn extract_material_id(unit: &Unit) -> String {
        for obj in &unit.objects {
            if let Some(UObject::MaterialId(id)) = &obj.u_object {
                return id.clone();
            }
        }
        // 如果没有 material_id，返回空字符串，由渲染层根据策略选择
        String::new()
    }

    /// 从 Unit 中提取 Tag（如果存在）
    fn extract_tag(unit: &Unit) -> Option<expto::rdmp::Tag> {
        for obj in &unit.objects {
            if let Some(UObject::Tag(tag)) = &obj.u_object {
                return Some(tag.clone());
            }
        }
        None
    }

    pub fn react_update(&mut self, unit: &Unit) {
        // 基于类型识别提取更新信息
        let mut id: Option<u64> = None;
        let mut transform: Option<expto::rdmp::ExTransform> = None;
        let mut material_id: Option<String> = None;
        let mut tag: Option<expto::rdmp::Tag> = None;
        
        for obj in &unit.objects {
            if let Some(u_object) = &obj.u_object {
                match u_object {
                    UObject::Id(obj_id) => id = Some(*obj_id),
                    UObject::Transform(transform_data) => transform = Some(*transform_data),
                    UObject::MaterialId(mat_id) => material_id = Some(mat_id.clone()),
                    UObject::Tag(tag_data) => tag = Some(tag_data.clone()),
                    _ => {}
                }
            }
        }
        
        // 必须有 ID 才能更新
        if let Some(entity_id) = id {
            if let Some(idx) = self.ids.get(&entity_id) {
                // 更新变换（如果存在）
                if let Some(transform_data) = transform {
                    self.packs[*idx].transform = e2i_transform(transform_data);
                }
                
                // 更新材质（如果存在）
                if let Some(mat_id) = material_id {
                    self.packs[*idx].material = mat_id;
                }
                
                // 更新标签（如果存在）
                if let Some(tag_data) = tag {
                    self.packs[*idx].tag = Some(tag_data);
                }
            }
        }
    }

    pub fn react_destroy(&mut self, unit: &Unit) {
        // 提取要销毁的 ID
        if let Some(entity_id) = extract_id(unit) {
            if let Some(idx) = self.ids.get(&entity_id) {
                let idx = *idx;
                self.packs.remove(idx);
                
                // 重新构建索引映射
                self.rebuild_index_after_remove(idx);
            }
        }
    }
    
    /// 在删除元素后重建索引映射
    fn rebuild_index_after_remove(&mut self, removed_idx: usize) {
        for (_, idx) in self.ids.iter_mut() {
            if *idx > removed_idx {
                *idx -= 1;
            }
        }
    }

    // ==================== 数据访问接口（供 FrameRenderer 使用）====================

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.packs.len()
    }

    /// 迭代所有实体数据
    /// 返回 (entity_id, Inpto) 的迭代器
    pub fn iter_entities(&self) -> impl Iterator<Item = (u64, &Inpto)> {
        self.ids.iter().map(|(&id, &idx)| (id, &self.packs[idx]))
    }

    /// 根据业务ID获取实体数据
    pub fn get_entity(&self, entity_id: u64) -> Option<&Inpto> {
        self.ids.get(&entity_id).map(|&idx| &self.packs[idx])
    }
}

// ============================================================================
// 序列化支持
// ============================================================================

/// 可序列化的变换数据
#[derive(Serialize, Deserialize)]
struct SerializableTransform {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(s: SerializableTransform) -> Self {
        Self {
            translation: s.translation.into(),
            rotation: bevy::math::Quat::from_array(s.rotation),
            scale: s.scale.into(),
        }
    }
}

/// 可序列化的 Inpto
#[derive(Serialize, Deserialize)]
pub struct SerializableInpto {
    mesh: expto::rdmp::ExMesh,
    material: String,
    transform: SerializableTransform,
    tag: Option<expto::rdmp::Tag>,
}

impl From<&Inpto> for SerializableInpto {
    fn from(inpto: &Inpto) -> Self {
        Self {
            mesh: inpto.mesh,
            material: inpto.material.clone(),
            transform: inpto.transform.into(),
            tag: inpto.tag.clone(),
        }
    }
}

impl From<SerializableInpto> for Inpto {
    fn from(s: SerializableInpto) -> Self {
        Self {
            mesh: s.mesh,
            material: s.material,
            transform: s.transform.into(),
            tag: s.tag,
        }
    }
}

/// 可序列化的 KeyFrame
#[derive(Serialize, Deserialize)]
pub struct SerializableKeyFrame {
    pub timestamp: u64,
    pub entities: Vec<(u64, SerializableInpto)>,
}

impl From<&KeyFrame> for SerializableKeyFrame {
    fn from(kf: &KeyFrame) -> Self {
        let entities = kf.iter_entities()
            .map(|(id, inpto)| (id, SerializableInpto::from(inpto)))
            .collect();
        
        Self {
            timestamp: kf.timestamp,
            entities,
        }
    }
}

impl From<SerializableKeyFrame> for KeyFrame {
    fn from(s: SerializableKeyFrame) -> Self {
        let mut keyframe = KeyFrame::new(s.timestamp);
        
        for (id, serializable_inpto) in s.entities {
            let inpto = Inpto::from(serializable_inpto);
            keyframe.ids.insert(id, keyframe.packs.len());
            keyframe.packs.push(inpto);
        }
        
        keyframe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expto::rdmp::{ExMesh, ExTransform, Tag, ex_mesh::UMesh};
    use expto::rdmp::proto::mesh::{Sphere, Point};

    fn create_test_unit_with_tag(
        id: u64,
        position: [f32; 3],
        scale: [f32; 3],
        material: String,
        tag_text: String,
    ) -> Unit {
        let mut unit = Unit {
            stamp: None,
            command: None,
            objects: Vec::new(),
        };
        
        // 添加ID对象
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Id(id)),
        });
        
        // 设置网格对象
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Mesh(ExMesh {
                u_mesh: Some(UMesh::Sphere(Sphere { 
                    location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                    radius: 1.0 
                })),
            })),
        });

        // 设置变换对象
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Transform(ExTransform {
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
        });
        
        // 设置材质对象
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::MaterialId(material)),
        });

        // 设置标签对象
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Tag(Tag {
                text: tag_text,
                offset: None,
                style: None,
            })),
        });

        unit
    }

    #[test]
    fn test_parse_spawn_with_tag() {
        let mut keyframe = KeyFrame::new(0);
        
        // 创建包含5个对象的Unit（ID + Mesh + Transform + MaterialId + Tag）
        let unit = create_test_unit_with_tag(
            1,
            [1.0, 2.0, 3.0],
            [1.0, 1.0, 1.0],
            "red".to_string(),
            "测试标签".to_string(),
        );
        
        keyframe.react_spawn(&unit);
        
        // 验证实体已添加
        assert_eq!(keyframe.entity_count(), 1);
        
        // 验证实体数据
        let entities: Vec<_> = keyframe.iter_entities().collect();
        assert_eq!(entities.len(), 1);
        
        let (id, inpto) = entities[0];
        assert_eq!(id, 1);
        assert_eq!(inpto.material, "red");
        assert!((inpto.transform.translation.x - 1.0).abs() < f32::EPSILON);
        assert!((inpto.transform.translation.y - 2.0).abs() < f32::EPSILON);
        assert!((inpto.transform.translation.z - 3.0).abs() < f32::EPSILON);
        
        // 验证标签
        assert!(inpto.tag.is_some());
        let tag = inpto.tag.as_ref().unwrap();
        assert_eq!(tag.text, "测试标签");
    }

    #[test]
    fn test_parse_update_with_tag() {
        let mut keyframe = KeyFrame::new(0);
        
        // 先添加一个实体
        let spawn_unit = create_test_unit_with_tag(
            1,
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            "red".to_string(),
            "原始标签".to_string(),
        );
        keyframe.react_spawn(&spawn_unit);
        
        // 创建更新Unit（ID + Transform + Tag）
        let mut update_unit = Unit {
            stamp: None,
            command: None,
            objects: Vec::new(),
        };
        update_unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Id(1)),
        });
        update_unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Transform(ExTransform {
                x: 10.0,
                y: 20.0,
                z: 30.0,
                rx: 0.0,
                ry: 0.0,
                rz: 0.0,
                sx: 1.0,
                sy: 1.0,
                sz: 1.0,
            })),
        });
        update_unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Tag(Tag {
                text: "新标签".to_string(),
                offset: None,
                style: None,
            })),
        });
        
        keyframe.react_update(&update_unit);
        
        // 验证更新
        let entities: Vec<_> = keyframe.iter_entities().collect();
        let (_, inpto) = entities[0];
        
        assert!((inpto.transform.translation.x - 10.0).abs() < f32::EPSILON);
        assert!((inpto.transform.translation.y - 20.0).abs() < f32::EPSILON);
        assert!((inpto.transform.translation.z - 30.0).abs() < f32::EPSILON);
        assert_eq!(inpto.tag.as_ref().unwrap().text, "新标签");
    }

    #[test]
    fn test_parse_without_material_and_tag() {
        let mut keyframe = KeyFrame::new(0);
        
        // 创建只包含ID和Mesh的Unit
        let mut unit = Unit {
            stamp: None,
            command: None,
            objects: Vec::new(),
        };
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Id(42)),
        });
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Mesh(ExMesh {
                u_mesh: Some(UMesh::Sphere(Sphere { 
                    location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), 
                    radius: 1.0 
                })),
            })),
        });
        
        keyframe.react_spawn(&unit);
        
        // 验证实体已添加，材质为空字符串，标签为None
        assert_eq!(keyframe.entity_count(), 1);
        let entities: Vec<_> = keyframe.iter_entities().collect();
        let (_, inpto) = entities[0];
        assert_eq!(inpto.material, "");
        assert!(inpto.tag.is_none());
    }
}
