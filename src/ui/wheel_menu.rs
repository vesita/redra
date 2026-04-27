use bevy::prelude::*;
use bevy_wheel_menu::*;

use crate::assets::fonts::FontAssets;

pub struct WheelMenuGraphPlugin;

impl Plugin for WheelMenuGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WheelMenuPlugin)
            .init_resource::<WheelMenuManager>()
            .add_systems(
                Update,
                (
                    enforce_cursor_lock_policy,
                    toggle_wheel_menu,
                    handle_wheel_select,
                    update_wheel_visuals,
                ),
            );
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMenuState {
    #[default]
    Hidden,
    Visible,
}

impl WheelMenuState {
    pub fn try_show(&mut self, cursor_locked: bool) -> bool {
        match self {
            WheelMenuState::Hidden if !cursor_locked => {
                *self = WheelMenuState::Visible;
                true
            }
            _ => false,
        }
    }
    pub fn hide(&mut self) -> bool {
        match self {
            WheelMenuState::Visible => {
                *self = WheelMenuState::Hidden;
                true
            }
            _ => false,
        }
    }
    pub fn force_hide(&mut self) -> bool {
        match self {
            WheelMenuState::Visible => {
                *self = WheelMenuState::Hidden;
                true
            }
            _ => false,
        }
    }
    pub fn is_visible(&self) -> bool {
        matches!(self, WheelMenuState::Visible)
    }
}

#[derive(Resource, Default)]
pub struct WheelMenuManager {
    pub state: WheelMenuState,
    pub active_menu: Option<Entity>,
}

#[derive(Component)]
pub struct WheelMenuRoot;

#[derive(Component)]
pub struct SliceVisual {
    pub index: usize,
}
#[derive(Component)]
pub struct SliceIcon {
    pub index: usize,
}
#[derive(Component)]
pub struct SliceLabel {
    pub index: usize,
}
#[derive(Component)]
pub struct WheelCenter;

/// VS Code 暗色主题色系
mod wheel_theme {
    use bevy::prelude::*;
    pub const SLICE_BASE: Color = Color::srgba(0.20, 0.22, 0.26, 0.92);
    pub const TEXT_NORMAL: Color = Color::srgba(0.75, 0.78, 0.82, 1.0);
    pub const TEXT_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
    pub const ICON_NORMAL: Color = Color::srgba(0.60, 0.64, 0.70, 1.0);
    pub const ICON_HOVER: Color = Color::srgba(0.95, 0.95, 0.50, 1.0);
    pub const CENTER_BG: Color = Color::srgba(0.08, 0.08, 0.10, 0.98);
    pub const ACCENT: Color = Color::srgba(0.00, 0.48, 0.80, 1.0);
}

/// 菜单项定义：(显示名, 图标, 描述)
const MENU_ITEMS: &[(&str, &str, &str)] = &[
    ("选中",   "⇱", "切换选取模式"),
    ("平移",   "✥", "平移视口"),
    ("旋转",   "⟳", "旋转视角"),
    ("缩放",   "⊕", "缩放视图"),
    ("重置",   "⌂", "重置视角"),
    ("俯视",   "◉", "俯视视图"),
    ("侧视",   "◧", "侧视视图"),
    ("前视",   "◨", "前视视图"),
];

pub fn enforce_cursor_lock_policy(
    mut commands: Commands,
    cursor_options: bevy::prelude::Single<&bevy::window::CursorOptions>,
    mut manager: ResMut<WheelMenuManager>,
    mut visibility_writer: MessageWriter<WheelMenuVisibilityChanged>,
    wheel_query: Query<Entity, With<WheelMenuRoot>>,
) {
    let cursor_locked = cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked;
    if cursor_locked && manager.state.is_visible() {
        manager.state.force_hide();
        manager.active_menu = None;
        for entity in wheel_query.iter() {
            commands.entity(entity).despawn();
        }
        visibility_writer.write(WheelMenuVisibilityChanged {
            visible: false,
            menu_entity: Entity::PLACEHOLDER,
        });
        info!("检测到光标锁定，强制关闭快捷菜单");
    }
}

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
    if keyboard.just_pressed(KeyCode::Tab) {
        if cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
            if manager.state.is_visible() {
                manager.state.force_hide();
                manager.active_menu = None;
                for entity in wheel_query.iter() {
                    commands.entity(entity).despawn();
                }
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: false,
                    menu_entity: Entity::PLACEHOLDER,
                });
            }
            return;
        }
        let cursor_locked = cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked;
        let toggled = if manager.state.is_visible() {
            manager.state.hide();
            true
        } else {
            manager.state.try_show(cursor_locked)
        };
        if toggled {
            for entity in wheel_query.iter() {
                commands.entity(entity).despawn();
            }
            if manager.state.is_visible() {
                let menu_entity = match font_assets {
                    Some(font) => spawn_wheel_menu(&mut commands, &mut meshes, &mut mats, &font.bevy_font),
                    None => spawn_wheel_menu_default(&mut commands, &mut meshes, &mut mats),
                };
                manager.active_menu = Some(menu_entity);
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: true,
                    menu_entity,
                });
                info!("打开快捷菜单");
            } else {
                manager.active_menu = None;
                visibility_writer.write(WheelMenuVisibilityChanged {
                    visible: false,
                    menu_entity: Entity::PLACEHOLDER,
                });
                info!("关闭快捷菜单");
            }
        }
    }
}

fn spawn_wheel_menu(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<ColorMaterial>,
    font_handle: &Handle<bevy::text::Font>,
) -> Entity {
    spawn_wheel_menu_inner(commands, meshes, mats, Some(font_handle))
}

fn spawn_wheel_menu_default(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<ColorMaterial>,
) -> Entity {
    spawn_wheel_menu_inner(commands, meshes, mats, None)
}

fn spawn_wheel_menu_inner(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<ColorMaterial>,
    font: Option<&Handle<bevy::text::Font>>,
) -> Entity {
    let slice_count = MENU_ITEMS.len();
    let menu_config = WheelMenu {
        slices: slice_count,
        radius: 160.0,
        inner_radius: 50.0,
        deadzone: 0.25,
        gap: 0.025,
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

    let mut children: Vec<Entity> = Vec::new();

    for (i, (label, icon, _desc)) in MENU_ITEMS.iter().enumerate() {
        let center_pos = slice_center(&menu_config, i);
        let (a0, a1) = slice_angles(&menu_config, i);

        // 扇形片
        let child = commands
            .spawn((
                WheelSlice { index: i },
                SliceVisual { index: i },
                Mesh2d(meshes.add(bevy_wheel_menu::mesh::wedge(
                    menu_config.inner_radius,
                    menu_config.radius,
                    a0,
                    a1,
                ))),
                MeshMaterial2d(mats.add(ColorMaterial::from_color(
                    wheel_theme::SLICE_BASE,
                ))),
                Transform::from_translation(Vec3::Z * 0.0),
            ))
            .id();
        children.push(child);

        // 图标
        let icon_id = commands
            .spawn((
                SliceIcon { index: i },
                Text2d::new(icon.to_string()),
                TextColor(wheel_theme::ICON_NORMAL),
                Transform::from_translation(Vec3::new(
                    center_pos.x,
                    center_pos.y + 14.0,
                    1.0,
                )),
                font.map(|f| TextFont {
                    font: f.clone(),
                    font_size: 24.0,
                    ..default()
                })
                .unwrap_or(TextFont {
                    font_size: 24.0,
                    ..default()
                }),
            ))
            .id();
        children.push(icon_id);

        // 标签
        let label_id = commands
            .spawn((
                SliceLabel { index: i },
                Text2d::new(label.to_string()),
                TextColor(wheel_theme::TEXT_NORMAL),
                Transform::from_translation(Vec3::new(
                    center_pos.x,
                    center_pos.y - 12.0,
                    1.0,
                )),
                font.map(|f| TextFont {
                    font: f.clone(),
                    font_size: 13.0,
                    ..default()
                })
                .unwrap_or(TextFont {
                    font_size: 13.0,
                    ..default()
                }),
            ))
            .id();
        children.push(label_id);
    }

    // 中心圆
    let center_id = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(menu_config.inner_radius - 5.0))),
            MeshMaterial2d(mats.add(ColorMaterial::from_color(wheel_theme::CENTER_BG))),
            Transform::from_translation(Vec3::Z * 2.0),
        ))
        .id();
    children.push(center_id);

    // 中心文本
    let text_id = commands
        .spawn((
            WheelCenter,
            Text2d::new("MENU"),
            TextColor(wheel_theme::ACCENT),
            Transform::from_translation(Vec3::Z * 3.0),
            font.map(|f| TextFont {
                font: f.clone(),
                font_size: 18.0,
                ..default()
            })
            .unwrap_or(TextFont {
                font_size: 18.0,
                ..default()
            }),
        ))
        .id();
    children.push(text_id);

    for &child in &children {
        commands.entity(root).add_child(child);
    }
    root
}

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
            for (_, mut text_color, mut text_font) in queries.p0().iter_mut() {
                *text_color = TextColor(wheel_theme::TEXT_NORMAL);
                text_font.font_size = 13.0;
            }
            for (_, mut icon_color) in queries.p1().iter_mut() {
                *icon_color = TextColor(wheel_theme::ICON_NORMAL);
            }
            if let Some(hovered_idx) = state.hovered {
                for (label, mut text_color, mut text_font) in queries.p0().iter_mut() {
                    if label.index == hovered_idx {
                        *text_color = TextColor(wheel_theme::TEXT_HOVER);
                        text_font.font_size = 15.0;
                    }
                }
                for (icon, mut icon_color) in queries.p1().iter_mut() {
                    if icon.index == hovered_idx {
                        *icon_color = TextColor(wheel_theme::ICON_HOVER);
                    }
                }
            }
        }
    }
}

pub fn handle_wheel_select(
    mut commands: Commands,
    mut reader: MessageReader<WheelMenuSelected>,
    mut manager: ResMut<WheelMenuManager>,
    wheel_query: Query<Entity, With<WheelMenuRoot>>,
) {
    for event in reader.read() {
        let item_name = MENU_ITEMS
            .get(event.index)
            .map(|(name, _, _)| *name)
            .unwrap_or("未知");
        info!("快捷菜单选择: {} (索引 {})", item_name, event.index);
        manager.state.hide();
        manager.active_menu = None;
        for entity in wheel_query.iter() {
            commands.entity(entity).despawn();
        }
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
        for i in 0..menu.slices {
            let (start, end) = slice_angles(&menu, i);
            assert!(start >= 0.0);
            assert!(end <= std::f32::consts::TAU);
            assert!(end > start);
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
