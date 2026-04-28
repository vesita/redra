#!/usr/bin/env python3
"""
Redra 客户端示例

这个示例展示了如何使用优化后的proto定义与Redra主程序通信。
"""

import socket
import time
import random
import sys
import os

# 添加proto目录到Python路径
sys.path.append(os.path.join(os.path.dirname(__file__), 'proto'))

import cmd_pb2
import conception_pb2
import designation_pb2
import transform_pb2
import shape_pb2
import formats_pb2


def create_socket_connection(host='localhost', port=8080):
    """创建到Redra服务器的socket连接"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        return sock
    except Exception as e:
        print(f"无法连接到服务器 {host}:{port} - {e}")
        return None


def send_command(sock, command):
    """发送命令到服务器"""
    try:
        # 序列化命令
        serialized_data = command.SerializeToString()
        
        # 发送数据长度（4字节）+ 数据
        data_length = len(serialized_data)
        sock.sendall(data_length.to_bytes(4, byteorder='little'))
        sock.sendall(serialized_data)
        
        print(f"已发送命令，大小: {data_length} 字节")
        return True
    except Exception as e:
        print(f"发送命令失败: {e}")
        return False


def create_cube_spawn_command():
    """创建一个立方体生成命令"""
    command = cmd_pb2.Command()
    
    # 设置命令ID和时间戳
    command.command_id = f"cmd_{int(time.time())}"
    command.timestamp = int(time.time())
    
    # 创建一个立方体
    spawn = designation_pb2.Spawn()
    spawn.id.id = 1
    spawn.id.has_set = True
    spawn.name = "Example Cube"
    spawn.tags.extend(["example", "shape", "cube"])
    
    # 设置立方体数据
    cube = shape_pb2.Cube()
    cube.translation.x = random.uniform(-5, 5)
    cube.translation.y = random.uniform(-5, 5)
    cube.translation.z = random.uniform(-5, 5)
    cube.rotation.rx = random.uniform(0, 3.14)
    cube.rotation.ry = random.uniform(0, 3.14)
    cube.rotation.rz = random.uniform(0, 3.14)
    cube.scale.sx = random.uniform(0.5, 2.0)
    cube.scale.sy = random.uniform(0.5, 2.0)
    cube.scale.sz = random.uniform(0.5, 2.0)
    
    # 设置颜色
    cube.color.r = random.random()
    cube.color.g = random.random()
    cube.color.b = random.random()
    cube.color.a = 1.0
    
    # 将立方体数据放入shape_data
    spawn.shape_data.data = shape_pb2.ShapePack.Data(cube=cube)
    
    # 将spawn命令放入designation
    command.designation.cmd = designation_pb2.DesignCMD.Cmd(spawn=spawn)
    
    return command


def create_sphere_spawn_command():
    """创建一个球体生成命令"""
    command = cmd_pb2.Command()
    
    # 设置命令ID和时间戳
    command.command_id = f"sphere_cmd_{int(time.time())}"
    command.timestamp = int(time.time())
    
    # 创建一个球体
    spawn = designation_pb2.Spawn()
    spawn.id.id = 2
    spawn.id.has_set = True
    spawn.name = "Example Sphere"
    spawn.tags.extend(["example", "shape", "sphere"])
    
    # 设置球体数据
    sphere = shape_pb2.Sphere()
    sphere.pos.x = random.uniform(-5, 5)
    sphere.pos.y = random.uniform(-5, 5)
    sphere.pos.z = random.uniform(-5, 5)
    sphere.radius = random.uniform(0.5, 2.0)
    sphere.segments = 32
    
    # 设置颜色
    sphere.color.r = random.random()
    sphere.color.g = random.random()
    sphere.color.b = random.random()
    sphere.color.a = 1.0
    
    # 将球体数据放入shape_data
    spawn.shape_data.data = shape_pb2.ShapePack.Data(sphere=sphere)
    
    # 将spawn命令放入designation
    command.designation.cmd = designation_pb2.DesignCMD.Cmd(spawn=spawn)
    
    return command


def create_transform_command():
    """创建一个变换命令"""
    command = cmd_pb2.Command()
    
    # 设置命令ID和时间戳
    command.command_id = f"transform_cmd_{int(time.time())}"
    command.timestamp = int(time.time())
    
    # 创建一个变换命令
    trans_cmd = transform_pb2.TransCMD()
    
    # 设置平移
    translation = transform_pb2.Translation()
    translation.x = random.uniform(-2, 2)
    translation.y = random.uniform(-2, 2)
    translation.z = random.uniform(-2, 2)
    
    trans_cmd.trans = transform_pb2.TransCMD.Trans(translation=translation)
    
    # 设置变换选项
    options = transform_pb2.TransformOptions()
    options.is_relative = True
    options.interpolation_time = 1.0
    options.easing_function = "linear"
    trans_cmd.options.CopyFrom(options)
    
    # 将变换命令放入command
    command.transform.cmd = transform_pb2.TransCMD(trans=trans_cmd)
    
    return command


def create_update_command():
    """创建一个更新命令"""
    command = cmd_pb2.Command()
    
    # 设置命令ID和时间戳
    command.command_id = f"update_cmd_{int(time.time())}"
    command.timestamp = int(time.time())
    
    # 创建一个更新命令
    update_cmd = designation_pb2.Update()
    update_cmd.id.id = 1
    update_cmd.id.has_set = True
    
    # 创建新的姿态
    pose = shape_pb2.Pose()
    pose.translation.x = random.uniform(-5, 5)
    pose.translation.y = random.uniform(-5, 5)
    pose.translation.z = random.uniform(-5, 5)
    pose.rotation.rx = random.uniform(0, 3.14)
    pose.rotation.ry = random.uniform(0, 3.14)
    pose.rotation.rz = random.uniform(0, 3.14)
    pose.scale.sx = random.uniform(0.5, 2.0)
    pose.scale.sy = random.uniform(0.5, 2.0)
    pose.scale.sz = random.uniform(0.5, 2.0)
    
    # 设置姿态更新
    update_cmd.data = designation_pb2.Update.Data(pose=pose)
    
    # 将更新命令放入designation
    command.designation.cmd = designation_pb2.DesignCMD.Cmd(update=update_cmd)
    
    return command


def main():
    """主函数，演示如何使用客户端"""
    print("Redra 客户端示例")
    print("="*50)
    
    # 连接到服务器
    sock = create_socket_connection('localhost', 8080)
    if not sock:
        print("无法建立连接，退出...")
        return
    
    try:
        print("连接成功，开始发送命令...")
        
        # 发送几种不同类型的命令
        
        # 1. 发送一个立方体生成命令
        print("\n1. 发送立方体生成命令...")
        cube_cmd = create_cube_spawn_command()
        send_command(sock, cube_cmd)
        time.sleep(1)
        
        # 2. 发送一个球体生成命令
        print("\n2. 发送球体生成命令...")
        sphere_cmd = create_sphere_spawn_command()
        send_command(sock, sphere_cmd)
        time.sleep(1)
        
        # 3. 发送一个变换命令
        print("\n3. 发送变换命令...")
        transform_cmd = create_transform_command()
        send_command(sock, transform_cmd)
        time.sleep(1)
        
        # 4. 发送一个更新命令
        print("\n4. 发送更新命令...")
        update_cmd = create_update_command()
        send_command(sock, update_cmd)
        time.sleep(1)
        
        # 5. 再发送几个随机形状
        print("\n5. 发送更多随机形状...")
        for i in range(3):
            if random.choice([True, False]):
                cmd = create_cube_spawn_command()
                print(f"   发送立方体命令 #{i+1}")
            else:
                cmd = create_sphere_spawn_command()
                print(f"   发送球体命令 #{i+1}")
                
            send_command(sock, cmd)
            time.sleep(0.5)
        
        print("\n所有命令发送完成!")
        
    except KeyboardInterrupt:
        print("\n用户中断操作")
    except Exception as e:
        print(f"\n发生错误: {e}")
    finally:
        sock.close()
        print("\n连接已关闭")


if __name__ == "__main__":
    main()