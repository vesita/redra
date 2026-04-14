use std::net::TcpStream;
use std::io::Write;
use prost::Message;

/// 点云数据测试工具 - 模拟传感器发送点云数据流
/// 
/// 此示例演示如何发送原始点云数据（而非几何形状命令）到服务器，
/// 以测试 SQLite 帧录制和存储功能。
/// 
/// # 使用方法
/// ```bash
/// cargo run --example point_cloud_recording_test
/// ```
/// 
/// # 注意事项
/// 1. 确保 redra 服务器正在运行并监听 8080 端口
/// 2. 此示例发送的是原始点云数据，不是几何形状命令
/// 3. 发送的数据会被记录到 SQLite 数据库中
/// 4. 在 UI 中查看"帧列表"窗口可以看到录制的帧
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== 点云数据录制测试 ===\n");
    
    // 连接到服务器
    println!("正在连接到服务器 127.0.0.1:8080...");
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("✓ 连接成功\n");

    // 生成并发送多帧点云数据
    let num_frames = 5;
    let points_per_frame = 100;
    
    for frame_id in 0..num_frames {
        println!("发送第 {} 帧点云数据 ({} 个点)...", frame_id + 1, points_per_frame);
        
        // 生成一帧点云数据
        let point_cloud = generate_point_cloud(frame_id, points_per_frame);
        
        // 编码为 Protobuf Command 格式
        let encoded_data = encode_point_cloud_command(&point_cloud, frame_id)?;
        
        // 通过 TCP 发送（使用 Trailer 协议）
        send_encoded_command_with_trailer(&stream, &encoded_data)?;
        
        println!("  ✓ 第 {} 帧发送完成", frame_id + 1);
        
        // 等待一小段时间，模拟真实的传感器数据流
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    println!("\n✅ 测试完成！共发送 {} 帧点云数据", num_frames);
    println!("提示：在 UI 中查看帧列表，应该能看到 {} 条记录", num_frames);
    
    Ok(())
}

/// 生成模拟的点云数据
/// 
/// 创建螺旋状的点云分布，每帧有不同的半径和高度偏移
fn generate_point_cloud(frame_id: u32, num_points: usize) -> Vec<(f32, f32, f32)> {
    use std::f32::consts::PI;
    
    let mut points = Vec::with_capacity(num_points);
    
    for i in 0..num_points {
        // 生成螺旋状点云，每帧有不同的偏移
        let t = (i as f32) / (num_points as f32) * 4.0 * PI;
        let radius = 2.0 + (frame_id as f32) * 0.5;
        
        let x = radius * t.cos();
        let y = radius * t.sin();
        let z = (frame_id as f32) * 0.5 + (i as f32) * 0.02;
        
        points.push((x, y, z));
    }
    
    points
}

/// 将点云数据编码为 Protobuf Command 格式
fn encode_point_cloud_command(
    points: &[(f32, f32, f32)],
    frame_id: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use redra_proto::proto::{
        command::{self, Command},
        pointcloud::{PointCloudPack, Point3D},
    };
    
    // 创建点列表
    let point_list: Vec<Point3D> = points
        .iter()
        .map(|&(x, y, z)| Point3D { x, y, z })
        .collect();
    
    // 创建点云数据包
    let pc_pack = PointCloudPack {
        frame_id,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64,
        points: point_list,
        source_id: "test_client".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    
    // 创建 Command 消息
    let cmd = Command {
        cmd_pack: Some(command::command::CmdPack::PointCloud(pc_pack)),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64,
        command_id: format!("pc_test_{}", frame_id),
    };
    
    // 编码为二进制
    let mut buffer = Vec::new();
    cmd.encode(&mut buffer)?;
    
    Ok(buffer)
}

/// 使用TCMP协议发送数据
/// 
/// 通过redra_proto中的encode_command_with_trailer函数自动处理Trailer
fn send_encoded_command_with_trailer(stream: &TcpStream, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    use redra_proto::proto::command::Command;
    use redra_proto::coding::encoding::encode_tcmp;  // 使用新的函数名
    use prost::Message;
    
    // 解码数据为Command对象
    let command = Command::decode(data)?;
    
    // 使用redra_proto中的函数自动编码并添加trailer
    let packet = encode_tcmp(&command)?;  // 使用新的函数名
    
    // 发送编码后的数据包
    let mut stream_clone = stream.try_clone()?;
    stream_clone.write_all(&packet)?;
    stream_clone.flush()?;
    
    Ok(())
}

/// 使用 Trailer 协议发送数据
/// 
/// Trailer 协议格式：
/// [trailer_size (4 bytes)] [trailer_data] [payload_data]
fn send_with_trailer(stream: &TcpStream, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    use redra_proto::proto::declare::Trailer;
    
    let payload_len = data.len() as u32;
    
    // 创建临时 trailer 以计算大小
    let temp_trailer = Trailer {
        me: 1,  // 临时值
        next: payload_len,
    };
    let trailer_size = temp_trailer.encoded_len() as u32;
    
    // 创建最终的 trailer
    let trailer = Trailer {
        me: trailer_size,
        next: payload_len,
    };
    
    // 编码 trailer
    let mut trailer_buf = Vec::new();
    trailer.encode(&mut trailer_buf)?;
    
    // 发送 trailer + 数据
    let mut stream_clone = stream.try_clone()?;
    stream_clone.write_all(&trailer_buf)?;
    stream_clone.write_all(data)?;
    stream_clone.flush()?;
    
    Ok(())
}
