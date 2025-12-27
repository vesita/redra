use prost::Message;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use std::sync::OnceLock;
use tokio::sync::Mutex;

use crate::proto::{declare, shape, command::{self, command::Cmd}};

static CLIENT_TCPLINK: OnceLock<Mutex<Option<TcpStream>>> = OnceLock::new();

async fn get_link() -> &'static Mutex<Option<TcpStream>> {
    CLIENT_TCPLINK.get_or_init(|| Mutex::new(None))
}

async fn ensure_connection(stream_mutex: &Mutex<Option<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
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
        pos: Some(
            crate::proto::transform::Translation {
                x,
                y,
                z,
            }
        )
    };

    // 使用新的Command结构，包含Spawn命令
    let spawn_cmd = crate::proto::designation::Spawn {
        id: Some(crate::proto::target::TargetID {
            has_set: true,
            id: 1,
        }),
        shape: Some(shape::Shape {
            shape: Some(shape::shape::Shape::Point(point)),
        }),
    };

    let cmd = command::Command {
        cmd: Some(Cmd::Designation(crate::proto::designation::DesignCMD {
            cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn_cmd)),
        })),
    };

    let encoded_data: Vec<u8> = cmd.encode_to_vec();
    send_bytes(&encoded_data).await?;
    Ok(())
}

pub async fn send_segment(start: [f32; 3], end: [f32; 3]) -> Result<(), Box<dyn std::error::Error>> { 
    let segment = shape::Segment {
        start: Some(shape::Point {
            pos: Some(
                crate::proto::transform::Translation {
                    x: start[0],
                    y: start[1],
                    z: start[2],
                }
            )
        }),
        end: Some(
            shape::Point {
                pos: Some(
                    crate::proto::transform::Translation {
                        x: end[0],
                        y: end[1],
                        z: end[2],
                    }
                )
            }
        )
    };

    // 使用新的Command结构，包含Spawn命令
    let spawn_cmd = crate::proto::designation::Spawn {
        id: Some(crate::proto::target::TargetID {
            has_set: true,
            id: 2,
        }),
        shape: Some(shape::Shape {
            shape: Some(shape::shape::Shape::Segment(segment)),
        }),
    };

    let cmd = command::Command {
        cmd: Some(Cmd::Designation(crate::proto::designation::DesignCMD {
            cmd: Some(crate::proto::designation::design_cmd::Cmd::Spawn(spawn_cmd)),
        })),
    };

    let encoded_data: Vec<u8> = cmd.encode_to_vec();
    send_bytes(&encoded_data).await?;

    Ok(())
}