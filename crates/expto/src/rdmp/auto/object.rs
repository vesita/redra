use crate::rdmp::{ExMesh, ExObject, ExTransform, ex_object};


impl ExObject {
    pub fn set_id<T: Into<u64>>(&mut self, id: T) -> Result<(), String> {
        self.u_object = Some(ex_object::UObject::Id(id.into()));
        Ok(())
    }

    pub fn set_transform<T: Into<ExTransform>>(&mut self, transform: T) -> Result<(), String> {
        self.u_object = Some(ex_object::UObject::Transform(transform.into()));
        Ok(())
    }

    pub fn set_mesh<T: Into<ExMesh>>(&mut self, mesh: T) -> Result<(), String> {
        self.u_object = Some(ex_object::UObject::Mesh(mesh.into()));
        Ok(())
    }

    pub fn set_material_id<T: Into<String>>(&mut self, material_id: T) -> Result<(), String> {
        self.u_object = Some(ex_object::UObject::MaterialId(material_id.into()));
        Ok(())
    }
}

// 实现From trait以支持多种类型的转换
impl From<u64> for ExObject {
    fn from(id: u64) -> Self {
        ExObject {
            u_object: Some(ex_object::UObject::Id(id)),
        }
    }
}

impl From<ExMesh> for ExObject {
    fn from(mesh: ExMesh) -> Self {
        ExObject {
            u_object: Some(ex_object::UObject::Mesh(mesh)),
        }
    }
}

impl From<ExTransform> for ExObject {
    fn from(transform: ExTransform) -> Self {
        ExObject {
            u_object: Some(ex_object::UObject::Transform(transform)),
        }
    }
}