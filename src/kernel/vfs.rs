//! 简易 VFS — 虚拟文件系统骨架
//!
//! 提供统一的文件操作接口，支持：
//!   - 匿名管道 (pipe)
//!   - 内存文件 (tmpfs)
//!   - 设备文件 (devfs)
//!
//! TODO: 持久化文件系统 (ext2/fat32)

/// 文件描述符
pub type Fd = u32;

/// 文件操作接口
pub trait FileOps {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, VfsError>;
    fn seek(&mut self, _pos: u64) -> Result<u64, VfsError> { Err(VfsError::NotSupported) }
    fn close(&mut self) -> Result<(), VfsError> { Ok(()) }
}

/// VFS 错误
#[derive(Debug)]
pub enum VfsError {
    NotSupported,
    NotFound,
    PermissionDenied,
    IoError,
}

/// 文件类型
pub enum FileType {
    Regular,
    Directory,
    Device,
    Pipe,
}

/// 打开的文件描述
pub struct OpenFile {
    pub fd: Fd,
    pub file_type: FileType,
    pub pos: u64,
    pub ops: Box<dyn FileOps>,
}

/// 简易 VFS
pub struct Vfs {
    /// 打开文件表（固定大小）
    files: heapless::Vec<OpenFile, 64>,
    /// 下一个 FD 号
    next_fd: Fd,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            files: heapless::Vec::new(),
            next_fd: 0,
        }
    }

    /// 注册一个文件/设备
    pub fn register(&mut self, file_type: FileType, ops: Box<dyn FileOps>) -> Fd {
        let fd = self.next_fd;
        self.next_fd += 1;

        let _ = self.files.push(OpenFile {
            fd,
            file_type,
            pos: 0,
            ops,
        });

        fd
    }

    /// 读取文件
    pub fn read(&mut self, fd: Fd, buf: &mut [u8]) -> Result<usize, VfsError> {
        for f in &mut self.files {
            if f.fd == fd {
                return f.ops.read(buf);
            }
        }
        Err(VfsError::NotFound)
    }

    /// 写入文件
    pub fn write(&mut self, fd: Fd, buf: &[u8]) -> Result<usize, VfsError> {
        for f in &mut self.files {
            if f.fd == fd {
                return f.ops.write(buf);
            }
        }
        Err(VfsError::NotFound)
    }

    /// 关闭文件
    pub fn close(&mut self, fd: Fd) -> Result<(), VfsError> {
        if let Some(idx) = self.files.iter().position(|f| f.fd == fd) {
            let mut file = self.files.remove(idx);
            file.ops.close()
        } else {
            Err(VfsError::NotFound)
        }
    }
}


