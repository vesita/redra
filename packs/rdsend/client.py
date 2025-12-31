import os
import sys
import socket
import struct

# 添加proto目录到路径
sys.path.append(os.path.join(os.path.dirname(__file__), 'proto'))

# 导入生成的protobuf模块
import declare_pb2
import cmd_pb2
import designation_pb2
import shape_pb2
import transform_pb2


class ClientSender:
    def __init__(self, host='127.0.0.1', port=8080):
        self.host = host
        self.port = port
        self.socket = None

    def connect(self):
        """建立到服务器的TCP连接"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            print(f"已连接到 {self.host}:{self.port}")
            return True
        except Exception as e:
            print(f"连接失败: {e}")
            return False

    def send_bytes(self, data):
        """发送带trailer的数据包"""
        if not self.socket:
            print("错误: 未建立连接")
            return False

        # 创建trailer - 包含当前trailer长度和数据长度
        # 根据Rust代码中的read_trailer函数，协议格式如下：
        # 1. 首先创建一个临时trailer，其中me字段初始为1
        temp_trailer = declare_pb2.Trailer()
        temp_trailer.me = 1  # 初始值
        temp_trailer.next = len(data)
        
        # 2. 计算这个trailer编码后的长度
        temp_trailer_data = temp_trailer.SerializeToString()
        trailer_length = len(temp_trailer_data)
        
        # 3. 创建最终的trailer，me字段是trailer的长度，next是数据长度
        final_trailer = declare_pb2.Trailer()
        final_trailer.me = trailer_length
        final_trailer.next = len(data)
        
        final_trailer_data = final_trailer.SerializeToString()

        try:
            # 发送trailer和数据
            self.socket.sendall(final_trailer_data)
            self.socket.sendall(data)
            return True
        except Exception as e:
            print(f"发送失败: {e}")
            return False

    def send_point(self, x, y, z):
        """发送点数据"""
        # 创建点对象
        point = shape_pb2.Point()
        point.pos.x = x
        point.pos.y = y
        point.pos.z = z

        # 创建ShapePack消息
        shape_pack = shape_pb2.ShapePack()
        shape_pack.point.CopyFrom(point)

        # 创建Spawn消息
        spawn = designation_pb2.Spawn()
        spawn.shape_data.CopyFrom(shape_pack)

        # 创建DesignCMD消息
        design_cmd = designation_pb2.DesignCMD()
        design_cmd.spawn.CopyFrom(spawn)

        # 创建Command消息
        command = cmd_pb2.Command()
        command.designation.CopyFrom(design_cmd)

        # 序列化并发送
        data = command.SerializeToString()
        return self.send_bytes(data)

    def send_segment(self, start, end):
        """发送线段数据"""
        # 创建线段对象
        segment = shape_pb2.Segment()
        
        # 设置起点
        segment.start.pos.x = start[0]
        segment.start.pos.y = start[1]
        segment.start.pos.z = start[2]
        
        # 设置终点
        segment.end.pos.x = end[0]
        segment.end.pos.y = end[1]
        segment.end.pos.z = end[2]

        # 创建ShapePack消息
        shape_pack = shape_pb2.ShapePack()
        shape_pack.segment.CopyFrom(segment)

        # 创建Spawn消息
        spawn = designation_pb2.Spawn()
        spawn.shape_data.CopyFrom(shape_pack)

        # 创建DesignCMD消息
        design_cmd = designation_pb2.DesignCMD()
        design_cmd.spawn.CopyFrom(spawn)

        # 创建Command消息
        command = cmd_pb2.Command()
        command.designation.CopyFrom(design_cmd)

        # 序列化并发送
        data = command.SerializeToString()
        return self.send_bytes(data)

    def send_sphere(self, x, y, z, radius):
        """发送球体数据"""
        # 创建球体对象
        sphere = shape_pb2.Sphere()
        sphere.pos.x = x
        sphere.pos.y = y
        sphere.pos.z = z
        sphere.radius = radius

        # 创建ShapePack消息
        shape_pack = shape_pb2.ShapePack()
        shape_pack.sphere.CopyFrom(sphere)

        # 创建Spawn消息
        spawn = designation_pb2.Spawn()
        spawn.shape_data.CopyFrom(shape_pack)

        # 创建DesignCMD消息
        design_cmd = designation_pb2.DesignCMD()
        design_cmd.spawn.CopyFrom(spawn)

        # 创建Command消息
        command = cmd_pb2.Command()
        command.designation.CopyFrom(design_cmd)

        # 序列化并发送
        data = command.SerializeToString()
        return self.send_bytes(data)

    def send_cube(self, x, y, z, rx=0, ry=0, rz=0, sx=1, sy=1, sz=1):
        """发送立方体数据"""
        # 创建立方体对象
        cube = shape_pb2.Cube()
        cube.translation.x = x
        cube.translation.y = y
        cube.translation.z = z
        
        cube.rotation.rx = rx
        cube.rotation.ry = ry
        cube.rotation.rz = rz
        
        cube.scale.sx = sx
        cube.scale.sy = sy
        cube.scale.sz = sz

        # 创建ShapePack消息
        shape_pack = shape_pb2.ShapePack()
        shape_pack.cube.CopyFrom(cube)

        # 创建Spawn消息
        spawn = designation_pb2.Spawn()
        spawn.shape_data.CopyFrom(shape_pack)

        # 创建DesignCMD消息
        design_cmd = designation_pb2.DesignCMD()
        design_cmd.spawn.CopyFrom(spawn)

        # 创建Command消息
        command = cmd_pb2.Command()
        command.designation.CopyFrom(design_cmd)

        # 序列化并发送
        data = command.SerializeToString()
        return self.send_bytes(data)

    def close(self):
        """关闭连接"""
        if self.socket:
            self.socket.close()
            self.socket = None


def main():
    # 创建客户端实例
    client = ClientSender()

    # 连接到服务器
    if not client.connect():
        print("无法连接到服务器")
        return

    try:
        # 测试发送点
        print("发送点 (1.0, 2.0, 3.0)")
        client.send_point(1.0, 2.0, 3.0)

        # 测试发送线段
        print("发送线段 (0,0,0) 到 (1,1,1)")
        client.send_segment([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])

        # 测试发送球体
        print("发送球体 (2.0, 2.0, 2.0), 半径 1.5")
        client.send_sphere(2.0, 2.0, 2.0, 1.5)

        # 测试发送立方体
        print("发送立方体 (0,0,0), 旋转 (0,0,0), 缩放 (1,1,1)")
        client.send_cube(0.0, 0.0, 0.0)

    finally:
        # 关闭连接
        client.close()
        print("连接已关闭")


if __name__ == "__main__":
    main()