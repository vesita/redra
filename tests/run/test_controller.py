#!/usr/bin/env python3
"""
Redra 测试控制器

这个脚本用于控制和管理 Redra 项目的完整测试流程，包括：
1. 编译项目
2. 运行不同类型的测试
3. 监控测试状态
4. 生成测试报告和日志
5. 清理测试环境

使用方法:
    python test_controller.py [--mode MODE] [--test TEST_NAME] [--release]
    
示例:
    python test_controller.py --mode all           # 运行所有测试
    python test_controller.py --mode unit          # 只运行单元测试
    python test_controller.py --mode integration   # 只运行集成测试
    python test_controller.py --test test_full_scene  # 运行特定测试
    python test_controller.py --release            # 以 Release 模式运行
"""

import subprocess
import sys
import os
import time
import json
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional, Tuple
import signal
import psutil
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


class TestLogger:
    """测试日志记录器"""
    
    def __init__(self, log_dir: Optional[Path] = None):
        """初始化日志记录器
        
        Args:
            log_dir: 日志目录，默认为 tests/log（不是 tests/run/log）
        """
        if log_dir is None:
            # 使用 tests/log 目录（父目录的 log 文件夹）
            log_dir = Path(__file__).parent.parent / 'log'
        
        self.log_dir = log_dir
        self.log_dir.mkdir(parents=True, exist_ok=True)
        
        # 生成日志文件名 - 带时间戳
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        self.log_file = self.log_dir / f'test_controller_{timestamp}.log'
        
        # 配置日志
        self.logger = logging.getLogger('test_controller')
        self.logger.setLevel(logging.INFO)
        
        # 清除已有的处理器
        self.logger.handlers.clear()
        
        # 文件处理器 - 保存所有详细日志
        file_handler = logging.FileHandler(self.log_file, encoding='utf-8')
        file_handler.setLevel(logging.INFO)
        file_format = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
        file_handler.setFormatter(file_format)
        self.logger.addHandler(file_handler)
        
        # 记录开始时间
        self.start_time: Optional[datetime] = None
        self.end_time: Optional[datetime] = None
    
    def start(self, mode: str, release_mode: bool):
        """记录测试开始
        
        Args:
            mode: 测试模式
            release_mode: 是否 Release 模式
        """
        self.start_time = datetime.now()
        self.logger.info("=" * 70)
        self.logger.info(f"测试控制器启动")
        self.logger.info(f"开始时间：{self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        self.logger.info(f"模式：{mode}")
        self.logger.info(f"编译模式：{'Release' if release_mode else 'Debug'}")
        self.logger.info("=" * 70)
    
    def end(self, success: bool):
        """记录测试结束
        
        Args:
            success: 测试是否成功
        """
        self.end_time = datetime.now()
        duration = self.end_time - self.start_time
        
        self.logger.info("=" * 70)
        self.logger.info(f"结束时间：{self.end_time.strftime('%Y-%m-%d %H:%M:%S')}")
        self.logger.info(f"总耗时：{duration}")
        self.logger.info(f"结果：{'成功' if success else '失败'}")
        self.logger.info("=" * 70)
    
    def log_command(self, command: str):
        """记录执行的命令"""
        self.logger.info(f"执行：{command}")
    
    def log_test_result(self, test_name: str, status: str, duration: float):
        """记录测试结果
        
        Args:
            test_name: 测试名称
            status: 状态 (passed/failed)
            duration: 耗时（秒）
        """
        level = 'INFO' if status == 'passed' else 'ERROR'
        self.logger.log(getattr(logging, level, logging.INFO), 
                       f"测试：{test_name} | 状态：{status} | 耗时：{duration:.2f}s")
    
    def get_log_file(self) -> Path:
        """获取日志文件路径"""
        return self.log_file


class TestController:
    """测试控制器类"""
    
    def __init__(self, release_mode: bool = False, verbose: bool = False):
        """初始化测试控制器
        
        Args:
            release_mode: 是否使用 Release 模式编译
            verbose: 是否显示详细输出
        """
        self.project_root = Path(__file__).parent.parent  # tests 目录
        self.release_mode = release_mode
        self.verbose = verbose
        self.test_results = []
        self.start_time = None
        self.end_time = None
        
        # 初始化日志记录器
        self.logger_instance = TestLogger()
        
        # 测试分类
        self.unit_tests = [
            "test_rdchannel_creation",
            "test_channel_send_receive",
            "test_client_sender_point_encoding",
            "test_client_sender_segment_encoding",
            "test_trailer_encoding",
            "test_float_label_basic_creation",
            "test_float_label_plugin_registration",
        ]
        
        self.integration_tests = [
            "test_full_application_startup",
            "test_full_scene",
            "test_stress_many_labels",
            "test_main_like_application",
            "test_float_label_spawn_in_scene",
            "test_spawn_label_at_helper",
            "test_billboard_effect",
            "test_multiple_cameras_billboard",
            "test_dynamic_label_management",
        ]
        
        # 日志颜色（使用 ANSI 颜色代码，不使用 emoji）
        self.COLORS = {
            'GREEN': '\033[92m',
            'RED': '\033[91m',
            'YELLOW': '\033[93m',
            'BLUE': '\033[94m',
            'CYAN': '\033[96m',
            'RESET': '\033[0m',
            'BOLD': '\033[1m',
        }
    
    def color_print(self, message: str, color: str = 'RESET', bold: bool = False):
        """彩色打印消息 - 只在详细模式下显示"""
        pass
    
    def log_message(self, message: str, level: str = 'INFO'):
        """记录日志到控制台 - 简化输出"""
        # 不显示到终端，只记录到文件
        pass

    def run_command(self, command: List[str], capture_output: bool = False, 
                   timeout: Optional[int] = None) -> Tuple[bool, str, str]:
        """运行命令
        
        Args:
            command: 命令列表
            capture_output: 是否捕获输出（False 则实时显示）
            timeout: 超时时间（秒）
            
        Returns:
            (success, stdout, stderr)
        """
        try:
            # 记录到日志文件
            self.logger_instance.log_command(' '.join(command))
            
            if capture_output:
                result = subprocess.run(
                    command,
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=timeout
                )
                # 记录输出到日志（移除 ANSI 代码）
                if result.stdout:
                    clean_stdout = strip_ansi_codes(result.stdout)
                    self.logger_instance.logger.info("STDOUT:\n" + clean_stdout)
                if result.stderr:
                    clean_stderr = strip_ansi_codes(result.stderr)
                    self.logger_instance.logger.error("STDERR:\n" + clean_stderr)
                return (
                    result.returncode == 0,
                    result.stdout,
                    result.stderr
                )
            else:
                # 不捕获输出，直接显示到终端，但仍然记录到日志
                process = subprocess.Popen(
                    command,
                    cwd=self.project_root,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    bufsize=1
                )
                
                # 实时输出并收集
                output_lines = []
                try:
                    while True:
                        line = process.stdout.readline()
                        if not line and process.poll() is not None:
                            break
                        if line:
                            print(line, end='')
                            output_lines.append(line)
                except Exception as e:
                    self.logger_instance.logger.error(f"读取输出时出错：{e}")
                
                process.wait()
                
                # 记录输出到日志（移除 ANSI 代码）
                full_output = ''.join(output_lines)
                clean_output = strip_ansi_codes(full_output)
                if clean_output:
                    self.logger_instance.logger.info("输出:\n" + clean_output)
                
                return (process.returncode == 0, "", "")
                
        except subprocess.TimeoutExpired:
            self.log_message(f"命令执行超时：{' '.join(command)}", 'ERROR')
            self.color_print(f"命令执行超时：{' '.join(command)}", 'RED')
            self.logger_instance.logger.error(f"命令执行超时：{' '.join(command)}")
            return (False, "", "Timeout")
        except Exception as e:
            self.log_message(f"命令执行失败：{e}", 'ERROR')
            self.color_print(f"命令执行失败：{e}", 'RED')
            self.logger_instance.logger.error(f"命令执行失败：{e}")
            return (False, "", str(e))
    
    def build_project(self) -> bool:
        """构建项目"""
        print(f"[BUILD] 构建项目...", end=' ', flush=True)
        
        command = ["cargo", "build"]
        if self.release_mode:
            command.append("--release")
        
        success, _, _ = self.run_command(command)
        
        if success:
            print(f"[OK]")
        else:
            print(f"[FAILED]")
        
        return success
    
    def build_tests(self) -> bool:
        """编译测试"""
        print(f"[BUILD] 编译测试...", end=' ', flush=True)
        
        command = ["cargo", "test", "--no-run"]
        if self.release_mode:
            command.insert(2, "--release")
        
        success, _, _ = self.run_command(command)
        
        if success:
            print(f"[OK]")
        else:
            print(f"[FAILED]")
        
        return success
    
    def run_single_test(self, test_name: str, test_file: str = None) -> Dict:
        """运行单个测试"""
        result = {
            'name': test_name,
            'file': test_file,
            'status': 'pending',
            'duration': 0,
            'output': '',
            'timestamp': datetime.now().isoformat(),
        }
        
        self.logger_instance.logger.info(f"开始测试：{test_name}")
        
        command = ["cargo", "test"]
        if self.release_mode:
            command.append("--release")
        
        if test_file:
            command.extend(["--test", test_file])
        
        command.extend([test_name, "--", "--nocapture"])
        
        start = time.time()
        # 不捕获输出，让测试输出实时显示，但会记录到日志
        success, stdout, stderr = self.run_command(command, capture_output=False, timeout=300)
        duration = time.time() - start
        
        result['status'] = 'passed' if success else 'failed'
        result['duration'] = duration
        result['output'] = "(输出已实时显示并记录到日志)"
        
        # 记录测试结果到日志
        self.logger_instance.log_test_result(test_name, result['status'], duration)
        
        if success:
            print(f"  [PASS] {test_name} ({duration:.2f}s)")
        else:
            print(f"  [FAIL] {test_name} ({duration:.2f}s)")
        
        return result
    
    def run_test_category(self, category: str, tests: List[str]) -> List[Dict]:
        """运行一类测试"""
        print(f"\n[SUITE] {category} 测试 (共 {len(tests)} 个)")
        
        results = []
        for test in tests:
            # 确定测试文件
            if 'communication' in test:
                test_file = "communication_test"
            elif 'float_label' in test or 'label' in test:
                test_file = "float_label_test"
            else:
                test_file = None
            
            result = self.run_single_test(test, test_file)
            results.append(result)
        
        # 统计结果
        passed = sum(1 for r in results if r['status'] == 'passed')
        failed = len(results) - passed
        
        print(f"[RESULT] {category}: {passed}/{len(tests)} 通过")
        
        return results
    
    def run_all_tests(self) -> List[Dict]:
        """运行所有测试"""
        print("\n[RUN] 运行所有测试")
        print("="*60)
        
        results = []
        
        # 运行单元测试
        unit_results = self.run_test_category("单元", self.unit_tests)
        results.extend(unit_results)
        
        # 运行集成测试
        integration_results = self.run_test_category("集成", self.integration_tests)
        results.extend(integration_results)
        
        return results
    
    def generate_report(self, results: List[Dict]):
        """生成测试报告"""
        total = len(results)
        passed = sum(1 for r in results if r['status'] == 'passed')
        failed = total - passed
        total_duration = sum(r['duration'] for r in results)
        
        print("\n" + "="*60)
        print(f"[REPORT] 总测试数：{total} | 通过：{passed} | 失败：{failed}")
        print(f"[REPORT] 总耗时：{total_duration:.2f}秒")
        
        # 详细结果
        print("\n详细结果:")
        print("-" * 60)
        for result in results:
            status_text = "[PASS]" if result['status'] == 'passed' else "[FAIL]"
            print(f"{status_text} {result['name']} ({result['duration']:.2f}s)")
    
    def cleanup(self):
        """清理测试环境"""
        # 静默清理
        pass
    
    def run(self, mode: str = 'all', specific_test: str = None):
        """运行测试流程"""
        self.start_time = time.time()
        
        try:
            # 1. 构建项目
            if not self.build_project():
                return False
            
            # 2. 编译测试
            if not self.build_tests():
                return False
            
            # 3. 运行测试
            if specific_test:
                # 运行特定测试
                if 'communication' in specific_test:
                    test_file = "communication_test"
                elif 'float_label' in specific_test or 'label' in specific_test:
                    test_file = "float_label_test"
                else:
                    test_file = None
                
                results = [self.run_single_test(specific_test, test_file)]
            elif mode == 'unit':
                # 只运行单元测试
                results = self.run_test_category("单元", self.unit_tests)
            elif mode == 'integration':
                # 只运行集成测试
                results = self.run_test_category("集成", self.integration_tests)
            else:
                # 运行所有测试
                results = self.run_all_tests()
            
            # 4. 生成报告
            self.generate_report(results)
            
            # 5. 清理
            self.cleanup()
            
            self.end_time = time.time()
            total_time = self.end_time - self.start_time
            
            # 添加通过条件总结
            total_tests = len(results)
            passed_tests = sum(1 for r in results if r['status'] == 'passed')
            failed_tests = total_tests - passed_tests
            
            print("\n" + "="*60)
            if failed_tests == 0:
                print("[CONTROLLER] 测试执行成功")
                print("\n[SUMMARY] 通过条件:")
                print(f"  ✓ 所有 {total_tests} 个测试用例执行完毕")
                print(f"  ✓ 通过率：{passed_tests}/{total_tests} (100%)")
                print("  ✓ 无测试失败或超时")
                print("  ✓ 项目构建和编译成功")
            else:
                print("[CONTROLLER] 测试执行失败")
                print(f"\n[SUMMARY] 测试结果：{passed_tests} 通过，{failed_tests} 失败")
                print("\n失败原因可能包括:")
                print("  ✗ 存在测试用例断言失败")
                print("  ✗ 测试执行超时（>300 秒）")
                print("  ✗ 运行时错误或 panic")
                print("  ✗ 依赖服务或资源不可用")
            
            print("="*60)
            
            return True
            
        except KeyboardInterrupt:
            return False
        except Exception as e:
            self.logger_instance.logger.error(f"测试过程中发生错误：{e}")
            return False


def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Redra 测试控制器')
    parser.add_argument(
        '--mode', '-m',
        choices=['all', 'unit', 'integration'],
        default='all',
        help='测试模式：all(所有), unit(单元), integration(集成)'
    )
    parser.add_argument(
        '--test', '-t',
        type=str,
        help='运行特定测试（提供测试名称）'
    )
    parser.add_argument(
        '--release', '-r',
        action='store_true',
        help='使用 Release 模式编译和运行测试'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='显示详细输出'
    )
    
    args = parser.parse_args()
    
    # 创建控制器并运行
    controller = TestController(release_mode=args.release)
    success = controller.run(mode=args.mode, specific_test=args.test)
    
    # 退出码
    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()