"""
Redra Python客户端包
用于连接Redra服务器并发送几何形状数据
提供类似Rust sender.rs的功能
"""

from .client_sender import ClientSender

__version__ = "0.1.0"
__author__ = "Redra Project"
__all__ = ["ClientSender"]