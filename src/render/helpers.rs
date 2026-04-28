use bevy::prelude::*;
use bevy::picking::Pickable;
use expto::rdmp::{ExMesh, ExTransform};

use crate::assets::materials::{MaterialManager, GenericMaterial3d};
use crate::render::conversion;

pub fn spawn_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    mesh: &ExMesh,
    transform: &ExTransform,
    material_name: &str,
    name: &str,
) -> Entity {
    let mesh_handle = conversion::proto_mesh_to_bevy(meshes, mesh)
        .unwrap_or_else(|| { log::warn!("网格转换失败，使用备用球体"); Mesh3d(meshes.add(Sphere::new(0.1))) });
    let material = material_manager.load_generic_material(material_name, asset_server);
    let transform_comp = conversion::proto_transform_to_bevy(transform);

    use std::time::{SystemTime, UNIX_EPOCH};
    let entity_id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

    commands.spawn((
        mesh_handle,
        GenericMaterial3d(material),
        transform_comp,
        Name::new(name.to_string()),
        Pickable::default(),
        crate::render::interaction::picking::PickableEntity { entity_id },
    )).id()
}

pub fn spawn_axis_with_arrow(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    asset_server: &AssetServer,
    material_manager: &MaterialManager,
    position: Vec3,
    rotation: Quat,
    length: f32,
    radius: f32,
    material_name: &str,
    name: &str,
) {
    let arrow_radius = radius * 2.0;
    let arrow_height = radius * 4.0;
    let material = material_manager.load_generic_material(material_name, asset_server);

    let cylinder_mesh = meshes.add(Cylinder::new(radius, length));
    let cylinder_transform = Transform::from_translation(position + rotation * Vec3::new(0.0, length / 2.0, 0.0)).with_rotation(rotation);
    commands.spawn((Mesh3d(cylinder_mesh), GenericMaterial3d(material.clone()), cylinder_transform, Name::new(format!("{}_Body", name))));

    let cone_mesh = meshes.add(Cone::new(arrow_radius, arrow_height));
    let cone_transform = Transform::from_translation(position + rotation * Vec3::new(0.0, length, 0.0)).with_rotation(rotation);
    commands.spawn((Mesh3d(cone_mesh), GenericMaterial3d(material), cone_transform, Name::new(format!("{}_Arrow", name))));
}
