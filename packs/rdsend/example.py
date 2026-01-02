import os
import sys
import time

# 添加proto目录到路径
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'proto'))

from client import ClientSender


def example_send_shapes():
    """示例：发送各种几何形状"""
    print("=== 发送几何形状示例 ===")
    
    # 创建客户端
    client = ClientSender()
    
    if not client.connect():
        print("无法连接到服务器")
        return False

    try:
        # 发送点
        print("发送点 (5.0, 5.0, 5.0)")
        client.send_point(5.0, 5.0, 5.0)
        time.sleep(0.1)  # 稍微等待一下

        # 发送线段
        print("发送线段 (0,0,0) 到 (3,3,3)")
        client.send_segment([0.0, 0.0, 0.0], [3.0, 3.0, 3.0])
        time.sleep(0.1)

        # 发送球体
        print("发送球体 (0, 0, 0)，半径 2.0")
        client.send_sphere(0.0, 0.0, 0.0, 2.0)
        time.sleep(0.1)

        # 发送多个立方体以创建一个简单的形状
        print("发送多个立方体")
        positions = [
            (0, 0, 0),
            (2, 0, 0),
            (0, 2, 0),
            (0, 0, 2),
            (2, 2, 0),
            (2, 0, 2),
            (0, 2, 2),
            (2, 2, 2)
        ]
        
        for i, (x, y, z) in enumerate(positions):
            print(f"发送立方体 {i+1}/8: ({x}, {y}, {z})")
            client.send_cube(x, y, z, sx=0.5, sy=0.5, sz=0.5)
            time.sleep(0.1)

        print("所有形状发送完成")
        return True

    except Exception as e:
        print(f"发送过程中出现错误: {e}")
        return False

    finally:
        client.close()



def main():
    print("Redra Python客户端示例")
    print("这个脚本演示了如何使用Python客户端发送几何形状到Redra服务器")
    
    # 运行基本示例
    success1 = example_send_shapes()

    if success1:
        print("\n示例运行成功！")
    else:
        print("\n示例运行失败。")


if __name__ == "__main__":
    main()