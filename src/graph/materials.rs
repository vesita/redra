use bevy::prelude::*;

#[derive(Clone)]
pub enum PredefinedMaterial {
    Color(Color),
    Standard(StandardMaterial),
}

#[derive(Resource)]
pub struct MaterialManager {
    pub materials: std::collections::HashMap<String, PredefinedMaterial>,
}

impl MaterialManager {
    pub fn new() -> Self {
        MaterialManager {
            materials: std::collections::HashMap::new(),
        }
    }

    pub fn register_material(&mut self, name: &String, material: PredefinedMaterial) {
        self.materials.insert(name.clone(), material);
    }

    pub fn get_material(&self, name: &str) -> Option<&PredefinedMaterial> {
        self.materials.get(name)
    }
}
