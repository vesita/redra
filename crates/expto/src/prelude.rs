pub use crate::rdmp::*;

// 导出 IP 地址相关的实用函数
pub use crate::ip::{get_ip, get_port, get_addr};

// 导出主要 API 接口
pub use crate::rdmp::decoding::decode;
pub use crate::rdmp::encoding::encode;

pub use crate::rdmp::auto::stamper;