#!/usr/bin/env python3
"""
简化版测试脚本，用于向Redra应用发送几何体定义数据
"""

import socket
import sys
import os
from typing import Tuple
import numpy as np

# 添加proto目录到Python路径
sys.path.append(os.path.join(os.path.dirname(__file__), 'proto'))

import rd_pb2
import shape_pb2
import declare_pb2


def create_pack_message(data_type: str, data: bytes) -> bytes:
    """
    创建包装消息
    
    Args:
        data_type: 数据类型字符串
        data: 实际数据
        
    Returns:
        bytes: 编码后的包装消息
    """
    pack = rd_pb2.Pack()
    pack.data_type = data_type
    pack.data = data
    return pack.SerializeToString()


def trailer_pack(pack):
    trailer = declare_pb2.Trailer()
    trailer.next = len(pack)
    # trailer.me = 1
    # initial_size = len(trailer.SerializeToString())
    # # 重新设置me为实际长度
    # trailer.me = initial_size
    trailer.me = 2 + len(trailer.SerializeToString())
    return trailer.SerializeToString()

def create_cube(pos: Tuple[float, float, float], 
               rot: Tuple[float, float, float] = (0.0, 0.0, 0.0),
               scale: Tuple[float, float, float] = (1.0, 1.0, 1.0)) -> bytes:
    """创建立方体消息"""
    cube = shape_pb2.Cube()
    cube.pos.x, cube.pos.y, cube.pos.z = pos
    cube.rot.rx, cube.rot.ry, cube.rot.rz = rot
    cube.scale.sx, cube.scale.sy, cube.scale.sz = scale
    return create_pack_message("cube", cube.SerializeToString())


def create_sphere(pos: Tuple[float, float, float], radius: float = 1.0) -> bytes:
    """创建球体消息"""
    sphere = shape_pb2.Sphere()
    sphere.pos.x, sphere.pos.y, sphere.pos.z = pos
    sphere.radius = radius
    return create_pack_message("sphere", sphere.SerializeToString())


def create_line(start_pos: Tuple[float, float, float], end_pos: Tuple[float, float, float]) -> bytes:
    """创建线段消息"""
    segment = shape_pb2.Segment()
    segment.start.pos.x, segment.start.pos.y, segment.start.pos.z = start_pos
    segment.end.pos.x, segment.end.pos.y, segment.end.pos.z = end_pos
    return create_pack_message("segment", segment.SerializeToString())


def send_messages(host: str, port: int, messages: list):
    """
    在一个连接中发送多条消息到指定主机和端口
    """
    try:
        print(f"正在连接到 {host}:{port}...")
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(5)
            sock.connect((host, port))
            
            total_bytes = sum(len(msg) for msg in messages)
            for i, message in enumerate(messages):
                sock.sendall(message)
                print(f"成功发送第 {i+1} 条消息 ({len(message)} 字节)")
            
            print(f"总共发送 {total_bytes} 字节")
            
    except (socket.timeout, ConnectionRefusedError) as e:
        print(f"连接错误: {e}")
        print("请检查Redra服务是否正在运行并监听正确的端口")
    except Exception as e:
        print(f"发送消息时发生错误: {e}")


def send_messages_with_trailer(host: str, port: int, messages: list):
    """
    在一个连接中发送多条消息到指定主机和端口
    """
    try:
        print(f"正在连接到 {host}:{port}...")
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(5)
            sock.connect((host, port))
            
            total_bytes = 0
            for i, message in enumerate(messages):
                # 解析消息以获取类型信息
                pack = rd_pb2.Pack()
                pack.ParseFromString(message)
                
                trailer = trailer_pack(message)
                # 解析trailer以显示其内容
                trailer_obj = declare_pb2.Trailer()
                trailer_obj.ParseFromString(trailer)
                
                sock.sendall(trailer)
                sock.sendall(message)
                
                print(f"成功发送第 {i+1} 条消息 - 类型: {pack.data_type}, "
                      f"数据大小: {len(message)} 字节, "
                      f"预告信息: {len(trailer)} 字节 (next={trailer_obj.next}, me={trailer_obj.me})")
                
                total_bytes += len(message) + len(trailer)
            
            print(f"总共发送 {total_bytes} 字节 (包含预告信息)")
    except (socket.timeout, ConnectionRefusedError) as e:
        print(f"连接错误: {e}")
        print("请检查Redra服务是否正在运行并监听正确的端口")
    except Exception as e:
        print(f"发送消息时发生错误: {e}")

def main():
    """主函数"""
    HOST, PORT = 'localhost', 8080
    
    print(f"准备发送几何体定义到 {HOST}:{PORT}")
    
    # 创建测试几何体
    messages = [
        create_cube(pos=(0.0, 0.0, 0.0)),
        create_cube(pos=(0.0, 0.0, 4.0)),
        create_cube(pos=(3.0, 1.0, 0.0), rot=(0.0, 45.0, 0.0), scale=(0.5, 1.5, 0.5)),
        create_sphere(pos=(0.0, 4.0, 0.0), radius=1.0),
        create_line(start_pos=(0.0, 2.0, 0.0), end_pos=(4.0, 2.0, 0.0))  # 添加一条线段
    ]
    
    # 显示将要发送的消息信息
    for i, msg in enumerate(messages):
        pack = rd_pb2.Pack()
        pack.ParseFromString(msg)
        print(f"消息 {i+1}: 类型={pack.data_type}, 大小={len(msg)} 字节")
    
    print("正在发送几何体定义...")
    send_messages_with_trailer(HOST, PORT, messages)


if __name__ == '__main__':
    main()