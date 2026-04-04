pub use redra_proto::proto::*;

// 保留原来的模块声明，以便兼容旧代码
pub mod shape {
    pub use redra_proto::proto::shape::*;
}
pub mod declare {
    pub use redra_proto::proto::declare::*;
}
pub mod target {
    pub use redra_proto::proto::target::*;
}
pub mod command {
    pub use redra_proto::proto::command::*;
}
pub mod resource {
    pub use redra_proto::proto::resource::*;
}
pub mod formats {
    pub use redra_proto::proto::formats::*;
}
pub mod designation {
    pub use redra_proto::proto::designation::*;
}
pub mod transform {
    pub use redra_proto::proto::transform::*;
}