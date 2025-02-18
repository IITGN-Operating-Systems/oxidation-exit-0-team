use core::fmt;   // formatted output
use core::time::Duration;  //represent durations of time, useful for setting read timeouts.

use shim::io;
use shim::const_assert_size;

use volatile::prelude::*;  //crate for working with volatile memory-mapped I/O.
use volatile::{Volatile, ReadVolatile, Reserved};

use crate::timer;
use crate::common::IO_BASE;
use crate::gpio::{Gpio, Function};

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;  // base address for the mini UART's registers

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;   //register used to enable auxiliary devices, including the mini UART. It's a pointer to a volatile memory location.

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {    // Line Status Register contains flags to indicate the status of the UART
    DataReady = 1,  // Data is ready to be read
    TxAvailable = 1 << 5,   // UART transmitter is ready to send data.
}

#[repr(C)]
#[allow(non_snake_case)]

struct Registers {
    IO: Volatile<u8>,
    __r0: [Reserved<u8>; 3],
    IER: Volatile<u8>,
    __r1: [Reserved<u8>; 3],
    IIR: Volatile<u8>,
    __r2: [Reserved<u8>; 3],
    LCR: Volatile<u8>,
    __r3: [Reserved<u8>; 3],
    MCR: Volatile<u8>,
    __r4: [Reserved<u8>; 3],
    LSR: ReadVolatile<u8>,
    __r5: [Reserved<u8>; 3],
    MSR: Volatile<u8>,
    __r6: [Reserved<u8>; 3],
    SCRATCH: Volatile<u8>,
    __r7: [Reserved<u8>; 3],
    CNTL: Volatile<u8>,
    __r8: [Reserved<u8>; 3],
    STAT: Volatile<u32>,
    BAUD: Volatile<u16>,
    __r9: [Reserved<u8>; 2],
}

const_assert_size!(Registers, 0x7E21506C - 0x7E215040);

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,  // pointer to the mini UART's registers
    timeout: Option<Duration>,          // read timeout
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // FIXME: Implement remaining mini UART initialization.
        // unimplemented!()
        registers.CNTL.write(0);
        registers.IER.write(0);
        registers.LCR.write(0b11);
        registers.MCR.write(0);
        registers.BAUD.write(270);

        let gpio14 = Gpio::new(14).into_alt(Function::Alt5);
        let gpio15 = Gpio::new(15).into_alt(Function::Alt5);

        registers.IIR.write(0b11);
        registers.CNTL.write(0b11);
        MiniUart { registers, timeout: None }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        while !self.registers.LSR.has_mask((LsrStatus::TxAvailable as u8).into()) {}
        self.registers.IO.write(byte);
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        self.registers.LSR.has_mask((LsrStatus::DataReady as u8).into())
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        let start = timer::current_time();
        while !self.has_byte() {
            if let Some(timeout) = self.timeout {
                if timer::current_time() - start > timeout {
                    return Err(()); // Timeout expired
                }
            }
        }
        Ok(())
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        // unimplemented!()
        while !self.has_byte() {} // Wait for data
        self.registers.IO.read()
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.
impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r'); // Convert '\n' to '\r\n'
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

mod uart_io {
    use super::io;
    use super::MiniUart;
    use volatile::prelude::*;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //

    impl io::Read for MiniUart {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if buf.is_empty() {
                return Ok(0);
            }

            match self.wait_for_byte() {
                Ok(()) => {
                    let mut count = 0;
                    while count < buf.len() && self.has_byte() {
                        buf[count] = self.read_byte();
                        count += 1;
                    }
                    Ok(count)
                }
                Err(()) => Err(io::Error::new(io::ErrorKind::TimedOut, "read timed out")),
            }
        }
    }
    
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
    impl io::Write for MiniUart {
        fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
            for &byte in buf {
                self.write_byte(byte);
            }
            Ok(buf.len())
        }
    
        fn flush(&mut self) -> Result<(), io::Error> {
            Ok(())
        }
    }
    
}
