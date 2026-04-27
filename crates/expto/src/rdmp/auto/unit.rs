use crate::rdmp::{ExCommand, CommandType, ExObject, Unit, auto::generate_stamp};

pub fn generate_unit() -> Unit { 
    Unit { 
        stamp: Some(generate_stamp()), 
        command: None, 
        objects: vec![],
    }
}

impl Unit {
    // pub fn encoding(&self) -> Result<Vec<u8>, String> {
    //     let encoded_data = Unit::encode_to_vec(self);
    //     Ok(encoded_data)
    // }

    // pub fn decoding(data: &[u8]) -> Result<Self, String> where Self: Sized {
    //     match prost::Message::decode(data) {
    //         Ok(pack) => {
    //             Ok(pack)
    //         },
    //         Err(e) => {
    //             println!("协议数据包解码失败: {}", e);
    //             Err("decode error".to_string())
    //         }
    //     }
    // }

    pub fn set_unknown(&mut self) -> Result<(), String> {
        self.command = Some(ExCommand { u_command: CommandType::Unknown as i32 });
        Ok(())
    }

    pub fn set_spawn(&mut self) -> Result<(), String> {
        self.command = Some(ExCommand { u_command: CommandType::Spawn as i32 });
        Ok(())
    }

    pub fn set_update(&mut self) -> Result<(), String> {
        self.command = Some(ExCommand { u_command: CommandType::Update as i32 });
        Ok(())
    }

    pub fn set_destroy(&mut self) -> Result<(), String> {
        self.command = Some(ExCommand { u_command: CommandType::Destroy as i32 });
        Ok(())
    }

    pub fn set_object<T: Into<ExObject>>(&mut self, object: T) -> Result<(), String> {
        self.objects = vec![object.into()];
        Ok(())
    }
}