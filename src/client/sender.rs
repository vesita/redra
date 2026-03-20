use prost::Message;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use std::sync::OnceLock;
use tokio::sync::Mutex;

use crate::proto::{
    command::{self, Command, command::CmdPack},
    declare,
    designation::{DesignCmd, Spawn, design_cmd::Cmd, spawn},
    shape::{self, ShapePack, shape_pack},
    transform::Translation,
};

static CLIENT_TCPLINK: OnceLock<Mutex<Option<TcpStream>>> = OnceLock::new();

async fn get_link() -> &'static Mutex<Option<TcpStream>> {
    CLIENT_TCPLINK.get_or_init(|| Mutex::new(None))
}

// 确保连接
async fn ensure_connection(
    stream_mutex: &Mutex<Option<TcpStream>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = stream_mutex.lock().await;
    if guard.is_none() {
        let stream = TcpStream::connect("127.0.0.1:8080").await?;
        *guard = Some(stream);
    }
    Ok(())
}

pub async fn send_bytes(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let stream_mutex = get_link().await;
    ensure_connection(stream_mutex).await?;

    let mut guard = stream_mutex.lock().await;
    let stream = guard.as_mut().unwrap();

    let trailer = declare::Trailer {
        me: 1,
        next: data.len() as u32,
    };
    let declare_length = trailer.encoded_len();
    let trailer = declare::Trailer {
        me: declare_length as u32,
        next: data.len() as u32,
    };

    stream.write_all(&trailer.encode_to_vec()).await?;
    stream.write_all(data).await?;

    Ok(())
}

pub async fn send_point(x: f32, y: f32, z: f32) -> Result<(), Box<dyn std::error::Error>> {
    let point = shape::Point {
        pos: Some(Translation { x, y, z }),
    };
    let spawn = Spawn {
        id: None,
        data: Some(spawn::Data::ShapeData(ShapePack {
            data: Some(shape_pack::Data::Point(point)),
        })),
    };
    let design_cmd = DesignCmd {
        cmd: Some(Cmd::Spawn(spawn)),
    };
    let pack = command::Command {
        cmd_pack: Some(CmdPack::Designation(design_cmd)),
    };
    let encoded_data: Vec<u8> = pack.encode_to_vec(); // 明确类型注解
    send_bytes(&encoded_data).await?;
    Ok(())
}

pub async fn send_segment(
    start: [f32; 3],
    end: [f32; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let segment = shape::Segment {
        start: Some(shape::Point {
            pos: Some(Translation {
                x: start[0],
                y: start[1],
                z: start[2],
            }),
        }),
        end: Some(shape::Point {
            pos: Some(Translation {
                x: end[0],
                y: end[1],
                z: end[2],
            }),
        }),
    };
    let spawn = Spawn {
        id: None,
        data: Some(spawn::Data::ShapeData(ShapePack {
            data: Some(shape_pack::Data::Segment(segment)),
        })),
    };
    let design_cmd = DesignCmd {
        cmd: Some(Cmd::Spawn(spawn)),
    };
    let pack = Command {
        cmd_pack: Some(CmdPack::Designation(design_cmd)),
    };
    let encoded_data: Vec<u8> = pack.encode_to_vec();
    send_bytes(&encoded_data).await?;

    Ok(())
}

pub fn send_image(_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn test_send_image_returns_ok() {
        // 测试 send_image 函数返回 Ok
        let dummy_data = vec![1, 2, 3, 4, 5];
        let result = send_image(&dummy_data);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_point_encoding() {
        // 测试点数据的编码逻辑（不涉及网络发送）
        let x = 1.0f32;
        let y = 2.0f32;
        let z = 3.0f32;

        let point = crate::proto::shape::Point {
            pos: Some(crate::proto::transform::Translation { x, y, z }),
        };
        let spawn = crate::proto::designation::Spawn {
            id: None,
            data: Some(crate::proto::designation::spawn::Data::ShapeData(
                crate::proto::shape::ShapePack {
                    data: Some(crate::proto::shape::shape_pack::Data::Point(point)),
                },
            )),
        };
        let design_cmd = crate::proto::designation::DesignCmd {
            cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn)),
        };
        let pack = crate::proto::command::Command {
            cmd_pack: Some(crate::proto::command::command::CmdPack::Designation(design_cmd)),
        };

        let encoded_data = pack.encode_to_vec();
        
        // 验证编码后的数据不为空
        assert!(!encoded_data.is_empty());
        
        // 验证可以解码回原始数据
        let decoded = crate::proto::command::Command::decode(encoded_data.as_slice()).expect("应该能成功解码");
        assert!(decoded.cmd_pack.is_some());
    }

    #[tokio::test]
    async fn test_segment_encoding() {
        // 测试线段数据的编码逻辑（不涉及网络发送）
        let start = [0.0f32, 0.0f32, 0.0f32];
        let end = [1.0f32, 1.0f32, 1.0f32];

        let segment = crate::proto::shape::Segment {
            start: Some(crate::proto::shape::Point {
                pos: Some(crate::proto::transform::Translation {
                    x: start[0],
                    y: start[1],
                    z: start[2],
                }),
            }),
            end: Some(crate::proto::shape::Point {
                pos: Some(crate::proto::transform::Translation {
                    x: end[0],
                    y: end[1],
                    z: end[2],
                }),
            }),
        };
        let spawn = crate::proto::designation::Spawn {
            id: None,
            data: Some(crate::proto::designation::spawn::Data::ShapeData(
                crate::proto::shape::ShapePack {
                    data: Some(crate::proto::shape::shape_pack::Data::Segment(segment)),
                },
            )),
        };
        let design_cmd = crate::proto::designation::DesignCmd {
            cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn)),
        };
        let pack = crate::proto::command::Command {
            cmd_pack: Some(crate::proto::command::command::CmdPack::Designation(design_cmd)),
        };

        let encoded_data = pack.encode_to_vec();
        
        // 验证编码后的数据不为空
        assert!(!encoded_data.is_empty());
        
        // 验证可以解码回原始数据
        let decoded = crate::proto::command::Command::decode(encoded_data.as_slice()).expect("应该能成功解码");
        assert!(decoded.cmd_pack.is_some());
    }

    #[tokio::test]
    async fn test_trailer_encoding() {
        // 测试 Trailer 编码逻辑
        let data = vec![1u8, 2, 3, 4, 5];
        
        let trailer = crate::proto::declare::Trailer {
            me: 1,
            next: data.len() as u32,
        };
        let declare_length = trailer.encoded_len();
        let final_trailer = crate::proto::declare::Trailer {
            me: declare_length as u32,
            next: data.len() as u32,
        };

        let encoded = final_trailer.encode_to_vec();
        
        // 验证编码后的数据不为空
        assert!(!encoded.is_empty());
        
        // 验证可以解码
        let decoded = crate::proto::declare::Trailer::decode(encoded.as_slice())
            .expect("应该能成功解码 Trailer");
        assert_eq!(decoded.next, data.len() as u32);
    }

    #[tokio::test]
    async fn test_point_with_different_coordinates() {
        // 测试不同坐标值的点编码
        let test_cases = vec![
            (0.0f32, 0.0f32, 0.0f32),
            (-1.0f32, -1.0f32, -1.0f32),
            (100.5f32, 200.3f32, 300.7f32),
            (f32::MAX, f32::MIN, 0.0f32),
        ];

        for (x, y, z) in test_cases {
            let point = crate::proto::shape::Point {
                pos: Some(crate::proto::transform::Translation { x, y, z }),
            };
            let spawn = crate::proto::designation::Spawn {
                id: None,
                data: Some(crate::proto::designation::spawn::Data::ShapeData(
                    crate::proto::shape::ShapePack {
                        data: Some(crate::proto::shape::shape_pack::Data::Point(point)),
                    },
                )),
            };
            let design_cmd = crate::proto::designation::DesignCmd {
                cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn)),
            };
            let pack = crate::proto::command::Command {
                cmd_pack: Some(crate::proto::command::command::CmdPack::Designation(design_cmd)),
            };

            let encoded = pack.encode_to_vec();
            assert!(!encoded.is_empty(), "坐标 ({}, {}, {}) 的编码不应为空", x, y, z);
            
            let decoded = crate::proto::command::Command::decode(encoded.as_slice())
                .expect("应该能成功解码");
            assert!(decoded.cmd_pack.is_some());
        }
    }

    #[tokio::test]
    async fn test_segment_with_different_directions() {
        // 测试不同方向的线段编码
        let test_cases = vec![
            ([0.0f32, 0.0f32, 0.0f32], [1.0f32, 0.0f32, 0.0f32]), // X 轴
            ([0.0f32, 0.0f32, 0.0f32], [0.0f32, 1.0f32, 0.0f32]), // Y 轴
            ([0.0f32, 0.0f32, 0.0f32], [0.0f32, 0.0f32, 1.0f32]), // Z 轴
            ([-5.0f32, -5.0f32, -5.0f32], [5.0f32, 5.0f32, 5.0f32]), // 对角线
        ];

        for (start, end) in test_cases {
            let segment = crate::proto::shape::Segment {
                start: Some(crate::proto::shape::Point {
                    pos: Some(crate::proto::transform::Translation {
                        x: start[0],
                        y: start[1],
                        z: start[2],
                    }),
                }),
                end: Some(crate::proto::shape::Point {
                    pos: Some(crate::proto::transform::Translation {
                        x: end[0],
                        y: end[1],
                        z: end[2],
                    }),
                }),
            };
            let spawn = crate::proto::designation::Spawn {
                id: None,
                data: Some(crate::proto::designation::spawn::Data::ShapeData(
                    crate::proto::shape::ShapePack {
                        data: Some(crate::proto::shape::shape_pack::Data::Segment(segment)),
                    },
                )),
            };
            let design_cmd = crate::proto::designation::DesignCmd {
                cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn)),
            };
            let pack = crate::proto::command::Command {
                cmd_pack: Some(crate::proto::command::command::CmdPack::Designation(design_cmd)),
            };

            let encoded = pack.encode_to_vec();
            assert!(!encoded.is_empty(), "线段 {:?} -> {:?} 的编码不应为空", start, end);
            
            let decoded = crate::proto::command::Command::decode(encoded.as_slice())
                .expect("应该能成功解码");
            assert!(decoded.cmd_pack.is_some());
        }
    }

    // 注意：以下测试需要实际的网络连接，可能需要 TCP server 在 127.0.0.1:8080
    // 如果服务器未运行，这些测试将会失败。可以根据需要启用或禁用。
    
    // #[tokio::test]
    // async fn test_send_point_integration() {
    //     // 集成测试：需要运行中的 TCP 服务器
    //     let result = send_point(1.0, 2.0, 3.0).await;
    //     // 如果服务器在运行，应该成功
    //     println!("send_point result: {:?}", result);
    // }

    // #[tokio::test]
    // async fn test_send_segment_integration() {
    //     // 集成测试：需要运行中的 TCP 服务器
    //     let result = send_segment([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).await;
    //     println!("send_segment result: {:?}", result);
    // }
}
