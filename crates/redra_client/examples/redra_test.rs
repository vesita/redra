//! 数据管理功能验证示例
//!
//! 使用 ShapeBuilder API 发送多帧数据，测试帧回放 UI

use redra_client::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("启动 Redra 数据管理功能验证示例");
    println!("将发送 5 帧数据用于测试帧回放UI\n");

    // 第1帧：初始状态 - 3个对象
    println!("=== 发送第 1 帧 ===");
    spawn_sphere([0.0, 0.0, 0.0], 1.0, "red").id(1).send().await?;
    spawn_cylinder([2.0, 0.0, 0.0], 0.5, 1.5, "green").id(2).send().await?;
    spawn_cone([-2.0, 0.0, 0.0], 0.8, 1.2, "blue").id(3).send().await?;
    send_frame_end().await?;
    println!("第 1 帧发送完成（3个对象）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第2帧：对象移动 - 位置变化
    println!("=== 发送第 2 帧 ===");
    spawn_sphere([1.0, 1.0, 0.0], 1.0, "red").id(1).send().await?;
    spawn_cylinder([2.0, 1.0, 0.0], 0.5, 1.5, "green").id(2).send().await?;
    spawn_cone([-1.0, 1.0, 0.0], 0.8, 1.2, "blue").id(3).send().await?;
    send_frame_end().await?;
    println!("第 2 帧发送完成（3个对象，位置更新）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第3帧：添加新对象 - 增加到4个
    println!("=== 发送第 3 帧 ===");
    spawn_sphere([2.0, 2.0, 0.0], 1.0, "red").id(1).send().await?;
    spawn_cylinder([2.0, 2.0, 0.0], 0.5, 1.5, "green").id(2).send().await?;
    spawn_cone([0.0, 2.0, 0.0], 0.8, 1.2, "blue").id(3).send().await?;
    spawn_sphere([0.0, 0.0, 0.0], 0.6, "yellow").id(4).send().await?;
    send_frame_end().await?;
    println!("第 3 帧发送完成（4个对象，新增1个）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第4帧：删除对象 - 减少到3个
    println!("=== 发送第 4 帧 ===");
    spawn_sphere([3.0, 3.0, 0.0], 1.0, "red").id(1).send().await?;
    spawn_cone([1.0, 3.0, 0.0], 0.8, 1.2, "blue").id(3).send().await?;
    spawn_sphere([1.0, 1.0, 0.0], 0.6, "yellow").id(4).send().await?;
    send_frame_end().await?;
    println!("第 4 帧发送完成（3个对象，删除ID=2）\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第5帧：最终状态 - 2个对象
    println!("=== 发送第 5 帧 ===");
    spawn_sphere([4.0, 4.0, 0.0], 1.2, "red").id(1).send().await?;
    spawn_sphere([2.0, 2.0, 0.0], 0.8, "yellow").id(4).send().await?;
    send_frame_end().await?;
    println!("第 5 帧发送完成（2个对象，删除ID=3）\n");

    println!("========================================");
    println!("数据管理功能验证示例执行完成！");
    println!("共发送 5 帧数据：");
    println!("  帧1: 3个对象（初始状态）");
    println!("  帧2: 3个对象（位置更新）");
    println!("  帧3: 4个对象（新增对象）");
    println!("  帧4: 3个对象（删除对象）");
    println!("  帧5: 2个对象（最终状态）");
    println!("\n请在UI中测试：");
    println!("  - 帧切换按钮（上一帧/下一帧）");
    println!("  - 时间轴拖动");
    println!("  - 播放/暂停功能");
    println!("  - 播放速度调节");
    println!("========================================");

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
