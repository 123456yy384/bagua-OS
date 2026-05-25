//! 串口输出 — COM1 调试日志
//!
//! 通过串口 (0x3F8) 输出内核日志。
//! QEMU 使用 `-serial stdio` 查看。

use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

const COM1: u16 = 0x3F8;

pub(crate) static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::new());

pub(crate) struct SerialPort {
    data: PortWriteOnly<u8>,
    status: PortReadOnly<u8>,
}

impl SerialPort {
    const fn new() -> Self {
        Self {
            data: PortWriteOnly::new(COM1),
            status: PortReadOnly::new(COM1 + 5),
        }
    }

    fn write_byte(&mut self, byte: u8) {
        unsafe {
            while (self.status.read() & 0x20) == 0 {}
            self.data.write(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// 初始化串口
pub fn init() {
    unsafe {
        let mut int_enable = PortWriteOnly::<u8>::new(COM1 + 1);
        let mut line_ctrl = PortWriteOnly::<u8>::new(COM1 + 3);
        let mut modem_ctrl = PortWriteOnly::<u8>::new(COM1 + 4);

        // 禁用中断
        int_enable.write(0x00u8);
        // DLAB = 1
        line_ctrl.write(0x80u8);
        // 波特率 38400 (divisor = 3)
        PortWriteOnly::<u8>::new(COM1 + 0).write(0x03u8);
        PortWriteOnly::<u8>::new(COM1 + 1).write(0x00u8);
        // 8N1
        line_ctrl.write(0x03u8);
        // DTR + RTS
        modem_ctrl.write(0x03u8);
    }
}

/// 格式化输出到串口
#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::serial::SERIAL.lock(), $($arg)*);
        let _ = write!($crate::serial::SERIAL.lock(), "\n");
    }};
}

/// 带格式的串口输出函数
pub fn print(args: fmt::Arguments) {
    use fmt::Write;
    SERIAL.lock().write_fmt(args).ok();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        $crate::serial::print(format_args!($($arg)*));
    }};
}
