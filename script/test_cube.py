#!/usr/bin/env python3
"""
测试脚本，用于向Redra应用发送立方体定义数据
"""

import socket
import sys
import os
import time
from typing import Tuple

# 添加proto目录到Python路径
sys.path.append(os.path.join(os.path.dirname(__file__), 'proto'))

import rdr_pb2
import shape_pb2


class DataType:
    """数据类型常量"""
    CUBE = "cube"
    SPHERE = "sphere"


def create_cube_message(pos: Tuple[float, float, float], 
                       rot: Tuple[float, float, float] = (0.0, 0.0, 0.0),
                       scale: Tuple[float, float, float] = (1.0, 1.0, 1.0)) -> bytes:
    """
    创建立方体消息
    
    Args:
        pos: 位置坐标 (x, y, z)
        rot: 旋转角度 (rx, ry, rz)
        scale: 缩放比例 (sx, sy, sz)
        
    Returns:
        bytes: 编码后的立方体消息
    """
    # 创建Cube消息
    cube = shape_pb2.Cube()
    
    # 设置位置
    cube.pos.x = pos[0]
    cube.pos.y = pos[1]
    cube.pos.z = pos[2]
    
    # 设置旋转
    cube.rot.rx = rot[0]
    cube.rot.ry = rot[1]
    cube.rot.rz = rot[2]
    
    # 设置缩放
    cube.scale.sx = scale[0]
    cube.scale.sy = scale[1]
    cube.scale.sz = scale[2]
    
    return cube.SerializeToString()


def create_sphere_message(pos: Tuple[float, float, float], radius: float = 1.0) -> bytes:
    """
    创建球体消息
    
    Args:
        pos: 位置坐标 (x, y, z)
        radius: 球体半径
        
    Returns:
        bytes: 编码后的球体消息
    """
    # 创建Sphere消息
    sphere = shape_pb2.Sphere()
    
    # 设置位置
    sphere.pos.x = pos[0]
    sphere.pos.y = pos[1]
    sphere.pos.z = pos[2]
    
    # 设置半径
    sphere.radius = radius
    
    return sphere.SerializeToString()


def create_pack_message(data_type: str, data: bytes) -> bytes:
    """
    创建包装消息
    
    Args:
        data_type: 数据类型字符串
        data: 实际数据
        
    Returns:
        bytes: 编码后的包装消息
    """
    # 创建Pack消息
    pack = rdr_pb2.Pack()
    pack.data_type = data_type
    pack.data = data
    
    # 先序列化pack以计算大小（不含total_size字段）
    temp_pack_data = pack.SerializeToString()
    
    # 设置total_size为完整pack消息的预计大小
    # total_size字段本身是uint32类型，占用1-5字节(varint编码)
    # 我们需要计算包含total_size字段的实际大小
    pack.total_size = len(temp_pack_data)
    
    # 重新序列化包含total_size的完整消息
    full_pack_data = pack.SerializeToString()
    
    # 修正total_size为实际的完整消息大小
    pack.total_size = len(full_pack_data)
    
    # 最终序列化
    return pack.SerializeToString()

def send_messages(host: str, port: int, messages: list):
    """
    在一个连接中发送多条消息到指定主机和端口
    
    Args:
        host: 主机地址
        port: 端口号
        messages: 要发送的消息列表
    """
    try:
        # 创建TCP套接字
        print(f"正在连接到 {host}:{port}...")
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)  # 设置5秒超时
        sock.connect((host, port))
        
        # 发送所有消息
        total_bytes = 0
        for i, message in enumerate(messages):
            sock.sendall(message)
            total_bytes += len(message)
            print(f"成功发送第 {i+1} 条消息 ({len(message)} 字节) 到 {host}:{port}")
        
        print(f"总共发送 {total_bytes} 字节到 {host}:{port}")
        
    except socket.timeout:
        print(f"连接超时：无法在5秒内连接到 {host}:{port}")
        print("请检查：")
        print("1. Redra服务是否正在运行")
        print("2. Redra服务是否正确监听端口")
        print("3. 防火墙是否阻止了连接")
    except ConnectionRefusedError:
        print(f"连接被拒绝：无法连接到 {host}:{port}")
        print("请检查：")
        print("1. Redra服务是否正在运行")
        print("2. Redra服务是否正确监听端口")
        print("3. 是否有其他程序占用了该端口")
    except Exception as e:
        print(f"发送消息时发生未知错误: {e}")
        print(f"错误类型: {type(e).__name__}")
    finally:
        sock.close()


def main():
    """主函数"""
    # 配置参数
    HOST = 'localhost'
    PORT = 8080  # 根据源码，这是默认端口
    
    # 创建立方体数据
    cube_data = create_cube_message(
        pos=(0.0, 0.0, 0.0),      # 位置在原点
        rot=(0.0, 0.0, 0.0),      # 无旋转
        scale=(1.0, 1.0, 1.0)     # 标准大小
    )
    
    # 包装成Pack消息
    pack_message = create_pack_message(DataType.CUBE, cube_data)
    
    # 创建立方体数据
    cube_data = create_cube_message(
        pos=(0.0, 0.0, 4.0),      # 位置在原点
        rot=(0.0, 0.0, 0.0),      # 无旋转
        scale=(1.0, 1.0, 1.0)     # 标准大小
    )
    
    # 包装成Pack消息
    pack_message1 = create_pack_message(DataType.CUBE, cube_data)
    
    # 创建另一个不同位置和大小的立方体进行测试
    cube_data2 = create_cube_message(
        pos=(3.0, 1.0, 0.0),      # 在(2,1,0)位置
        rot=(0.0, 45.0, 0.0),     # Y轴旋转45度
        scale=(0.5, 1.5, 0.5)     # 不同比例缩放
    )
    
    pack_message2 = create_pack_message(DataType.CUBE, cube_data2)
    
    # 创建球体数据
    sphere_data = create_sphere_message(
        pos=(0.0, 4.0, 0.0),      # 在(0,2,0)位置
        radius=1.0                # 半径为1.0
    )
    
    pack_message3 = create_pack_message(DataType.SPHERE, sphere_data)
    
    # 在一个连接中发送所有消息
    print("正在发送立方体和球体定义...")
    send_messages(HOST, PORT, [pack_message, pack_message1, pack_message2, pack_message3])


if __name__ == '__main__':
    main()