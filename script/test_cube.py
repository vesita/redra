#!/usr/bin/env python3
"""
测试脚本，用于向Redra应用发送立方体定义数据
"""

import socket
import struct
import time
from typing import Tuple


class DataType:
    """数据类型常量"""
    CUBE = "cube"
    SPHERE = "sphere"


def encode_varint(value: int) -> bytes:
    """编码varint值"""
    if value < 0x80:
        return struct.pack('B', value)
    # 处理更大的数值
    encoded_bytes = b''
    while value >= 0x80:
        encoded_bytes += struct.pack('B', (value & 0x7F) | 0x80)
        value >>= 7
    encoded_bytes += struct.pack('B', value)
    return encoded_bytes


def encode_string_field(field_number: int, value: str) -> bytes:
    """编码字符串字段"""
    key = (field_number << 3) | 2  # 字符串类型标签
    value_bytes = value.encode('utf-8')
    return encode_varint(key) + encode_varint(len(value_bytes)) + value_bytes


def encode_bytes_field(field_number: int, value: bytes) -> bytes:
    """编码bytes字段"""
    key = (field_number << 3) | 2  # bytes类型标签
    return encode_varint(key) + encode_varint(len(value)) + value


def encode_float_field(field_number: int, value: float) -> bytes:
    """编码浮点数字段"""
    key = (field_number << 3) | 5  # 32位浮点类型标签
    return encode_varint(key) + struct.pack('<f', value)


def create_position_message(x: float, y: float, z: float) -> bytes:
    """创建Position消息"""
    position_data = b''
    position_data += encode_float_field(1, x)  # x
    position_data += encode_float_field(2, y)  # y
    position_data += encode_float_field(3, z)  # z
    return position_data


def create_rotation_message(rx: float, ry: float, rz: float) -> bytes:
    """创建Rotation消息"""
    rotation_data = b''
    rotation_data += encode_float_field(1, rx)  # rx
    rotation_data += encode_float_field(2, ry)  # ry
    rotation_data += encode_float_field(3, rz)  # rz
    return rotation_data


def create_scale_message(sx: float, sy: float, sz: float) -> bytes:
    """创建Scale消息"""
    scale_data = b''
    scale_data += encode_float_field(1, sx)   # sx
    scale_data += encode_float_field(2, sy)   # sy
    scale_data += encode_float_field(3, sz)   # sz
    return scale_data


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
    # 构造嵌套的Position消息 (tag=1)
    position_data = create_position_message(pos[0], pos[1], pos[2])
    
    # 构造嵌套的Rotation消息 (tag=2)
    rotation_data = create_rotation_message(rot[0], rot[1], rot[2])
    
    # 构造嵌套的Scale消息 (tag=3)
    scale_data = create_scale_message(scale[0], scale[1], scale[2])
    
    # 构造Cube消息
    cube_data = b''
    # 注意：在proto定义中，Cube消息的字段是pos(1), rot(2), scale(3)，都是嵌套消息类型
    cube_data += encode_bytes_field(1, position_data)  # pos (嵌套消息)
    cube_data += encode_bytes_field(2, rotation_data)  # rot (嵌套消息)
    cube_data += encode_bytes_field(3, scale_data)     # scale (嵌套消息)
    
    return cube_data


def create_sphere_message(pos: Tuple[float, float, float], radius: float = 1.0) -> bytes:
    """
    创建球体消息
    
    Args:
        pos: 位置坐标 (x, y, z)
        radius: 球体半径
        
    Returns:
        bytes: 编码后的球体消息
    """
    # 构造嵌套的Position消息 (tag=1)
    position_data = create_position_message(pos[0], pos[1], pos[2])
    
    # 构造Sphere消息
    sphere_data = b''
    sphere_data += encode_bytes_field(1, position_data)  # pos (嵌套消息)
    sphere_data += encode_float_field(2, radius)         # radius (float)
    
    return sphere_data


def create_pack_message(data_type: str, data: bytes) -> bytes:
    """
    创建包装消息
    
    Args:
        data_type: 数据类型字符串
        data: 实际数据
        
    Returns:
        bytes: 编码后的包装消息
    """
    # 构造Pack消息
    pack_data = b''
    pack_data += encode_string_field(1, data_type)  # data_type字段
    pack_data += encode_bytes_field(2, data)       # data字段
    
    return pack_data




def test_port_connectivity(host: str, port: int) -> bool:
    """
    测试端口连通性
    
    Args:
        host: 主机地址
        port: 端口号
        
    Returns:
        bool: 是否可以连接
    """
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(3)
        result = sock.connect_ex((host, port))
        sock.close()
        return result == 0
    except Exception:
        return False


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
    
    # 首先测试端口连通性
    print("正在测试端口连通性...")
    if not test_port_connectivity(HOST, PORT):
        print(f"错误：无法连接到 {HOST}:{PORT}")
        print("\n可能的原因：")
        print("1. Redra服务未启动")
        print("2. Redra服务未正确监听端口")
        print("3. 端口号配置错误")
        print("\n请确保：")
        print("1. 运行 'cargo run' 启动Redra服务")
        print("2. 检查Redra服务日志确认监听成功")
        print("3. 使用 'ss -tuln | grep 8080' 或 'netstat -an | grep 8080' 确认端口监听状态")
        return
    
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
