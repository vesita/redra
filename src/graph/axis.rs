use bevy::prelude::*;

/// 创建一个3D坐标轴系统，包含X、Y、Z三个方向的轴
/// X轴为红色，Y轴为绿色，Z轴为蓝色
pub fn spawn_axis(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    scale: f32,
) {
    let axis_length = scale;
    let axis_radius = scale * 0.025;
    
    // X轴 - 红色
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(axis_radius, axis_length))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
        Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(axis_length / 2.0, 0.0, 0.0)),
    ));

    // Y轴 - 绿色
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(axis_radius, axis_length))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 1.0, 0.0)))),
        Transform::from_translation(Vec3::new(0.0, axis_length / 2.0, 0.0)),
    ));

    // Z轴 - 蓝色
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(axis_radius, axis_length))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 0.0, 1.0)))),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(0.0, 0.0, axis_length / 2.0)),
    ));

    // X轴箭头
    commands.spawn((
        Mesh3d(meshes.add(Cone::new(axis_radius * 2.0, axis_radius * 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
        Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(axis_length, 0.0, 0.0)),
    ));

    // Y轴箭头
    commands.spawn((
        Mesh3d(meshes.add(Cone::new(axis_radius * 2.0, axis_radius * 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 1.0, 0.0)))),
        Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
    ));

    // Z轴箭头
    commands.spawn((
        Mesh3d(meshes.add(Cone::new(axis_radius * 2.0, axis_radius * 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 0.0, 1.0)))),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(0.0, 0.0, axis_length)),
    ));
}