use crate::rdmp::{Object, Mesh, Transform, object::object::AObject};

impl Object {
    pub fn set_id<T: Into<u64>>(&mut self, id: T) -> Result<(), String> {
        self.a_object = Some(AObject::Id(id.into()));
        Ok(())
    }

    pub fn set_transform<T: Into<Transform>>(&mut self, transform: T) -> Result<(), String> {
        self.a_object = Some(AObject::Transform(transform.into()));
        Ok(())
    }

    pub fn set_mesh<T: Into<Mesh>>(&mut self, Mesh: T) -> Result<(), String> {
        self.a_object = Some(AObject::Mesh(Mesh.into()));
        Ok(())
    }
}

// 实现From trait以支持多种类型的转换
impl From<u64> for Object {
    fn from(id: u64) -> Self {
        Object {
            a_object: Some(AObject::Id(id)),
        }
    }
}

impl From<Mesh> for Object {
    fn from(Mesh: Mesh) -> Self {
        Object {
            a_object: Some(AObject::Mesh(Mesh)),
        }
    }
}

impl From<Transform> for Object {
    fn from(transform: Transform) -> Self {
        Object {
            a_object: Some(AObject::Transform(transform)),
        }
    }
}