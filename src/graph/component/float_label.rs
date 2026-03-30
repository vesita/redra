use bevy::{prelude::*, sprite::Sprite};

/// 3D 浮动标签组件
/// 
/// 此组件用于在 3D 空间中显示文本，并使文本始终面向摄像头
/// 适用于标注、说明文字等场景
#[derive(Component, Clone)]
pub struct FloatLabel {
    /// 要显示的文本内容
    pub text: String,
    /// 字体大小（像素）
    pub font_size: f32,
    /// 文本颜色
    pub color: Color,
    /// 是否启用背景板
    pub with_background: bool,
    /// 背景颜色（如果启用背景）
    pub background_color: Color,
}

impl Default for FloatLabel {
    fn default() -> Self {
        Self {
            text: String::from("Label"),
            font_size: 48.0,
            color: Color::WHITE,
            with_background: false,
            background_color: Color::BLACK.with_alpha(0.7),
        }
    }
}

impl FloatLabel {
    /// 创建一个新的浮动标签
    /// 
    /// # 参数
    /// * `text` - 要显示的文本内容
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// 设置字体大小
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// 设置文本颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 启用背景
    pub fn with_background(mut self, color: Color) -> Self {
        self.with_background = true;
        self.background_color = color;
        self
    }
}

/// 标记组件，用于标识这是浮动标签的图形实体
#[derive(Component)]
pub struct FloatLabelGraphics;


/// 标记组件，用于标识相机用于 billboard 计算
#[derive(Component)]
pub struct MainCamera3d;

/// 系统：生成浮动标签的图形表示
/// 
/// 当检测到新的 FloatLabel 组件时，为其创建对应的 Sprite 精灵
/// 该精灵将自动面向摄像头
pub fn spawn_float_label_graphics(
    mut commands: Commands,
    labels: Query<(Entity, &FloatLabel), Without<FloatLabelGraphics>>,
) {
    for (entity, label) in labels.iter() {
        // 为标签实体添加子实体用于渲染
        commands.entity(entity).with_children(|parent| {
            // 创建一个 Sprite 作为占位符
            // 注意：实际文本渲染需要字体支持，这里使用简单的彩色矩形
            // 在实际项目中，建议使用 bevy 的 TextBundle 或集成字体渲染库
            
            let size = calculate_text_size(&label.text, label.font_size);
            
            parent.spawn((
                Sprite {
                    custom_size: Some(size),
                    color: if label.with_background {
                        label.background_color
                    } else {
                        Color::NONE
                    },
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                FloatLabelGraphics,
            ));
        });
    }
}

/// 系统：更新浮动标签使其始终面向摄像头
/// 
/// 每帧更新所有浮动标签的旋转，使其 Z 轴指向摄像头
pub fn update_float_label_facing_camera(
    mut query: Query<&mut Transform, (With<FloatLabelGraphics>, Without<MainCamera3d>)>,
    camera_query: Query<&Transform, With<MainCamera3d>>,
) {
    // 获取主相机的位置和方向
    if let Ok(camera_transform) = camera_query.single() {
        let camera_position = camera_transform.translation;
        
        for mut label_transform in query.iter_mut() {
            // 让标签始终面向摄像头
            let label_position = label_transform.translation;
            
            // 计算从标签指向摄像头的方向
            let direction = (camera_position - label_position).try_normalize();
            
            if let Some(dir) = direction {
                // 创建旋转四元数，使标签面向摄像头
                // 假设文本的正面是 +Z 方向
                let target_rotation = Quat::from_rotation_arc(Vec3::Z, dir);
                
                // 应用旋转（保持位置和缩放不变）
                label_transform.rotation = target_rotation;
            }
        }
    }
}

/// 辅助函数：计算文本尺寸
fn calculate_text_size(text: &str, font_size: f32) -> Vec2 {
    let aspect_ratio = text.len() as f32 * 0.6;
    Vec2::new(aspect_ratio * font_size / 100.0, font_size / 50.0)
}

/// 插件：注册所有浮动标签相关的系统
pub struct FloatLabelPlugin;

impl Plugin for FloatLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_float_label_graphics,
                update_float_label_facing_camera,
            ),
        );
    }
}

/// 辅助函数：在世界坐标处生成一个浮动标签
/// 
/// # 参数
/// * `commands` - Bevy 命令处理器
/// * `position` - 标签在 3D 空间中的位置
/// * `text` - 要显示的文本
/// * `label_config` - 标签配置（可选）
/// 
/// # 返回
/// 生成的实体 ID
pub fn spawn_label_at(
    commands: &mut Commands,
    position: Vec3,
    text: impl Into<String>,
    label_config: Option<FloatLabel>,
) -> Entity {
    let label = label_config.unwrap_or_else(|| FloatLabel::new(text));
    
    commands
        .spawn((
            Transform::from_translation(position),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            label,
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_label_creation() {
        let label = FloatLabel::new("Test Label");
        assert_eq!(label.text, "Test Label");
        assert_eq!(label.font_size, 48.0);
    }

    #[test]
    fn test_float_label_builder_pattern() {
        let label = FloatLabel::new("Custom")
            .with_font_size(32.0)
            .with_color(Color::srgb(1.0, 0.0, 0.0))
            .with_background(Color::srgba(0.0, 0.0, 1.0, 1.0));
        
        assert_eq!(label.text, "Custom");
        assert_eq!(label.font_size, 32.0);
        assert!(label.with_background);
    }
}
