use log::info;
use redra_proto::proto::{
    command::Command,
    shape::Color,
};
use std::collections::HashMap;

use crate::client::{sender::Sender, shape};

// ==================== 全局客户端管理 ====================

/// 全局客户端实例(使用懒加载)
static GLOBAL_CLIENT: tokio::sync::OnceCell<tokio::sync::Mutex<Option<Sender>>> = tokio::sync::OnceCell::const_new();

/// 初始化全局客户端连接
/// 
/// # 参数
/// * `addr` - 服务器地址,格式为 "host:port",例如 "127.0.0.1:8080"
/// 
/// # 示例
/// ```no_run
/// #[tokio::main]
/// async fn main() {
///     redra_client::init_global_client("127.0.0.1:8080").await.unwrap();
/// }
/// ```
pub async fn init_global_client(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("初始化全局客户端,连接到: {}", addr);
    let sender = Sender::connect(addr).await?;
    GLOBAL_CLIENT.get_or_init(|| async {
        tokio::sync::Mutex::new(Some(sender))
    }).await;
    info!("全局客户端初始化成功");
    Ok(())
}

/// 获取全局客户端引用
async fn get_global_client() -> Option<tokio::sync::MutexGuard<'static, Option<Sender>>> {
    GLOBAL_CLIENT.get()?.lock().await.into()
}

// ==================== 数据集管理 ====================

/// 通知服务器切换到下一个数据集
/// 
/// 服务器会自动管理数据 ID，调用此函数后，后续发送的形状将属于新的数据集
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 发送第一批数据
/// redra_client::send_point(1.0, 2.0, 3.0).await?;
/// redra_client::send_point(4.0, 5.0, 6.0).await?;
/// 
/// // 切换到下一个数据集
/// redra_client::next_set().await?;
/// 
/// // 发送第二批数据（属于新数据集）
/// redra_client::send_point(7.0, 8.0, 9.0).await?;
/// # Ok(())
/// # }
/// ```
pub async fn next_set() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    info!("请求切换到下一个数据集");
    // TODO: 实现数据集切换命令
    // 这里需要创建一个特殊的命令来通知服务器切换数据集
    // 暂时使用一个简单的标记命令
    Ok(())
}

// ==================== 形状配置构建器 ====================

/// 形状配置的可选参数
#[derive(Debug, Clone, Default)]
pub struct ShapeConfig {
    /// 颜色
    pub color: Option<Color>,
    /// 名称
    pub name: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 其他扩展字段（预留）
    pub metadata: HashMap<String, String>,
}

impl ShapeConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    /// 设置名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// 添加多个标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ==================== Level 1: 最简 API (直接发送) ====================

/// 发送点 - 最简接口
/// 
/// # 参数
/// * `x` - X坐标
/// * `y` - Y坐标  
/// * `z` - Z坐标
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 最简单的用法
/// redra_client::send_point(1.0, 2.0, 3.0).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_point(x: f32, y: f32, z: f32) -> Result<(), Box<dyn std::error::Error>> {
    send_point_with_config(x, y, z, ShapeConfig::default()).await
}

/// 发送线段 - 最简接口
/// 
/// # 参数
/// * `start` - 起点坐标 [x, y, z]
/// * `end` - 终点坐标 [x, y, z]
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// redra_client::send_segment([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_segment(start: [f32; 3], end: [f32; 3]) -> Result<(), Box<dyn std::error::Error>> {
    send_segment_with_config(start, end, ShapeConfig::default()).await
}

/// 发送立方体 - 最简接口
/// 
/// # 参数
/// * `x` - 中心X坐标
/// * `y` - 中心Y坐标
/// * `z` - 中心Z坐标
/// * `width` - 宽度(X轴方向)
/// * `height` - 高度(Y轴方向)
/// * `depth` - 深度(Z轴方向)
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// redra_client::send_cube(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_cube(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> Result<(), Box<dyn std::error::Error>> {
    send_cube_with_config(x, y, z, width, height, depth, ShapeConfig::default()).await
}

/// 发送球体 - 最简接口
/// 
/// # 参数
/// * `x` - 中心X坐标
/// * `y` - 中心Y坐标
/// * `z` - 中心Z坐标
/// * `radius` - 半径
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// redra_client::send_sphere(0.0, 0.0, 0.0, 1.0).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_sphere(x: f32, y: f32, z: f32, radius: f32) -> Result<(), Box<dyn std::error::Error>> {
    send_sphere_with_config(x, y, z, radius, ShapeConfig::default()).await
}

// ==================== Level 2: 带配置的 API ====================

/// 发送点 - 带配置
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use redra_client::ShapeConfig;
/// use redra_proto::proto::shape::Color;
/// 
/// // 使用自定义配置
/// let config = ShapeConfig::new()
///     .with_color(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 })
///     .with_name("my_point")
///     .with_tag("important");
///     
/// redra_client::send_point_with_config(1.0, 2.0, 3.0, config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_point_with_config(
    x: f32,
    y: f32,
    z: f32,
    config: ShapeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    let color = config.color.unwrap_or(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
    let name = config.name.unwrap_or_else(|| format!("point_{}_{}_{}", x, y, z));
    
    let command = shape::create_point_command(x, y, z, Some(color), None, name);
    info!("发送点: ({}, {}, {})", x, y, z);
    sender.send_command(command).await?;
    Ok(())
}

/// 发送线段 - 带配置
pub async fn send_segment_with_config(
    start: [f32; 3],
    end: [f32; 3],
    config: ShapeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    let color = config.color.unwrap_or(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
    let name = config.name.unwrap_or_else(|| format!("segment_{:.2}_{:.2}_{:.2}", start[0], start[1], start[2]));
    
    let command = shape::create_line_command(
        start[0], start[1], start[2],
        end[0], end[1], end[2],
        Some(color), None, name
    );
    info!("发送线段: {:?} -> {:?}", start, end);
    sender.send_command(command).await?;
    Ok(())
}

/// 发送立方体 - 带配置
pub async fn send_cube_with_config(
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    depth: f32,
    config: ShapeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    let color = config.color.unwrap_or(Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 });
    let name = config.name.unwrap_or_else(|| format!("cube_{}_{}_{}", x, y, z));
    
    let command = shape::create_cube_command(x, y, z, width, height, depth, Some(color), None, name);
    info!("发送立方体: ({}, {}, {})", x, y, z);
    sender.send_command(command).await?;
    Ok(())
}

/// 发送球体 - 带配置
pub async fn send_sphere_with_config(
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    config: ShapeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    let color = config.color.unwrap_or(Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 });
    let name = config.name.unwrap_or_else(|| format!("sphere_{}_{}_{}", x, y, z));
    
    let command = shape::create_sphere_command(x, y, z, radius, Some(color), None, name);
    info!("发送球体: ({}, {}, {}), 半径: {}", x, y, z, radius);
    sender.send_command(command).await?;
    Ok(())
}

// ==================== Level 3: 批量 API ====================

/// 批量发送点
/// 
/// # 示例
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let points = vec![
///     [0.0, 0.0, 0.0],
///     [1.0, 1.0, 1.0],
///     [2.0, 2.0, 2.0],
/// ];
/// redra_client::send_points(&points).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_points(points: &[[f32; 3]]) -> Result<(), Box<dyn std::error::Error>> {
    send_points_with_config(points, ShapeConfig::default()).await
}

/// 批量发送点 - 带配置
pub async fn send_points_with_config(
    points: &[[f32; 3]],
    config: ShapeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("批量发送 {} 个点", points.len());
    for point in points {
        send_point_with_config(point[0], point[1], point[2], config.clone()).await?;
    }
    Ok(())
}

// ==================== 辅助函数 ====================

/// 直接发送原始命令(高级API)
pub async fn send_command(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = get_global_client().await.ok_or("客户端未初始化,请先调用 init_global_client")?;
    let Some(ref mut sender) = *client else {
        return Err("客户端未初始化,请先调用 init_global_client".into());
    };
    
    info!("发送自定义命令");
    sender.send_command(command).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_client_not_initialized() {
        // 测试未初始化时的错误处理
        let result = send_point(1.0, 2.0, 3.0).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("客户端未初始化"));
    }

    #[test]
    fn test_shape_config_builder() {
        // 测试配置构建器
        use redra_proto::proto::shape::Color;
        
        let config = ShapeConfig::new()
            .with_color(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 })
            .with_name("test")
            .with_tag("tag1")
            .with_tag("tag2");
        
        assert!(config.color.is_some());
        assert_eq!(config.name, Some("test".to_string()));
        assert_eq!(config.tags.len(), 2);
    }
}
