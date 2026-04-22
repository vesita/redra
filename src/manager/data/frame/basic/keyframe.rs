use std::collections::HashMap;

use bevy::prelude::*;
use expto::rdmp::{CommandType, Unit, ex_object::UObject};

use crate::manager::{data::frame::{Inpto, KeyFrame}, data_flow::parser::{e2i_transform, parse_command, parse_object}};


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
            CommandType::Unknown => todo!(),
            CommandType::Spawn => {
                self.react_spawn(unit);
            },
            CommandType::Update => {
                self.react_update(unit);
            },
            CommandType::Destroy => {
                self.react_destroy(unit);
            },
            CommandType::Frameend => todo!(),
        }
    }

    pub fn react_spawn(&mut self, unit: &Unit) {
        let objects = parse_object(unit);
        match objects.len() {
            2 => {
                // Id + Mesh（无变换和材质，使用默认值）
                match (&objects[0], &objects[1]) {
                    (UObject::Id(id), UObject::Mesh(mesh)) => {
                        let material_id = Self::extract_material_id(unit);
                        
                        self.ids.insert(*id, self.packs.len());
                        self.packs.push(Inpto::new(mesh.clone(), material_id, Transform::default()));
                    },
                    _ => {}
                }
            },
            3 => {
                // Id + Mesh + Transform（带变换，材质从Unit中提取）
                match (&objects[0], &objects[1], &objects[2]) {
                    (UObject::Id(id), UObject::Mesh(mesh), UObject::Transform(transform)) => {
                        let material_id = Self::extract_material_id(unit);
                        let bevy_transform = e2i_transform(transform.clone());
                        
                        self.ids.insert(*id, self.packs.len());
                        self.packs.push(Inpto::new(mesh.clone(), material_id, bevy_transform));
                    },
                    _ => {}
                }
            },
            4 => {
                // Id + Mesh + Transform + MaterialId（完整信息）
                match (&objects[0], &objects[1], &objects[2], &objects[3]) {
                    (UObject::Id(id), UObject::Mesh(mesh), UObject::Transform(transform), UObject::MaterialId(material_id)) => {
                        let bevy_transform = e2i_transform(transform.clone());
                        
                        self.ids.insert(*id, self.packs.len());
                        self.packs.push(Inpto::new(mesh.clone(), material_id.clone(), bevy_transform));
                    },
                    _ => {}
                }
            },
            _ => {}
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

    pub fn react_update(&mut self, unit: &Unit) {
        let objects = parse_object(unit);
        match objects.len() {
            2 => {
                match (&objects[0], &objects[1]) {
                    (UObject::Id(id), UObject::Transform(transform)) => {
                        if let Some(idx) = self.ids.get(id) {
                            self.packs[*idx].transform = e2i_transform(transform.clone());
                        }
                    },
                    _ => {

                    }
                }
            },
            3 => {
                match (&objects[0], &objects[1], &objects[2]) {
                    (UObject::Id(id), UObject::MaterialId(material_id), UObject::Transform(transform)) => {
                        if let Some(idx) = self.ids.get(id) {
                            self.packs[*idx].material = material_id.clone();
                            self.packs[*idx].transform = e2i_transform(transform.clone());
                        }
                    },
                    _ => {
                    }
                }
            },
            _ => {

            }
        }
    }

    pub fn react_destroy(&mut self, unit: &Unit) {
        let objects = parse_object(unit);
        if objects.len() == 1 {
            match &objects[0] {
                UObject::Id(id) => {
                    if let Some(idx) = self.ids.get(id) {
                        self.packs.remove(*idx);
                    }
                },
                _ => {
                }
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
}

