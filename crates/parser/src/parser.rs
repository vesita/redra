use expto::rdmp::{Unit, CommandType};

/// 解析后的命令类型
#[derive(Debug, Clone)]
pub enum ParsedCommand {
    /// 创建实体
    Spawn {
        id: u64,
        transform: Option<TransformData>,
    },
    /// 更新实体
    Update {
        id: u64,
        transform: Option<TransformData>,
    },
    /// 销毁实体
    Destroy {
        id: u64,
    },
    /// 未知命令
    Unknown,
}

/// 变换数据
#[derive(Debug, Clone)]
pub struct TransformData {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl TransformData {
    pub fn from_proto(transform: &expto::rdmp::Transform) -> Self {
        Self {
            position: [transform.x, transform.y, transform.z],
            rotation: [transform.rx, transform.ry, transform.rz],
            scale: [transform.sx, transform.sy, transform.sz],
        }
    }
}

/// 协议解析器（无状态工具类）
pub struct ProtocolParser;

impl ProtocolParser {
    /// 解析单个 Unit，返回解析后的命令
    pub fn parse_unit(unit: &Unit) -> ParsedCommand {
        let Some(command) = &unit.command else {
            return ParsedCommand::Unknown;
        };

        let command_type = command.a_command();

        // 提取对象ID和变换信息
        let mut object_id: Option<u64> = None;
        let mut transform_data: Option<TransformData> = None;

        for obj in &unit.objects {
            use expto::rdmp::object::object::AObject;

            match &obj.a_object {
                Some(AObject::Id(id)) => {
                    object_id = Some(*id);
                },
                Some(AObject::Transform(transform)) => {
                    transform_data = Some(TransformData::from_proto(transform));
                },
                _ => {}
            }
        }

        let Some(id) = object_id else {
            return ParsedCommand::Unknown;
        };

        match command_type {
            CommandType::Spawn => ParsedCommand::Spawn {
                id,
                transform: transform_data,
            },
            CommandType::Update => ParsedCommand::Update {
                id,
                transform: transform_data,
            },
            CommandType::Destroy => ParsedCommand::Destroy { id },
            _ => ParsedCommand::Unknown,
        }
    }

    /// 批量解析 Units
    pub fn parse_units(units: &[Unit]) -> Vec<ParsedCommand> {
        units.iter().map(|u| Self::parse_unit(u)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unknown_command() {
        let unit = Unit {
            stamp: None,
            command: None,
            objects: vec![],
        };
        
        assert!(matches!(ProtocolParser::parse_unit(&unit), ParsedCommand::Unknown));
    }
}
