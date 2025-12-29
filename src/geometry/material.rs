use bevy::prelude::*;
use std::collections::HashMap;

/// 预定义的材质类型
#[derive(Debug, Clone)]
pub enum PredefinedMaterial {
    Color(Color),
    Standard(StandardMaterial),
}

/// 材质管理器
pub struct MaterialManager {
    predefined_materials: HashMap<String, PredefinedMaterial>,
}

impl MaterialManager {
    pub fn new() -> Self {
        let mut manager = MaterialManager {
            predefined_materials: HashMap::new(),
        };
        
        // 注册一些默认材质
        manager.register_default_materials();
        manager
    }

    /// 注册默认材质
    fn register_default_materials(&mut self) {
        // 基础颜色材质
        self.predefined_materials.insert(
            "default".to_string(),
            PredefinedMaterial::Color(Color::srgb(0.8, 0.7, 0.6))
        );
        self.predefined_materials.insert(
            "red".to_string(),
            PredefinedMaterial::Color(Color::srgb(0.8, 0.1, 0.1))
        );
        self.predefined_materials.insert(
            "green".to_string(),
            PredefinedMaterial::Color(Color::srgb(0.1, 0.8, 0.2))
        );
        self.predefined_materials.insert(
            "blue".to_string(),
            PredefinedMaterial::Color(Color::srgb(0.1, 0.3, 0.8))
        );
        self.predefined_materials.insert(
            "yellow".to_string(),
            PredefinedMaterial::Color(Color::srgb(0.9, 0.9, 0.2))
        );
        self.predefined_materials.insert(
            "white".to_string(),
            PredefinedMaterial::Color(Color::WHITE)
        );
        self.predefined_materials.insert(
            "black".to_string(),
            PredefinedMaterial::Color(Color::BLACK)
        );
        
        // 特殊效果材质
        self.predefined_materials.insert(
            "emissive".to_string(),
            PredefinedMaterial::Standard(StandardMaterial {
                base_color: Color::srgb(0.1, 0.8, 0.5),
                emissive: LinearRgba::from(Color::srgb(0.1, 0.8, 0.5)) * 3.0,
                ..default()
            })
        );
        self.predefined_materials.insert(
            "shiny".to_string(),
            PredefinedMaterial::Standard(StandardMaterial {
                base_color: Color::srgb(0.7, 0.8, 0.9),
                metallic: 0.9,
                perceptual_roughness: 0.1,
                ..default()
            })
        );
        self.predefined_materials.insert(
            "transparent".to_string(),
            PredefinedMaterial::Standard(StandardMaterial {
                base_color: Color::srgba(0.5, 0.5, 0.9, 0.5),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })
        );
    }

    /// 注册新的预定义材质
    pub fn register_material(&mut self, name: &str, material: PredefinedMaterial) {
        self.predefined_materials.insert(name.to_string(), material);
    }

    /// 获取材质的句柄，如果不存在则创建默认材质
    pub fn get_or_insert_material(
        &self,
        name: &str,
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) -> Handle<StandardMaterial> {
        if let Some(predefined) = self.predefined_materials.get(name) {
            match predefined {
                PredefinedMaterial::Color(color) => {
                    materials.add(StandardMaterial::from(*color))
                },
                PredefinedMaterial::Standard(material) => {
                    materials.add(material.clone())
                }
            }
        } else {
            // 如果没有找到预定义材质，则使用默认材质
            materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6)))
        }
    }

    /// 从外部资源加载材质（例如从文件）
    pub fn load_material_from_config(&mut self, name: &str, config: &str) -> Result<(), String> {
        // 这里可以实现从配置字符串加载材质的逻辑
        // 例如解析JSON格式的材质配置
        if let Ok(color) = parse_color(config) {
            self.predefined_materials.insert(
                name.to_string(),
                PredefinedMaterial::Color(color)
            );
            Ok(())
        } else {
            Err("无法解析材质配置".to_string())
        }
    }
}

/// 解析颜色配置的辅助函数
fn parse_color(config: &str) -> Result<Color, ()> {
    // 简单的颜色解析实现，可以扩展支持更多格式
    match config.to_lowercase().as_str() {
        "red" => Ok(Color::srgb(1.0, 0.0, 0.0)),
        "green" => Ok(Color::srgb(0.0, 1.0, 0.0)),
        "blue" => Ok(Color::srgb(0.0, 0.0, 1.0)),
        "white" => Ok(Color::WHITE),
        "black" => Ok(Color::BLACK),
        _ => {
            // 尝试解析十六进制颜色
            if config.starts_with('#') && config.len() == 7 {
                let hex = &config[1..];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16)
                ) {
                    return Ok(Color::srgb(
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0
                    ));
                }
            }
            Err(())
        }
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}