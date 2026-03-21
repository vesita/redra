#!/usr/bin/env python3
"""
Redra 交互式测试管理器

提供一个交互式的测试管理界面，支持：
1. 实时查看可用测试
2. 选择并运行测试
3. 监控测试进度
4. 查看测试结果统计
5. 批量运行测试套件
6. 生成测试日志和报告

使用方法:
    python test_manager.py
    
或者使用命令行参数:
    python test_manager.py --list          # 列出所有测试
    python test_manager.py --run TEST_NAME # 运行指定测试
    python test_manager.py --batch FILE    # 批量运行测试列表中的测试
"""

import subprocess
import sys
import os
import time
import json
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional
import threading
import queue
import logging
import re


def strip_ansi_codes(text: str) -> str:
    """移除 ANSI 转义序列（颜色代码等）
    
    Args:
        text: 包含 ANSI 代码的文本
        
    Returns:
        清理后的纯文本
    """
    # ANSI 转义序列正则表达式
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    return ansi_escape.sub('', text)


class TestManager:
    """交互式测试管理器"""
    
    def __init__(self):
        """初始化测试管理器"""
        self.project_root = Path(__file__).parent.parent  # tests 目录
        self.running_tests = []
        self.test_history = []
        
        # 初始化日志记录器 - 使用 tests/log 目录（不是 tests/run/log）
        log_dir = Path(__file__).parent.parent / 'log'  # 父目录的 log 文件夹
        log_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        self.log_file = log_dir / f'test_manager_{timestamp}.log'
        
        # 配置日志
        self.logger = logging.getLogger('test_manager')
        self.logger.setLevel(logging.INFO)
        self.logger.handlers.clear()
        
        # 文件处理器
        file_handler = logging.FileHandler(self.log_file, encoding='utf-8')
        file_handler.setLevel(logging.INFO)
        file_format = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
        file_handler.setFormatter(file_format)
        self.logger.addHandler(file_handler)
        
        # 定义测试套件
        self.test_suites = {
            'quick': [
                'test_rdchannel_creation',
                'test_channel_send_receive',
                'test_float_label_basic_creation',
            ],
            'communication': [
                'test_rdchannel_creation',
                'test_channel_send_receive',
                'test_client_sender_point_encoding',
                'test_client_sender_segment_encoding',
                'test_trailer_encoding',
                'test_full_application_startup',
            ],
            'float_label': [
                'test_float_label_basic_creation',
                'test_float_label_plugin_registration',
                'test_float_label_spawn_in_scene',
                'test_spawn_label_at_helper',
                'test_billboard_effect',
                'test_full_scene',
            ],
            'integration': [
                'test_full_application_startup',
                'test_full_scene',
                'test_main_like_application',
            ],
            'stress': [
                'test_stress_many_messages',
                'test_stress_many_labels',
            ],
            'all': None,  # 表示所有测试
        }
        
        # 颜色代码
        self.COLORS = {
            'GREEN': '\033[92m',
            'RED': '\033[91m',
            'YELLOW': '\033[93m',
            'BLUE': '\033[94m',
            'CYAN': '\033[96m',
            'MAGENTA': '\033[95m',
            'RESET': '\033[0m',
            'BOLD': '\033[1m',
        }
    
    def color_print(self, message: str, color: str = 'RESET', bold: bool = False):
        """彩色打印"""
        color_code = self.COLORS.get(color, '')
        bold_code = self.COLORS['BOLD'] if bold else ''
        print(f"{bold_code}{color_code}{message}{self.COLORS['RESET']}")
    
    def log_message(self, message: str, level: str = 'INFO'):
        """记录日志到控制台和文件（自动清理 ANSI 代码）
        
        Args:
            message: 日志消息
            level: 日志级别 (INFO, WARNING, ERROR, SUCCESS)
        """
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        
        # 输出到控制台（保留原始消息，包含颜色）
        print(f"[{timestamp}] [{level}] {message}")
        
        # 记录到文件（移除 ANSI 代码）
        clean_message = strip_ansi_codes(message)
        log_level = getattr(logging, level.upper() if level != 'SUCCESS' else 'INFO', logging.INFO)
        self.logger.log(log_level, f"[{level}] {clean_message}")
    
    def save_test_result(self, result: Dict):
        """保存测试结果（仅保留内存中的历史记录）
        
        Args:
            result: 测试结果字典
        """
        # 不再保存 JSON 文件，只保留在内存历史中
        self.test_history.append(result)

    def list_available_tests(self) -> List[str]:
        """列出所有可用的测试
        
        Returns:
            测试名称列表
        """
        self.color_print("\n[SCAN] 正在扫描可用测试...", 'BLUE')
        self.log_message("开始扫描可用测试", 'INFO')
        
        command = ["cargo", "test", "--no-run", "--message-format=json"]
        
        try:
            result = subprocess.run(
                command,
                cwd=self.project_root,
                capture_output=True,
                text=True
            )
            
            tests = set()
            for line in result.stdout.split('\n'):
                if line.strip():
                    try:
                        msg = json.loads(line)
                        if msg.get('reason') == 'test' and msg.get('event') == 'started':
                            tests.add(msg['name'])
                    except:
                        pass
            
            # 如果 JSON 解析失败，使用预定义的测试列表
            if not tests:
                for suite_name, suite_tests in self.test_suites.items():
                    if suite_tests:
                        tests.update(suite_tests)
            
            tests = sorted(list(tests))
            self.log_message(f"找到 {len(tests)} 个可用测试", 'SUCCESS')
            self.color_print(f"[OK] 找到 {len(tests)} 个可用测试", 'GREEN')
            return tests
            
        except Exception as e:
            self.log_message(f"扫描测试失败：{e}", 'ERROR')
            self.color_print(f"[FAILED] 扫描测试失败：{e}", 'RED')
            return []
    
    def run_test(self, test_name: str, release: bool = False, 
                 timeout: int = 300) -> Dict:
        """运行单个测试
        
        Args:
            test_name: 测试名称
            release: 是否使用 Release 模式
            timeout: 超时时间（秒）
            
        Returns:
            测试结果字典
        """
        result = {
            'name': test_name,
            'status': 'running',
            'start_time': datetime.now(),
            'end_time': None,
            'duration': 0,
            'output': '',
            'success': False,
            'release_mode': release,
        }
        
        self.color_print(f"\n[TEST] 运行：{test_name}", 'CYAN', True)
        self.log_message(f"开始运行测试：{test_name} ({'Release' if release else 'Debug'})", 'INFO')
        self.logger.info(f"测试文件：{test_name}")
        
        command = ["cargo", "test"]
        if release:
            command.append("--release")
        
        # 确定测试文件
        if 'communication' in test_name or 'channel' in test_name or 'sender' in test_name:
            command.extend(["--test", "communication_test"])
        elif 'float_label' in test_name or 'label' in test_name:
            command.extend(["--test", "float_label_test"])
        
        command.extend([test_name, "--", "--nocapture"])
        
        start_time = time.time()
        
        try:
            # 直接输出到终端，不捕获
            proc = subprocess.Popen(
                command,
                cwd=self.project_root
            )
            
            # 等待完成
            try:
                proc.wait(timeout=timeout)
                result['success'] = proc.returncode == 0
                result['status'] = 'passed' if result['success'] else 'failed'
                result['output'] = "(输出已实时显示)"
            except subprocess.TimeoutExpired:
                proc.kill()
                result['status'] = 'timeout'
                result['success'] = False
                result['output'] = f"测试超时 ({timeout}秒)"
                self.log_message(f"测试超时：{test_name}", 'ERROR')
                self.color_print(f"\n[TIMEOUT] 测试超时", 'RED')
            
        except Exception as e:
            result['status'] = 'error'
            result['success'] = False
            result['output'] = str(e)
            self.log_message(f"测试出错：{test_name} - {e}", 'ERROR')
            self.color_print(f"\n[ERROR] 测试出错：{e}", 'RED')
        
        end_time = time.time()
        result['end_time'] = datetime.now()
        result['duration'] = end_time - start_time
        
        # 显示结果
        if result['success']:
            self.log_message(f"测试通过：{test_name} ({result['duration']:.2f}s)", 'SUCCESS')
            self.color_print(
                f"\n[PASS] 通过：{test_name} ({result['duration']:.2f}s)",
                'GREEN', True
            )
            self.logger.info(f"测试通过：{test_name}, 耗时：{result['duration']:.2f}s")
        else:
            self.log_message(f"测试失败：{test_name} ({result['duration']:.2f}s)", 'ERROR')
            self.color_print(
                f"\n[FAIL] 失败：{test_name} ({result['duration']:.2f}s)",
                'RED', True
            )
            self.logger.error(f"测试失败：{test_name}, 耗时：{result['duration']:.2f}s")
        
        # 保存结果
        self.save_test_result(result)
        
        return result
    
    def run_suite(self, suite_name: str, release: bool = False) -> List[Dict]:
        """运行测试套件
        
        Args:
            suite_name: 套件名称
            release: 是否使用 Release 模式
            
        Returns:
            测试结果列表
        """
        if suite_name not in self.test_suites:
            self.log_message(f"未知的测试套件：{suite_name}", 'ERROR')
            self.color_print(f"[FAILED] 未知的测试套件：{suite_name}", 'RED')
            return []
        
        tests = self.test_suites[suite_name]
        
        if tests is None:
            # 所有测试
            tests = self.list_available_tests()
        
        self.color_print(f"\n{'='*60}", 'CYAN', True)
        self.color_print(f"[SUITE] 运行测试套件：{suite_name}", 'CYAN', True)
        self.color_print(f"包含 {len(tests)} 个测试", 'BLUE')
        self.log_message(f"开始运行测试套件：{suite_name}, 共 {len(tests)} 个测试", 'INFO')
        print('='*60)
        
        results = []
        for i, test in enumerate(tests, 1):
            self.color_print(f"\n[{i}/{len(tests)}]", 'YELLOW')
            result = self.run_test(test, release)
            results.append(result)
            self.test_history.append(result)
        
        # 统计
        passed = sum(1 for r in results if r['success'])
        failed = len(results) - passed
        
        self.log_message(
            f"测试套件 {suite_name} 完成：{passed}/{len(results)} 通过",
            'SUCCESS' if failed == 0 else 'WARNING'
        )
        
        self.color_print(f"\n{'='*60}", 'CYAN', True)
        self.color_print(f"[COMPLETE] 套件完成：{passed}/{len(results)} 通过", 
                        'GREEN' if failed == 0 else 'YELLOW', True)
        print('='*60)
        
        # 保存套件结果摘要（仅内存中）
        summary = {
            'suite': suite_name,
            'timestamp': datetime.now().isoformat(),
            'total': len(results),
            'passed': passed,
            'failed': failed,
            'release_mode': release,
            'results': [
                {
                    'name': r['name'],
                    'status': r['status'],
                    'duration': r['duration']
                } for r in results
            ]
        }
        
        # 不再保存套件摘要文件，仅在控制台显示
        
        return results
    
    def show_menu(self):
        """显示主菜单"""
        print("\n" + "="*60)
        self.color_print("[MANAGER] Redra 交互式测试管理器", 'CYAN', True)
        print("="*60)
        print("\n可用命令:")
        print("  l / list          - 列出所有测试")
        print("  r / run <TEST>    - 运行指定测试")
        print("  s / suite <NAME>  - 运行测试套件")
        print("  h / history       - 查看测试历史")
        print("  c / clear         - 清空历史")
        print("  q / quit          - 退出")
        print("\n可用套件:")
        for name in self.test_suites.keys():
            print(f"  - {name}")
        print("="*60)
    
    def interactive_mode(self):
        """交互模式"""
        self.show_menu()
        
        while True:
            try:
                cmd = input("\n> ").strip().lower()
                
                if not cmd:
                    continue
                
                parts = cmd.split()
                command = parts[0]
                args = parts[1:] if len(parts) > 1 else []
                
                if command in ['q', 'quit', 'exit']:
                    self.color_print("\n[BYE] 再见！", 'CYAN')
                    break
                
                elif command in ['l', 'list']:
                    tests = self.list_available_tests()
                    print("\n可用测试:")
                    for test in tests:
                        print(f"  - {test}")
                
                elif command in ['r', 'run']:
                    if not args:
                        self.color_print("[ERROR] 请提供测试名称", 'RED')
                        continue
                    test_name = args[0]
                    self.run_test(test_name)
                
                elif command in ['s', 'suite']:
                    if not args:
                        self.color_print("[ERROR] 请提供套件名称", 'RED')
                        continue
                    suite_name = args[0]
                    self.run_suite(suite_name)
                
                elif command in ['h', 'history']:
                    if not self.test_history:
                        print("暂无测试历史")
                        continue
                    
                    print("\n测试历史:")
                    for i, record in enumerate(self.test_history[-10:], 1):
                        status = "[PASS]" if record['success'] else "[FAIL]"
                        print(f"  {i}. {status} {record['name']} ({record['duration']:.2f}s)")
                
                elif command in ['c', 'clear']:
                    self.test_history.clear()
                    self.color_print("[OK] 历史已清空", 'GREEN')
                
                else:
                    self.color_print(f"[ERROR] 未知命令：{command}", 'RED')
                    self.show_menu()
                    
            except KeyboardInterrupt:
                print("\n")
                continue
            except EOFError:
                print("\n")
                break


def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Redra 交互式测试管理器')
    parser.add_argument('--list', '-l', action='store_true',
                       help='列出所有可用测试')
    parser.add_argument('--run', '-r', type=str, metavar='TEST',
                       help='运行指定测试')
    parser.add_argument('--batch', '-b', type=str, metavar='FILE',
                       help='从文件批量运行测试')
    parser.add_argument('--suite', '-s', type=str, metavar='SUITE',
                       choices=['quick', 'communication', 'float_label', 
                               'integration', 'stress', 'all'],
                       help='运行测试套件')
    parser.add_argument('--release', action='store_true',
                       help='使用 Release 模式')
    parser.add_argument('--interactive', '-i', action='store_true',
                       help='进入交互模式')
    
    args = parser.parse_args()
    
    manager = TestManager()
    
    if args.list:
        tests = manager.list_available_tests()
        print("\n可用测试:")
        for test in tests:
            print(f"  - {test}")
        return 0
    
    elif args.run:
        result = manager.run_test(args.run, args.release)
        return 0 if result['success'] else 1
    
    elif args.batch:
        batch_file = Path(args.batch)
        if not batch_file.exists():
            print(f"[ERROR] 文件不存在：{batch_file}")
            return 1
        
        with open(batch_file, 'r') as f:
            tests = [line.strip() for line in f if line.strip() and not line.startswith('#')]
        
        results = []
        for test in tests:
            result = manager.run_test(test, args.release)
            results.append(result)
        
        passed = sum(1 for r in results if r['success'])
        print(f"\n批量测试完成：{passed}/{len(results)} 通过")
        return 0 if passed == len(results) else 1
    
    elif args.suite:
        results = manager.run_suite(args.suite, args.release)
        passed = sum(1 for r in results if r['success'])
        return 0 if passed == len(results) else 1
    
    else:
        # 默认进入交互模式
        manager.interactive_mode()
        return 0


if __name__ == '__main__':
    sys.exit(main())
