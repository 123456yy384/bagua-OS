//! ELF64 加载器
//!
//! 解析 ELF 可执行文件，映射 LOAD 段到内存，返回入口地址。

use core::mem;

/// ELF64 头
#[repr(C)]
pub struct Elf64Header {
    pub ident: [u8; 16],
    pub etype: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// ELF64 程序头
#[repr(C)]
pub struct Elf64ProgramHeader {
    pub ptype: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

// ELF 常量
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELF_CLASS_64: u8 = 2;
const ET_EXEC: u16 = 2;
const PT_LOAD: u32 = 1;
const EM_X86_64: u16 = 62;

/// 加载结果
pub struct LoadedProgram {
    /// 入口地址（虚拟地址）
    pub entry_point: u64,
    /// 加载的段数量
    pub segments_loaded: usize,
}

/// ELF 加载器
pub struct ElfLoader;

impl ElfLoader {
    /// 加载 ELF 可执行文件
    ///
    /// 返回程序入口地址。
    /// 当前为解析器骨架，实际内存映射需要页表支持。
    pub fn load(elf_data: &[u8]) -> Result<LoadedProgram, ElfError> {
        // 验证 ELF header
        if elf_data.len() < mem::size_of::<Elf64Header>() {
            return Err(ElfError::TooSmall);
        }

        let header: &Elf64Header = unsafe { &*(elf_data.as_ptr() as *const Elf64Header) };

        // 验证魔数
        if header.ident[0..4] != ELF_MAGIC {
            return Err(ElfError::BadMagic);
        }
        // 验证 64 位
        if header.ident[4] != ELF_CLASS_64 {
            return Err(ElfError::Not64Bit);
        }
        // 验证可执行文件
        if header.etype != ET_EXEC {
            return Err(ElfError::NotExecutable);
        }
        // 验证 x86_64
        if header.machine != EM_X86_64 {
            return Err(ElfError::WrongArchitecture);
        }

        let entry_point = header.entry;

        // 遍历程序头，加载 LOAD 段
        let phoff = header.phoff as usize;
        let phentsize = header.phentsize as usize;
        let phnum = header.phnum as usize;

        let mut segments_loaded = 0;

        for i in 0..phnum {
            let ph_addr = phoff + i * phentsize;
            if ph_addr + mem::size_of::<Elf64ProgramHeader>() > elf_data.len() {
                break;
            }

            let ph: &Elf64ProgramHeader = unsafe {
                &*(elf_data.as_ptr().add(ph_addr) as *const Elf64ProgramHeader)
            };

            if ph.ptype == PT_LOAD {
                let filesz = ph.filesz as usize;
                let memsz = ph.memsz as usize;
                let offset = ph.offset as usize;
                let vaddr = ph.vaddr;

                if offset + filesz > elf_data.len() {
                    return Err(ElfError::SegmentOutOfBounds);
                }

                // TODO: 实际内存映射
                // 1. 分配物理页帧
                // 2. 映射 vaddr → phys pages
                // 3. memcpy elf_data[offset..offset+filesz] → mapped memory
                // 4. memsz > filesz → zero-fill .bss

                let _ = (filesz, memsz, offset, vaddr);
                segments_loaded += 1;
            }
        }

        Ok(LoadedProgram {
            entry_point,
            segments_loaded,
        })
    }
}

#[derive(Debug)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    Not64Bit,
    NotExecutable,
    WrongArchitecture,
    SegmentOutOfBounds,
}
