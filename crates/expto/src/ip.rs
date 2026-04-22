use std::env;

/// 获取服务器IP地址
/// 
/// 优先级顺序：
/// 1. 环境变量 REDRA_SERVER_IP
/// 2. 默认值 "127.0.0.1"
pub fn get_ip() -> String {
    env::var("REDRA_SERVER_IP")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// 获取服务器端口号
/// 
/// 优先级顺序：
/// 1. 环境变量 REDRA_SERVER_PORT
/// 2. 默认值 17372
pub fn get_port() -> u16 {
    env::var("REDRA_SERVER_PORT")
        .unwrap_or_else(|_| "17372".to_string())
        .parse()
        .unwrap_or(17372)
}

/// 组合IP和端口为地址字符串
pub fn get_addr() -> String {
    format!("{}:{}", get_ip(), get_port())
}