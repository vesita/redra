use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use expto::rdmp::{CommandType, ExMesh, ExTransform, Tag, Unit, ex_object::UObject};
use serde::{Serialize, Deserialize};

use crate::data::protocol::{e2i_transform, parse_command, extract_id, extract_material_id, extract_tag};
use crate::data::frame::Inpto;

/// 生成一个单调递增的实体 ID（基于时间戳和当前 pack 数量）
fn generate_entity_id(packs_len: usize) -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ts ^ (packs_len as u64).rotate_left(32)
}

/// 关键帧 — 某一时刻的场景快照
pub struct KeyFrame {
    pub timestamp: u64,
    pub ids: HashMap<u64, usize>,
    pub packs: Vec<Inpto>,
}

impl KeyFrame {
    pub fn new(timestamp: u64) -> Self {
        Self { timestamp, ids: HashMap::new(), packs: Vec::new() }
    }

    pub fn update(&mut self, unit: &Unit) {
        let command = parse_command(unit).unwrap_or(CommandType::Spawn);
        self.match_command(&command, unit);
    }

    fn match_command(&mut self, command: &CommandType, unit: &Unit) {
        match command {
            CommandType::Unknown => {}
            CommandType::Spawn => self.react_spawn(unit),
            CommandType::Update => self.react_update(unit),
            CommandType::Destroy => self.react_destroy(unit),
            CommandType::Frameend => {}
        }
    }

    /// 将 Unit 对象列表按 Id 边界分组，每组生成一个实体
    fn react_spawn(&mut self, unit: &Unit) {
        // 检测是否有显式 Id 对象：有则按 Id 分组（batch 模式），无则按 legacy 单实体处理
        let has_ids = unit.objects.iter().any(|obj| {
            obj.u_object.as_ref().is_some_and(|u| matches!(u, UObject::Id(_)))
        });

        if !has_ids {
            // Legacy 模式：所有对象属于同一个实体，自动生成 ID
            return self.spawn_legacy(unit);
        }

        // Batch 模式：按 Id 对象切分，每组独立生成一个实体
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for (i, obj) in unit.objects.iter().enumerate() {
            let is_id = obj.u_object.as_ref().is_some_and(|u| matches!(u, UObject::Id(_)));
            if is_id && !groups.is_empty() {
                groups.push(vec![i]);
            } else if groups.is_empty() {
                groups.push(vec![i]);
            } else {
                groups.last_mut().unwrap().push(i);
            }
        }

        for indices in &groups {
            let mut entity_id: Option<u64> = None;
            let mut mesh: Option<ExMesh> = None;
            let mut transform: Option<ExTransform> = None;
            let mut material: Option<String> = None;
            let mut tag: Option<Tag> = None;

            for &idx in indices {
                if let Some(u_object) = &unit.objects[idx].u_object {
                    match u_object {
                        UObject::Id(id) => entity_id = Some(*id),
                        UObject::Mesh(m) => mesh = Some(m.clone()),
                        UObject::Transform(t) => transform = Some(*t),
                        UObject::MaterialId(m) => material = Some(m.clone()),
                        UObject::Tag(t) => tag = Some(t.clone()),
                    }
                }
            }

            let id = entity_id.unwrap_or_else(|| generate_entity_id(self.packs.len()));
            if let Some(m) = mesh {
                let bevy_t = transform.map(|t| e2i_transform(t)).unwrap_or_default();
                let mat = material.unwrap_or_default();
                self.ids.insert(id, self.packs.len());
                let mut inpto = Inpto::new(m, mat, bevy_t);
                if let Some(tag_data) = tag {
                    inpto.tag = Some(tag_data);
                }
                self.packs.push(inpto);
            }
        }
    }

    /// Legacy 模式：Unit 中无 Id 对象时，自动生成 ID，收集所有对象属性创建单个实体
    fn spawn_legacy(&mut self, unit: &Unit) {
        let mut mesh: Option<ExMesh> = None;
        let mut transform: Option<ExTransform> = None;

        for obj in &unit.objects {
            if let Some(u_object) = &obj.u_object {
                match u_object {
                    UObject::Mesh(mesh_data) => mesh = Some(mesh_data.clone()),
                    UObject::Transform(transform_data) => transform = Some(*transform_data),
                    _ => {}
                }
            }
        }

        if let Some(mesh_data) = mesh {
            let entity_id = generate_entity_id(self.packs.len());
            let material_id = extract_material_id(unit).unwrap_or_default();
            let tag = extract_tag(unit);
            let bevy_transform = transform.map(|t| e2i_transform(t)).unwrap_or(Transform::default());

            self.ids.insert(entity_id, self.packs.len());
            let mut inpto = Inpto::new(mesh_data, material_id, bevy_transform);
            if let Some(tag_data) = tag {
                inpto.tag = Some(tag_data);
            }
            self.packs.push(inpto);
        }
    }

    fn react_update(&mut self, unit: &Unit) {
        let mut id: Option<u64> = None;
        let mut transform: Option<ExTransform> = None;
        let mut material_id: Option<String> = None;
        let mut tag: Option<Tag> = None;

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

        if let Some(entity_id) = id {
            if let Some(idx) = self.ids.get(&entity_id) {
                if let Some(transform_data) = transform {
                    self.packs[*idx].transform = e2i_transform(transform_data);
                }
                if let Some(mat_id) = material_id {
                    self.packs[*idx].material = mat_id;
                }
                if let Some(tag_data) = tag {
                    self.packs[*idx].tag = Some(tag_data);
                }
            }
        }
    }

    fn react_destroy(&mut self, unit: &Unit) {
        if let Some(entity_id) = extract_id(unit) {
            if let Some(idx) = self.ids.get(&entity_id) {
                let idx = *idx;
                self.packs.remove(idx);
                self.rebuild_index_after_remove(idx);
            }
        }
    }

    fn rebuild_index_after_remove(&mut self, removed_idx: usize) {
        for (_, idx) in self.ids.iter_mut() {
            if *idx > removed_idx {
                *idx -= 1;
            }
        }
    }

    // ==================== 数据访问接口 ====================

    pub fn entity_count(&self) -> usize { self.packs.len() }

    pub fn iter_entities(&self) -> impl Iterator<Item = (u64, &Inpto)> {
        self.ids.iter().map(|(&id, &idx)| (id, &self.packs[idx]))
    }

    pub fn get_entity(&self, entity_id: u64) -> Option<&Inpto> {
        self.ids.get(&entity_id).map(|&idx| &self.packs[idx])
    }
}

// ============================================================================
// 序列化（轻量级文件 I/O 格式）
// ============================================================================

#[derive(Serialize, Deserialize)]
struct SerializableTransform {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(t: Transform) -> Self {
        Self { translation: t.translation.into(), rotation: t.rotation.into(), scale: t.scale.into() }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(s: SerializableTransform) -> Self {
        Self { translation: s.translation.into(), rotation: Quat::from_array(s.rotation), scale: s.scale.into() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableInpto {
    mesh: ExMesh,
    material: String,
    transform: SerializableTransform,
    tag: Option<Tag>,
}

impl From<&Inpto> for SerializableInpto {
    fn from(inpto: &Inpto) -> Self {
        Self { mesh: inpto.mesh.clone(), material: inpto.material.clone(), transform: inpto.transform.into(), tag: inpto.tag.clone() }
    }
}

impl From<SerializableInpto> for Inpto {
    fn from(s: SerializableInpto) -> Self {
        Self { mesh: s.mesh, material: s.material, transform: s.transform.into(), tag: s.tag }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableKeyFrame {
    pub timestamp: u64,
    pub entities: Vec<(u64, SerializableInpto)>,
}

impl From<&KeyFrame> for SerializableKeyFrame {
    fn from(kf: &KeyFrame) -> Self {
        let entities = kf.iter_entities().map(|(id, inpto)| (id, SerializableInpto::from(inpto))).collect();
        Self { timestamp: kf.timestamp, entities }
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
    use expto::rdmp::mesh::ex_mesh::UMesh;
    use expto::rdmp::proto::mesh::{Sphere, Point};

    fn create_test_unit_with_tag(
        id: u64, position: [f32; 3], scale: [f32; 3], material: String, tag_text: String,
    ) -> Unit {
        let mut unit = Unit { stamp: None, command: None, objects: Vec::new() };
        unit.objects.push(expto::rdmp::ExObject { u_object: Some(UObject::Id(id)) });
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Mesh(ExMesh {
                u_mesh: Some(UMesh::Sphere(Sphere { location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), radius: 1.0 })),
            })),
        });
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Transform(ExTransform {
                x: position[0], y: position[1], z: position[2],
                rx: 0.0, ry: 0.0, rz: 0.0,
                sx: scale[0], sy: scale[1], sz: scale[2],
            })),
        });
        unit.objects.push(expto::rdmp::ExObject { u_object: Some(UObject::MaterialId(material)) });
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Tag(Tag { text: tag_text, offset: None, style: None })),
        });
        unit
    }

    #[test]
    fn test_parse_spawn_with_tag() {
        let mut keyframe = KeyFrame::new(0);
        let unit = create_test_unit_with_tag(1, [1.0, 2.0, 3.0], [1.0, 1.0, 1.0], "red".to_string(), "测试标签".to_string());
        keyframe.react_spawn(&unit);
        assert_eq!(keyframe.entity_count(), 1);
        let (id, inpto) = keyframe.iter_entities().next().unwrap();
        assert_eq!(id, 1);
        assert_eq!(inpto.material, "red");
        assert!(inpto.tag.is_some());
        assert_eq!(inpto.tag.as_ref().unwrap().text, "测试标签");
    }

    #[test]
    fn test_parse_update_with_tag() {
        let mut keyframe = KeyFrame::new(0);
        let spawn_unit = create_test_unit_with_tag(1, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], "red".to_string(), "原始标签".to_string());
        keyframe.react_spawn(&spawn_unit);

        let mut update_unit = Unit { stamp: None, command: None, objects: Vec::new() };
        update_unit.objects.push(expto::rdmp::ExObject { u_object: Some(UObject::Id(1)) });
        update_unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Transform(ExTransform { x: 10.0, y: 20.0, z: 30.0, rx: 0.0, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 })),
        });
        update_unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Tag(Tag { text: "新标签".to_string(), offset: None, style: None })),
        });
        keyframe.react_update(&update_unit);
        let (_, inpto) = keyframe.iter_entities().next().unwrap();
        assert!((inpto.transform.translation.x - 10.0).abs() < f32::EPSILON);
        assert_eq!(inpto.tag.as_ref().unwrap().text, "新标签");
    }

    #[test]
    fn test_parse_without_material_and_tag() {
        let mut keyframe = KeyFrame::new(0);
        let mut unit = Unit { stamp: None, command: None, objects: Vec::new() };
        unit.objects.push(expto::rdmp::ExObject { u_object: Some(UObject::Id(42)) });
        unit.objects.push(expto::rdmp::ExObject {
            u_object: Some(UObject::Mesh(ExMesh {
                u_mesh: Some(UMesh::Sphere(Sphere { location: Some(Point { x: 0.0, y: 0.0, z: 0.0 }), radius: 1.0 })),
            })),
        });
        keyframe.react_spawn(&unit);
        assert_eq!(keyframe.entity_count(), 1);
        let (_, inpto) = keyframe.iter_entities().next().unwrap();
        assert_eq!(inpto.material, "");
        assert!(inpto.tag.is_none());
    }
}
