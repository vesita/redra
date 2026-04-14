use redra_parser::RDPack;
use redra_parser::InternalPointCloudPack;

/// 序列化点云数据为二进制格式
pub fn serialize_point_cloud(packs: &[RDPack]) -> Vec<u8> {
    // 简单的二进制序列化（稍后可以用 bincode 或 protobuf 改进）
    let mut buffer = Vec::new();
    
    // 写入点的数量
    buffer.extend_from_slice(&(packs.len() as u32).to_le_bytes());
    
    // 写入每个包（简化版，生产环境中应使用 bincode）
    for pack in packs {
        match pack {
            RDPack::Message(msg) => {
                buffer.push(0);  // 类型标记
                buffer.extend_from_slice(&(msg.len() as u32).to_le_bytes());
                buffer.extend_from_slice(msg.as_bytes());
            },
            RDPack::SpawnShape(_) => {
                buffer.push(1);  // 类型标记
                // TODO: 实现 Shape 序列化
            },
            RDPack::SpawnFormat(_) => {
                buffer.push(2);  // 类型标记
                // TODO: 实现 Format 序列化
            },
            RDPack::PointCloud(point_cloud) => {
                buffer.push(3);  // 类型标记
                buffer.extend_from_slice(&point_cloud.frame_id.to_le_bytes());
                buffer.extend_from_slice(&point_cloud.timestamp.to_le_bytes());
                buffer.extend_from_slice(&(point_cloud.points.len() as u32).to_le_bytes());
                for &(x, y, z) in &point_cloud.points {
                    buffer.extend_from_slice(&x.to_le_bytes());
                    buffer.extend_from_slice(&y.to_le_bytes());
                    buffer.extend_from_slice(&z.to_le_bytes());
                }
            },
        }
    }
    
    buffer
}

/// 反序列化点云数据
pub fn deserialize_point_cloud(buffer: &[u8]) -> Result<Vec<RDPack>, Box<dyn std::error::Error>> {
    let mut offset = 0;
    
    // 读取点的数量
    if buffer.len() < 4 {
        return Err("缓冲区太短".into());
    }
    let num_points = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    offset += 4;
    
    let mut packs = Vec::with_capacity(num_points as usize);
    
    for _ in 0..num_points {
        if offset >= buffer.len() {
            return Err("意外的缓冲区结束".into());
        }
        
        let pack_type = buffer[offset];
        offset += 1;
        
        match pack_type {
            0 => {
                // RDPack::Message
                if offset + 4 > buffer.len() {
                    return Err("意外的缓冲区结束".into());
                }
                let msg_len = u32::from_le_bytes([
                    buffer[offset],
                    buffer[offset + 1],
                    buffer[offset + 2],
                    buffer[offset + 3],
                ]) as usize;
                offset += 4;
                
                if offset + msg_len > buffer.len() {
                    return Err("意外的缓冲区结束".into());
                }
                
                let msg = String::from_utf8(buffer[offset..offset + msg_len].to_vec())?;
                offset += msg_len;
                
                packs.push(RDPack::Message(msg));
            }
            1 => {
                // RDPack::SpawnShape - 尚未实现
                return Err("SpawnShape 反序列化尚未实现".into());
            }
            2 => {
                // RDPack::SpawnFormat - 尚未实现
                return Err("SpawnFormat 反序列化尚未实现".into());
            }
            3 => {
                // RDPack::PointCloud
                if offset + 12 > buffer.len() {
                    return Err("缓冲区太短，无法读取 PointCloud 头部".into());
                }
                
                let frame_id = u32::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                ]);
                offset += 4;
                
                let timestamp = u64::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3],
                    buffer[offset + 4], buffer[offset + 5], buffer[offset + 6], buffer[offset + 7]
                ]);
                offset += 8;
                
                let point_count = u32::from_le_bytes([
                    buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                ]) as usize;
                offset += 4;
                
                if offset + point_count * 12 > buffer.len() {
                    return Err("缓冲区太短，无法读取点数据".into());
                }
                
                let mut points = Vec::with_capacity(point_count);
                for _ in 0..point_count {
                    let x = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    let y = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    let z = f32::from_le_bytes([
                        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]
                    ]);
                    offset += 4;
                    
                    points.push((x, y, z));
                }
                
                packs.push(RDPack::PointCloud(InternalPointCloudPack {
                    frame_id,
                    timestamp: timestamp as f64,
                    points,
                }));
            }
            _ => {
                return Err("未知的包类型".into());
            }
        }
    }
    
    Ok(packs)
}