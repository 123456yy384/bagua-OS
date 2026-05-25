//! 系统调用表
//!
//! 八卦 OS 系统调用接口。
//! 通过 `syscall` 指令（x86_64）触发，从用户态切换到内核态。

/// 系统调用号
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum SyscallNumber {
    Exit = 1,
    Write = 2,
    Read = 3,
    Fork = 4,
    Exec = 5,
    Yield = 6,
    GetPid = 7,
    SchedInfo = 8, // 八卦调度信息查询
}

/// 系统调用处理器
pub struct SyscallHandler;

impl SyscallHandler {
    /// 处理系统调用
    ///
    /// 参数约定（x86_64 System V ABI）：
    ///   rax = syscall number
    ///   rdi = arg1, rsi = arg2, rdx = arg3
    ///   返回值放入 rax
    pub fn handle(
        num: u64,
        _arg1: u64,
        _arg2: u64,
        _arg3: u64,
    ) -> u64 {
        match num {
            1 => {
                // exit(status)
                // TODO: 终止当前进程
                0
            }
            2 => {
                // write(fd, buf, len)
                // TODO: VFS 写入
                0
            }
            3 => {
                // read(fd, buf, len)
                // TODO: VFS 读取
                0
            }
            4 => {
                // fork() → child_pid
                // TODO: ProcessSpawner::fork_cow()
                0
            }
            5 => {
                // exec(path)
                // TODO: ElfLoader::load() + switch context
                u64::MAX // 失败
            }
            6 => {
                // yield() → reschedule
                0
            }
            7 => {
                // getpid()
                // TODO: 返回当前进程 PID
                0
            }
            8 => {
                // sched_info() → pressure * 1000
                // 八卦特有：查询调度器状态
                // TODO: 读取 SCHEDULER.health.logic_pressure
                1000
            }
            _ => u64::MAX, // 未知系统调用
        }
    }
}
