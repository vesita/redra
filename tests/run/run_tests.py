#!/usr/bin/env python3
"""
Redra 测试快速启动脚本

这个脚本提供最常用的测试命令快捷方式。

使用方法:
    python run_tests.py              # 运行所有测试
    python run_tests.py quick        # 运行快速测试
    python run_tests.py integration  # 运行集成测试
    python run_tests.py release      # 以 Release 模式运行所有测试

输出:
    所有测试结果将直接在控制台显示，带进度条和详细信息。
    同时日志会被保存到 tests/log/ 目录下。
"""

import datetime
import subprocess
import sys
from pathlib import Path
import logging
from typing import Optional
import re
import os


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


try:
    from tqdm import tqdm
    HAS_TQDM = True
except ImportError:
    HAS_TQDM = False
    print("[WARNING] 未安装 tqdm，进度条功能将被禁用")
    print("   安装：pip install tqdm\n")


class TestLogger:
    """测试日志记录器"""
    
    MAX_LOG_FILES = 30  # 最多保留 30 个日志文件
    
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
        
        # 生成日志文件名
        timestamp = datetime.datetime.now().strftime('%Y%m%d_%H%M%S')
        mode = sys.argv[1] if len(sys.argv) > 1 else 'all'
        mode_str = f"_{mode}" if mode != 'all' else ""
        self.log_file = self.log_dir / f'test_run{mode_str}_{timestamp}.log'
        
        # 配置日志 - 统一使用 INFO 级别记录所有信息
        self.logger = logging.getLogger('test_runner')
        self.logger.setLevel(logging.INFO)
        
        # 文件处理器 - 保存所有详细日志
        file_handler = logging.FileHandler(self.log_file, encoding='utf-8')
        file_handler.setLevel(logging.INFO)
        file_format = logging.Formatter('%(asctime)s - %(message)s')
        file_handler.setFormatter(file_format)
        
        # 只添加文件处理器，控制台输出由 print 负责
        self.logger.addHandler(file_handler)
        
        # 记录开始时间
        self.start_time: Optional[datetime.datetime] = None
        self.end_time: Optional[datetime.datetime] = None
        
        # 清理旧日志文件
        self._cleanup_old_logs()
    
    def _cleanup_old_logs(self):
        """清理旧的日志文件，保持最多 MAX_LOG_FILES 个文件"""
        try:
            # 获取所有日志文件
            all_files = list(self.log_dir.glob('*.log'))
            
            if len(all_files) <= self.MAX_LOG_FILES:
                return
            
            # 按修改时间排序，最旧的在前
            all_files.sort(key=lambda f: f.stat().st_mtime)
            
            # 计算需要删除的文件数量
            files_to_delete = len(all_files) - self.MAX_LOG_FILES
            
            # 删除最旧的文件
            deleted_count = 0
            for i in range(files_to_delete):
                try:
                    all_files[i].unlink()
                    deleted_count += 1
                except Exception as e:
                    self.logger.info(f"[CLEANUP] 无法删除日志文件 {all_files[i]}: {e}")
            
            if deleted_count > 0:
                self.logger.info(f"[CLEANUP] 已清理 {deleted_count} 个旧日志文件，保留最近的 {self.MAX_LOG_FILES} 个")
                
        except Exception as e:
            # 清理失败不影响正常使用
            self.logger.info(f"[CLEANUP] 日志清理失败：{e}")
    
    def start(self):
        """记录测试开始"""
        self.start_time = datetime.datetime.now()
        self.logger.info("=" * 70)
        self.logger.info(f"[TIME] 测试开始时间：{self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        self.logger.info("=" * 70)
    
    def end(self, success: bool):
        """记录测试结束
        
        Args:
            success: 测试是否成功
        """
        self.end_time = datetime.datetime.now()
        duration = self.end_time - self.start_time
        
        self.logger.info("=" * 70)
        self.logger.info(f"[TIME] 测试结束时间：{self.end_time.strftime('%Y-%m-%d %H:%M:%S')}")
        self.logger.info(f"[INFO] 总耗时：{duration}")
        self.logger.info(f"[INFO] 结果：{'成功' if success else '失败'}")
        self.logger.info("=" * 70)
    
    def log_command(self, cmd: list):
        """记录执行的命令
        
        Args:
            cmd: 命令列表
        """
        self.logger.info(f"[INFO] 执行命令：{' '.join(cmd)}")
    
    def log_output(self, output: str, level: str = 'INFO'):
        """记录输出内容（自动清理 ANSI 代码）
        
        Args:
            output: 输出内容
            level: 日志级别（统一使用 INFO，不再区分级别）
        """
        # 移除 ANSI 颜色代码后再记录
        clean_output = strip_ansi_codes(output)
        
        # 统一使用 INFO 级别记录所有信息
        self.logger.info(clean_output.rstrip())
    
    def log_pass_rate(self, passed: int, total: int, duration: Optional[float] = None):
        """记录通过率信息
        
        Args:
            passed: 通过的测试数
            total: 总测试数
            duration: 总耗时（秒）
        """
        pass_rate = (passed / total * 100) if total > 0 else 0.0
        avg_duration = (duration / total) if total > 0 and duration else 0.0
        
        self.logger.info("=" * 70)
        self.logger.info(f"[REPORT] 测试结果统计")
        self.logger.info(f"[REPORT] 通过：{passed}/{total} ({pass_rate:.1f}%)")
        if duration:
            self.logger.info(f"[REPORT] 总耗时：{duration:.2f}秒")
            self.logger.info(f"[REPORT] 平均耗时：{avg_duration:.2f}秒/测试")
        self.logger.info("=" * 70)
    
    def get_log_file(self) -> Path:
        """获取日志文件路径
        
        Returns:
            日志文件路径
        """
        return self.log_file


def run_command(cmd, description="", show_progress=True, logger: Optional[TestLogger] = None):
    """执行命令并返回是否成功"""
    start_time = datetime.datetime.now()
    
    # 只显示简短提示
    print(f"[RUN] {description if description else ' '.join(cmd)}", end=' ', flush=True)
    
    # 记录到日志（详细）
    if logger:
        logger.log_command(cmd)
        if description:
            logger.log_output(f"描述：{description}")
    
    process = subprocess.Popen(
        cmd, 
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        cwd=Path(__file__).parent,
        bufsize=1,
        text=True
    )
    
    output_lines = []
    try:
        while True:
            line = process.stdout.readline()
            if not line and process.poll() is not None:
                break
            if line:
                # 不显示到终端，只收集用于记录
                output_lines.append(line)
    except Exception as e:
        pass
    
    process.wait()
    
    end_time = datetime.datetime.now()
    duration = end_time - start_time
    
    # 记录到日志
    if logger:
        full_output = ''.join(output_lines)
        logger.log_output(full_output)
        logger.log_output(f"耗时：{duration}")
        logger.log_output(f"状态码：{process.returncode}")
        
        # 解析通过率信息
        # 匹配格式："通过率：X/Y" 或 "X/Y 通过"
        passed_total_match = re.search(r'通过率 [::]\s*(\d+)\s*/\s*(\d+)|(\d+)\s*/\s*(\d+)\s+通过', full_output)
        failed_match = re.search(r'(\d+)\s+失败|失败 [::]\s*(\d+)', full_output)
        duration_match = re.search(r'总耗时 [::]\s*([\d.]+)', full_output)
        
        if passed_total_match:
            # 根据匹配组确定 passed 和 total 的值
            if passed_total_match.group(1):  # 格式：通过率：X/Y
                passed = int(passed_total_match.group(1))
                total = int(passed_total_match.group(2))
            else:  # 格式：X/Y 通过
                passed = int(passed_total_match.group(3))
                total = int(passed_total_match.group(4))
            
            # 提取失败数
            failed = 0
            if failed_match:
                failed = int(failed_match.group(1) or failed_match.group(2) or 0)
            
            total_duration = None
            if duration_match:
                total_duration = float(duration_match.group(1))
            
            # 记录通过率信息
            logger.log_pass_rate(passed, total, total_duration)
    
    # 只显示结果
    if process.returncode != 0:
        print(f"[FAILED] ({duration})")
        if logger:
            logger.log_output(f"命令失败：{' '.join(cmd)}")
        return False
    else:
        print(f"[OK] ({duration})")
        if logger:
            logger.log_output("命令成功完成")
        return True


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else 'all'
    
    logger = TestLogger()
    logger.start()
    
    # 只显示基本信息
    print(f"[INFO] Redra 测试 | 模式：{mode} | 日志：{logger.get_log_file()}")
    print("="*60)
    
    success = False
    
    if mode == 'quick':
        success = run_command(
            ['python', 'test_manager.py', '--suite', 'quick'],
            "运行快速测试套件",
            logger=logger
        )
    
    elif mode == 'unit':
        success = run_command(
            ['python', 'test_controller.py', '--mode', 'unit'],
            "运行单元测试",
            logger=logger
        )
    
    elif mode == 'integration':
        success = run_command(
            ['python', 'test_manager.py', '--suite', 'integration'],
            "运行集成测试（会打开图形窗口）",
            logger=logger
        )
    
    elif mode == 'stress':
        success = run_command(
            ['python', 'test_manager.py', '--suite', 'stress', '--release'],
            "运行压力测试（Release 模式）",
            logger=logger
        )
    
    elif mode == 'release':
        success = run_command(
            ['python', 'test_controller.py', '--mode', 'all', '--release'],
            "运行所有测试（Release 模式）",
            logger=logger
        )
    
    elif mode == 'demo':
        success = run_command(
            ['python', 'test_manager.py', '--run', 'test_full_scene'],
            "运行演示测试（查看 FloatLabel 效果）",
            logger=logger
        )
    
    elif mode == 'all':
        success = run_command(
            ['python', 'test_controller.py', '--mode', 'all'],
            "运行所有测试",
            logger=logger
        )
    
    elif mode == 'interactive':
        success = run_command(
            ['python', 'test_manager.py'],
            "进入交互式测试管理",
            logger=logger
        )
    
    elif mode == 'help':
        print("""
可用模式:
  quick       - 快速测试（仅基础单元测试，< 5 秒）
  unit        - 单元测试（不打开窗口）
  integration - 集成测试（打开窗口查看效果）
  stress      - 压力测试（性能测试）
  release     - Release 模式运行所有测试
  demo        - 演示模式（运行单个可视化测试）
  all         - 运行所有测试（默认）
  interactive - 进入交互模式
  help        - 显示帮助信息
""")
        success = True
        logger.log_output("显示帮助信息")
    
    else:
        print(f"[ERROR] 未知模式：{mode}")
        print("使用 'help' 查看可用选项")
        success = False
        logger.log_output(f"未知模式：{mode}", 'ERROR')
    
    logger.end(success)
    
    # 简洁的总结
    print("="*60)
    if success:
        print(f"[COMPLETE] 测试完成 | 日志：{logger.get_log_file()}")
        print("\n[SUMMARY] 通过条件:")
        print("  所有请求的测试用例执行完毕")
        print("  无测试失败或超时")
        print("  项目构建成功")
        print("  测试环境正常清理")
    else:
        print(f"[FAILED] 测试失败 | 请查看日志：{logger.get_log_file()}")
        print("\n[SUMMARY] 失败原因可能包括:")
        print("  存在测试用例失败或断言错误")
        print("  测试执行超时（>300 秒）")
        print("  项目构建或编译失败")
        print("  测试环境清理异常")
        print("  未知错误或中断")
    
    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()