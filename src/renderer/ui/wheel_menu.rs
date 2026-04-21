//! 轮盘菜单模块 - 提供 Tab 键呼出的径向菜单功能
//! 
//! 此模块实现了基于 bevy_wheel_menu 的轮盘菜单系统，支持：
//! - Tab 键切换菜单显示/隐藏
//! - WASD/方向键导航
//! - Enter/Space 确认选择
//! - 手柄摇杆和按钮控制

use bevy::prelude::*;
use bevy_wheel_menu::*;

use crate::manager::interaction::font_manager::FontAssets;

/// 轮盘菜单插件
pub struct WheelMenuGraphPlugin;

impl Plugin for WheelMenuGraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(WheelMenuPlugin)
            .init_resource::<WheelMenuManager>()  // 使用新的管理器资源
            .add_systems(Update, (
                enforce_cursor_lock_policy,  // 优先执行：强制光标锁定策略
                toggle_wheel_menu,
                handle_wheel_select,
                update_wheel_visuals,
            ));
    }
}

/// 轮盘菜单状态（状态机）
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMenuState {
    /// 菜单隐藏状态
    #[default]
    Hidden,
    /// 菜单显示状态
    Visible,
}

impl WheelMenuState {
    /// 尝试切换到显示状态（需要光标未锁定）
    pub fn try_show(&mut self, cursor_locked: bool) -> bool {
        match self {
            WheelMenuState::Hidden if !cursor_locked => {
                *self = WheelMenuState::Visible;
                true
            }
            _ => false,
        }
    }

    /// 切换到隐藏状态
    pub fn hide(&mut self) -> bool {
        match self {
            WheelMenuState::Visible => {
                *self = WheelMenuState::Hidden;
                true
            }
            WheelMenuState::Hidden => false,
        }
    }

    /// 强制隐藏（用于光标锁定等异常情况）
    pub fn force_hide(&mut self) -> bool {
        match self {
            WheelMenuState::Visible => {
                *self = WheelMenuState::Hidden;
                true
            }
            WheelMenuState::Hidden => false,
        }
    }

    /// 检查是否可见
    pub fn is_visible(&self) -> bool {
        matches!(self, WheelMenuState::Visible)
    }
}

/// 轮盘菜单管理器资源（包含状态和实体引用）
#[derive(Resource, Default)]
pub struct WheelMenuManager {
    /// 当前状态
    pub state: WheelMenuState,
    /// 当前激活的轮盘菜单实体
    pub active_menu: Option<Entity>,
}

/// 标记轮盘菜单根实体
#[derive(Component)]
pub struct WheelMenuRoot;

/// 扇形视觉组件
#[derive(Component)]
pub struct SliceVisual {
    pub index: usize,
}

/// 图标文本组件
#[derive(Component)]
pub struct SliceIcon {
    pub index: usize,
}

/// 标签文本组件
#[derive(Component)]
pub struct SliceLabel {
    pub index: usize,
}

/// 中心圆组件
#[derive(Component)]
pub struct WheelCenter;

// 现代科技风格颜色主题
mod wheel_theme {
    use bevy::prelude::*;

    #[allow(dead_code)]
    pub const BACKGROUND: Color = Color::srgba(0.05, 0.05, 0.08, 0.95);
    #[allow(dead_code)]
    pub const SLICE_BASE: Color = Color::srgba(0.15, 0.18, 0.24, 0.90);
    #[allow(dead_code)]
    pub const SLICE_HOVER: Color = Color::srgba(0.25, 0.55, 0.85, 0.95);
    pub const TEXT_NORMAL: Color = Color::srgba(0.75, 0.80, 0.85, 1.0);
    pub const TEXT_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
    pub const ICON_NORMAL: Color = Color::srgba(0.65, 0.70, 0.75, 1.0);
    pub const ICON_HOVER: Color = Color::srgba(0.95, 0.90, 0.60, 1.0);
    #[allow(dead_code)]
    pub const CENTER_BG: Color = Color::srgba(0.08, 0.10, 0.14, 0.98);
}

/// 系统：强制执行光标锁定策略 - 如果光标被锁定且菜单可见，强制关闭菜单
pub fn enforce_cursor_lock_policy(
    mut commands: Commands,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut manager: ResMut<WheelMenuManager>,
    mut visibility_writer: MessageWriter<WheelMenuVisibilityChanged>,
    wheel_query: Query<Entity, With<WheelMenuRoot>>,
) {
    let cursor_locked = cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked;
    
    // 状态机转换：Visible + 光标锁定 → Hidden（强制）
    if cursor_locked && manager.state.is_visible() {
        manager.state.force_hide();
        manager.active_menu = None;
        
        // 销毁菜单实体
        for entity in wheel_query.iter() {
            commands.entity(entity).despawn();
        }
        
        visibility_writer.write(WheelMenuVisibilityChanged {
            visible: false,
            menu_entity: Entity::PLACEHOLDER,
        });
        
        info!("检测到光标锁定，强制关闭轮盘菜单");
    }
}

/// 系统：使用 Tab 键切换轮盘菜单的显示/隐藏
pub fn toggle_wheel_menu(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut manager: ResMut<WheelMenuManager>,
    mut visibility_writer: MessageWriter<WheelMenuVisibilityChanged>,
    wheel_query: Query<Entity, With<WheelMenuRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    font_assets: Option<Res<FontAssets>>,
) {
    // 检测 Tab 键按下
    if keyboard.just_pressed(KeyCode::Tab) {
        // 如果光标被锁定（FPS模式），不打开菜单，但如果菜单已打开则关闭它
        if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
            // 如果菜单当前是可见的，强制关闭它并恢复状态
            if manager.state.is_visible() {
                manager.state.force_hide();
                manager.active_menu = None;
                
                // 销毁菜单实体
                for entity in wheel_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: false,
                    menu_entity: Entity::PLACEHOLDER,
                });
                
                info!("检测到光标锁定，强制关闭轮盘菜单");
            }
            return;
        }

        // 光标未锁定，正常切换菜单状态
        let cursor_locked = cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked;
        let toggled = if manager.state.is_visible() {
            manager.state.hide();
            true
        } else {
            manager.state.try_show(cursor_locked)
        };
        
        if toggled {
            // 如果之前有菜单，先销毁
            for entity in wheel_query.iter() {
                commands.entity(entity).despawn();
            }
            
            if manager.state.is_visible() {
                // 创建新的轮盘菜单（如果有字体则使用自定义字体，否则使用默认）
                let menu_entity = match font_assets {
                    Some(font) => spawn_wheel_menu(&mut commands, &mut meshes, &mut mats, &font.bevy_font),
                    None => spawn_wheel_menu_default(&mut commands, &mut meshes, &mut mats),
                };
                manager.active_menu = Some(menu_entity);
                
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: true,
                    menu_entity,
                });
                
                info!("打开轮盘菜单");
            } else {
                manager.active_menu = None;
                
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: false,
                    menu_entity: Entity::PLACEHOLDER,
                });
                
                info!("关闭轮盘菜单");
            }
        }
    }
}

/// 生成轮盘菜单（带完整渲染，使用自定义字体）
fn spawn_wheel_menu(
    commands: &mut Commands, 
    meshes: &mut Assets<Mesh>, 
    mats: &mut Assets<ColorMaterial>,
    font_handle: &Handle<bevy::text::Font>,
) -> Entity {
    // 定义菜单项（示例：8 个选项，使用纯文本符号代替 Emoji）
    let menu_items = vec![
        ("选项 1", "[A]"),
        ("选项 2", "[B]"),
        ("选项 3", "[C]"),
        ("选项 4", "[D]"),
        ("选项 5", "[E]"),
        ("选项 6", "[F]"),
        ("选项 7", "[G]"),
        ("选项 8", "[H]"),
    ];
    
    let menu_config = WheelMenu {
        slices: menu_items.len(),
        radius: 150.0,
        inner_radius: 50.0,
        deadzone: 0.3,
        gap: 0.03,
    };
    
    let root = commands
        .spawn((
            WheelMenuRoot,
            menu_config.clone(),
            WheelState::default(),
            Transform::default(),
            Visibility::Visible,
        ))
        .id();
    
    // 为每个菜单项生成内容（包括扇形网格和文本）
    for (i, (label, icon)) in menu_items.iter().enumerate() {
        let center_pos = slice_center(&menu_config, i);
        
        // 生成扇形网格
        let (a0, a1) = slice_angles(&menu_config, i);
        let mesh_handle = meshes.add(bevy_wheel_menu::mesh::wedge(menu_config.inner_radius, menu_config.radius, a0, a1));
        let mat_handle = mats.add(ColorMaterial::from_color(wheel_theme::SLICE_BASE));
        
        // 扇形视觉组件标记
        let slice_entity = commands
            .spawn((
                WheelSlice { index: i },
                SliceVisual { index: i },
                Mesh2d(mesh_handle),
                MeshMaterial2d(mat_handle),
                Transform::from_translation(Vec3::Z * 0.0),
            ))
            .id();
        
        commands.entity(root).add_child(slice_entity);
        
        // 图标文本（使用自定义字体）
        let icon_entity = commands
            .spawn((
                SliceIcon { index: i },
                Text2d::new(icon.to_string()),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(wheel_theme::ICON_NORMAL),
                Transform::from_translation(Vec3::new(center_pos.x, center_pos.y + 12.0, 1.0)),
            ))
            .id();
        
        commands.entity(root).add_child(icon_entity);
        
        // 标签文本（使用自定义字体）
        let label_entity = commands
            .spawn((
                SliceLabel { index: i },
                Text2d::new(label.to_string()),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(wheel_theme::TEXT_NORMAL),
                Transform::from_translation(Vec3::new(center_pos.x, center_pos.y - 10.0, 1.0)),
            ))
            .id();
        
        commands.entity(root).add_child(label_entity);
    }
    
    // 中心圆背景
    let center_mesh = meshes.add(Circle::new(menu_config.inner_radius - 5.0));
    let center_mat = mats.add(ColorMaterial::from_color(wheel_theme::BACKGROUND));
    let center_entity = commands
        .spawn((
            Mesh2d(center_mesh),
            MeshMaterial2d(center_mat),
            Transform::from_translation(Vec3::Z * 2.0),
        ))
        .id();
    
    commands.entity(root).add_child(center_entity);
    
    // 中心提示文本（使用自定义字体）
    let center_text = commands
        .spawn((
            WheelCenter,
            Text2d::new("MENU"),
            TextFont {
                font: font_handle.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(wheel_theme::TEXT_HOVER),
            Transform::from_translation(Vec3::Z * 3.0),
        ))
        .id();
    
    commands.entity(root).add_child(center_text);
    
    root
}

/// 生成轮盘菜单（使用默认字体，当自定义字体加载失败时使用）
fn spawn_wheel_menu_default(
    commands: &mut Commands, 
    meshes: &mut Assets<Mesh>, 
    mats: &mut Assets<ColorMaterial>,
) -> Entity {
    // 定义菜单项（示例：8 个选项，使用纯文本符号代替 Emoji）
    let menu_items = vec![
        ("选项 1", "[A]"),
        ("选项 2", "[B]"),
        ("选项 3", "[C]"),
        ("选项 4", "[D]"),
        ("选项 5", "[E]"),
        ("选项 6", "[F]"),
        ("选项 7", "[G]"),
        ("选项 8", "[H]"),
    ];
    
    let menu_config = WheelMenu {
        slices: menu_items.len(),
        radius: 150.0,
        inner_radius: 50.0,
        deadzone: 0.3,
        gap: 0.03,
    };
    
    let root = commands
        .spawn((
            WheelMenuRoot,
            menu_config.clone(),
            WheelState::default(),
            Transform::default(),
            Visibility::Visible,
        ))
        .id();
    
    // 为每个菜单项生成内容（包括扇形网格和文本）
    for (i, (label, icon)) in menu_items.iter().enumerate() {
        let center_pos = slice_center(&menu_config, i);
        
        // 生成扇形网格
        let (a0, a1) = slice_angles(&menu_config, i);
        let mesh_handle = meshes.add(bevy_wheel_menu::mesh::wedge(menu_config.inner_radius, menu_config.radius, a0, a1));
        let mat_handle = mats.add(ColorMaterial::from_color(wheel_theme::SLICE_BASE));
        
        // 扇形视觉组件标记
        let slice_entity = commands
            .spawn((
                WheelSlice { index: i },
                SliceVisual { index: i },
                Mesh2d(mesh_handle),
                MeshMaterial2d(mat_handle),
                Transform::from_translation(Vec3::Z * 0.0),
            ))
            .id();
        
        commands.entity(root).add_child(slice_entity);
        
        // 图标文本（使用默认字体）
        let icon_entity = commands
            .spawn((
                SliceIcon { index: i },
                Text2d::new(icon.to_string()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(wheel_theme::ICON_NORMAL),
                Transform::from_translation(Vec3::new(center_pos.x, center_pos.y + 12.0, 1.0)),
            ))
            .id();
        
        commands.entity(root).add_child(icon_entity);
        
        // 标签文本（使用默认字体）
        let label_entity = commands
            .spawn((
                SliceLabel { index: i },
                Text2d::new(label.to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(wheel_theme::TEXT_NORMAL),
                Transform::from_translation(Vec3::new(center_pos.x, center_pos.y - 10.0, 1.0)),
            ))
            .id();
        
        commands.entity(root).add_child(label_entity);
    }
    
    // 中心圆背景
    let center_mesh = meshes.add(Circle::new(menu_config.inner_radius - 5.0));
    let center_mat = mats.add(ColorMaterial::from_color(wheel_theme::BACKGROUND));
    let center_entity = commands
        .spawn((
            Mesh2d(center_mesh),
            MeshMaterial2d(center_mat),
            Transform::from_translation(Vec3::Z * 2.0),
        ))
        .id();
    
    commands.entity(root).add_child(center_entity);
    
    // 中心提示文本（使用默认字体）
    let center_text = commands
        .spawn((
            WheelCenter,
            Text2d::new("MENU"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(wheel_theme::TEXT_HOVER),
            Transform::from_translation(Vec3::Z * 3.0),
        ))
        .id();
    
    commands.entity(root).add_child(center_text);
    
    root
}

/// 系统：更新轮盘菜单的视觉效果（悬停高亮）
pub fn update_wheel_visuals(
    mut hover_reader: MessageReader<WheelMenuHoverChanged>,
    menu_query: Query<(Entity, &WheelMenu, &WheelState), With<WheelMenuRoot>>,
    mut queries: ParamSet<(
        Query<(&SliceLabel, &mut TextColor, &mut TextFont)>,
        Query<(&SliceIcon, &mut TextColor)>,
    )>,
) {
    for event in hover_reader.read() {
        if let Ok((_menu_entity, _menu, state)) = menu_query.get(event.menu_entity) {
            // 重置所有扇区的样式
            for (_, mut text_color, mut text_font) in queries.p0().iter_mut() {
                *text_color = TextColor(wheel_theme::TEXT_NORMAL);
                text_font.font_size = 12.0;
            }
            
            for (_, mut icon_color) in queries.p1().iter_mut() {
                *icon_color = TextColor(wheel_theme::ICON_NORMAL);
            }
            
            // 高亮当前悬停的扇区
            if let Some(hovered_idx) = state.hovered {
                // 高亮标签
                for (label, mut text_color, mut text_font) in queries.p0().iter_mut() {
                    if label.index == hovered_idx {
                        *text_color = TextColor(wheel_theme::TEXT_HOVER);
                        text_font.font_size = 14.0; // 稍微放大
                    }
                }
                
                // 高亮图标
                for (icon, mut icon_color) in queries.p1().iter_mut() {
                    if icon.index == hovered_idx {
                        *icon_color = TextColor(wheel_theme::ICON_HOVER);
                    }
                }
            }
        }
    }
}

/// 系统：处理轮盘菜单选择事件
pub fn handle_wheel_select(
    mut commands: Commands,
    mut reader: MessageReader<WheelMenuSelected>,
    mut manager: ResMut<WheelMenuManager>,
    wheel_query: Query<Entity, With<WheelMenuRoot>>,
) {
    for event in reader.read() {
        info!("选择了菜单项 {} (菜单实体：{:?})", event.index, event.menu_entity);
        
        // 这里可以添加具体的选择逻辑
        // 例如：发送网络消息、执行动作等
        
        // 选择后自动关闭菜单
        manager.state.hide();
        manager.active_menu = None;
        
        // 销毁菜单
        for entity in wheel_query.iter() {
            commands.entity(entity).despawn();
        }
        
        info!("轮盘菜单已关闭");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_angles_basic() {
        let menu = WheelMenu {
            slices: 8,
            radius: 150.0,
            inner_radius: 50.0,
            deadzone: 0.3,
            gap: 0.03,
        };

        // 测试所有扇形都有有效的角度范围
        for i in 0..menu.slices {
            let (start, end) = slice_angles(&menu, i);
            assert!(start >= 0.0, "Start angle should be non-negative");
            assert!(end <= std::f32::consts::TAU, "End angle should not exceed full circle");
            assert!(end > start, "Each slice should have positive angle range");
        }
    }
    
    #[test]
    fn test_menu_configuration() {
        let menu = WheelMenu::default();
        assert_eq!(menu.slices, 8);
        assert_eq!(menu.radius, 120.0);
        assert_eq!(menu.inner_radius, 40.0);
    }
}