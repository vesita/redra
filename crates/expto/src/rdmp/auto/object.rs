use crate::rdmp::{ExMesh, ExObject, ExTransform, ex_object, Tag, TagStyle};


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

    pub fn set_tag<T: Into<Tag>>(&mut self, tag: T) -> Result<(), String> {
        self.u_object = Some(ex_object::UObject::Tag(tag.into()));
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

impl From<Tag> for ExObject {
    fn from(tag: Tag) -> Self {
        ExObject {
            u_object: Some(ex_object::UObject::Tag(tag)),
        }
    }
}

// Tag 辅助构造函数
impl Tag {
    /// 创建一个新的标签
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            offset: None,
            style: None,
        }
    }

    /// 设置标签的位置偏移
    pub fn with_offset(mut self, offset: ExTransform) -> Self {
        self.offset = Some(offset);
        self
    }

    /// 设置标签样式
    pub fn with_style(mut self, style: TagStyle) -> Self {
        self.style = Some(style);
        self
    }
}

// TagStyle 辅助构造函数和默认值
impl TagStyle {
    /// 创建默认的标签样式
    pub fn default_style() -> Self {
        Self {
            font_size: 14.0,
            bg_r: 0.1,
            bg_g: 0.1,
            bg_b: 0.1,
            bg_a: 0.8,
            text_r: 1.0,
            text_g: 1.0,
            text_b: 1.0,
            text_a: 1.0,
            corner_radius: 4.0,
        }
    }

    /// 设置字体大小
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// 设置背景颜色 (RGBA)
    pub fn with_bg_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.bg_r = r;
        self.bg_g = g;
        self.bg_b = b;
        self.bg_a = a;
        self
    }

    /// 设置文字颜色 (RGBA)
    pub fn with_text_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.text_r = r;
        self.text_g = g;
        self.text_b = b;
        self.text_a = a;
        self
    }

    /// 设置圆角半径
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}
