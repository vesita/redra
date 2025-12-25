use prost::Message;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::proto::{declare, rd, shape};

pub async fn send_bytes(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
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
            rd::Position {
                x,
                y,
                z,
            }
        )
    };
    let pack = rd::Pack {
        data_type: "point".to_string(),
        data: point.encode_to_vec(),
    };
    let encoded_data: Vec<u8> = pack.encode_to_vec(); // 明确类型注解
    send_bytes(&encoded_data).await?;
    Ok(())
}

pub async fn send_segment(start: [f32; 3], end: [f32; 3]) -> Result<(), Box<dyn std::error::Error>> { 
    let segment = shape::Segment {
        start: Some(shape::Point {
            pos: Some(
                rd::Position {
                    x: start[0],
                    y: start[1],
                    z: start[2],
                }
            )
        }),
        end: Some(
            shape::Point {
                pos: Some(
                    rd::Position {
                        x: end[0],
                        y: end[1],
                        z: end[2],
                    }
                )
            }
        )
    };
    let pack = rd::Pack {
        data_type: "segment".to_string(),
        data: segment.encode_to_vec(),
    };
    let encoded_data: Vec<u8> = pack.encode_to_vec();
    send_bytes(&encoded_data).await?;

    Ok(())
}