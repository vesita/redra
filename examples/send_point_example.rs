//! send_point 函数使用示例
//! 
//! 这个示例展示了如何使用 send_point 函数发送点数据到服务器
//! 
//! 运行方式:
//! 1. 确保服务器在 127.0.0.1:8080 上运行
//! 2. 运行 `cargo run --example send_point_example`

use redra::client::send_point;

#[tokio::main]
async fn main() {
    println!("开始发送点数据示例...");

    // 发送一个点 (1.0, 2.0, 3.0)
    match send_point(1.0, 2.0, 3.0).await {
        Ok(()) => println!("成功发送点 (1.0, 2.0, 3.0)!"),
        Err(e) => eprintln!("发送点失败: {}", e),
    }

    // 发送另一个点 (5.5, 6.5, 7.5)
    match send_point(5.5, 6.5, 7.5).await {
        Ok(()) => println!("成功发送点 (5.5, 6.5, 7.5)!"),
        Err(e) => eprintln!("发送点失败: {}", e),
    }

    // 发送原点 (0.0, 0.0, 0.0)
    match send_point(0.0, 0.0, 0.0).await {
        Ok(()) => println!("成功发送原点 (0.0, 0.0, 0.0)!"),
        Err(e) => eprintln!("发送点失败: {}", e),
    }

    // 批量发送点的示例
    let points = vec![
        (10.0, 20.0, 30.0),
        (-5.0, -10.0, -15.0),
        (100.0, 50.0, 25.0),
    ];

    println!("开始批量发送点数据...");
    for (x, y, z) in points {
        match send_point(x, y, z).await {
            Ok(()) => println!("成功发送点 ({}, {}, {})!", x, y, z),
            Err(e) => eprintln!("发送点 ({}, {}, {}) 失败: {}", x, y, z, e),
        }
    }

    println!("示例执行完成!");
}