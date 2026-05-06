use bevy::prelude::*;
use bevy_materialize::prelude::*;
use smooth_bevy_cameras::controllers::fps::{FpsCameraBundle, FpsCameraController};

/// 环境光照模式
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum LightMode {
    #[default]
    Light,
    Dark,
}

pub struct InitPlugin;

impl Plugin for InitPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LightMode>()
            .add_plugins(MaterializePlugin::new(TomlMaterialDeserializer))
            .add_systems(Startup, rd_setup)
            .add_systems(Update, sync_light_mode);
    }
}

pub fn rd_setup(
    mut commands: Commands,
    mut global_ambient: ResMut<GlobalAmbientLight>,
) {
    *global_ambient = GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: true,
    };

    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    )).insert(FpsCameraBundle::new(
        FpsCameraController { enabled: true, mouse_rotate_sensitivity: Vec2::new(0.1, 0.1), ..Default::default() },
        Vec3::new(-2.5, 4.5, 9.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Y,
    ));

    commands.spawn((
        Camera2d,
        Camera { order: 1, ..default() },
    ));
}

fn sync_light_mode(
    light_mode: Res<LightMode>,
    mut clear_color: ResMut<ClearColor>,
) {
    if !light_mode.is_changed() { return; }
    match *light_mode {
        LightMode::Light => {
            *clear_color = ClearColor(Color::srgb(0.7, 0.8, 0.9));
        }
        LightMode::Dark => {
            *clear_color = ClearColor(Color::srgb(0.05, 0.05, 0.08));
        }
    }
}
