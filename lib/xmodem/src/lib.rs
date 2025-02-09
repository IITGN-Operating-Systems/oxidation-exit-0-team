#![cfg_attr(feature = "no_std", no_std)]

#![feature(decl_macro)]

use shim::io;
use shim::ioerr;

#[cfg(test)] mod tests;
mod read_ext;
mod progress;

pub use progress::{Progress, ProgressFn};

use read_ext::ReadExt;

const SOH: u8 = 0x01;
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;

/// Implementation of the XMODEM protocol.
pub struct Xmodem<R> {
    packet: u8,
    started: bool,
    inner: R,
    progress: ProgressFn
}

impl Xmodem<()> {
    #[inline]
    pub fn transmit<R, W>(data: R, to: W) -> io::Result<usize>
        where W: io::Read + io::Write, R: io::Read
    {
        Xmodem::transmit_with_progress(data, to, progress::noop)
    }

    pub fn receive_with_progress<R, W>(from: R, mut into: W, f: ProgressFn) -> io::Result<usize>
    where R: io::Read + io::Write, W: io::Write
    {
        let mut receiver = Xmodem::new_with_progress(from, f);
        let mut packet = [0u8; 128];
        let mut received = 0;

        // Send initial NAK to initiate transfer
        receiver.write_byte(NAK)?;

        'next_packet: loop {
            for _ in 0..10 {
                match receiver.read_packet(&mut packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(0) => break 'next_packet,
                    Ok(n) => {
                        received += n;
                        into.write_all(&packet)?;
                        continue 'next_packet;
                    }
                }
            }
            return ioerr!(BrokenPipe, "bad receive");
        }

        Ok(received)
    }

    #[inline]
    pub fn receive<R, W>(from: R, into: W) -> io::Result<usize>
       where R: io::Read + io::Write, W: io::Write
    {
        Xmodem::receive_with_progress(from, into, progress::noop)
    }

    pub fn transmit_with_progress<R, W>(mut data: R, to: W, f: ProgressFn) -> io::Result<usize>
    where W: io::Read + io::Write, R: io::Read
    {
        let mut transmitter = Xmodem::new_with_progress(to, f);
        let mut packet = [0u8; 128];
        let mut written = 0;

        'next_packet: loop {
            let n = data.read_max(&mut packet)?;
            packet[n..].iter_mut().for_each(|b| *b = 0);

            if n == 0 {
                transmitter.write_packet(&[])?;
                return Ok(written);
            }

            for _ in 0..10 {
                match transmitter.write_packet(&packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(_) => {
                        written += n;
                        continue 'next_packet;
                    }
                }
            }

            return ioerr!(BrokenPipe, "bad transmit");
        }
    }
}

fn get_checksum(buf: &[u8]) -> u8 {
    return buf.iter().fold(0, |a, b| a.wrapping_add(*b));
}

impl<T: io::Read + io::Write> Xmodem<T> {
   
    pub fn new(inner: T) -> Self {
        Xmodem { packet: 1, started: false, inner, progress: progress::noop}
    }

    pub fn new_with_progress(inner: T, f: ProgressFn) -> Self {
        Xmodem { packet: 1, started: false, inner, progress: f }
    }
 
    fn read_byte(&mut self, abort_on_can: bool) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf)?;

        let byte = buf[0];
        if abort_on_can && byte == CAN {
            return ioerr!(ConnectionAborted, "received CAN");
        }

        Ok(byte)
    }

   
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.inner.write_all(&[byte])?;
        self.inner.flush()
    }

    fn expect_byte_or_cancel(&mut self, byte: u8, expected: &'static str) -> io::Result<u8> {
        let received = self.read_byte(false)?;
        if received == byte {
            Ok(received)
        } else {
            self.write_byte(CAN)?;
            if received == CAN {
                Err(io::Error::new(io::ErrorKind::ConnectionAborted, "received CAN"))
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, expected))
            }
        }
    }

    fn expect_byte(&mut self, byte: u8, expected: &'static str) -> io::Result<u8> {
        let received = self.read_byte(false)?;
        if received == byte {
            Ok(received)
        } else if received == CAN {
            Err(io::Error::new(io::ErrorKind::ConnectionAborted, "received CAN"))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, expected))
        }
    }

    pub fn read_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 128 {
            return ioerr!(UnexpectedEof, "buffer too small");
        }

        let byte = self.read_byte(false)?;

        if byte == CAN {
            return ioerr!(ConnectionAborted, "received CAN");
        }

        match byte {
            SOH => {
                // Mark started only on first SOH
                if !self.started {
                    self.started = true;
                    (self.progress)(Progress::Started);
                }

                let packet_num = self.read_byte(false)?;
                let packet_num_neg = self.read_byte(false)?;

                // Ensure self.packet starts at 1
                if packet_num != self.packet || packet_num_neg != !self.packet {
                    self.write_byte(NAK)?;
                    return ioerr!(InvalidData, "packet number mismatch");
                }

                self.inner.read_exact(&mut buf[..128])?;
                let checksum = self.read_byte(false)?;

                if get_checksum(&buf[..128]) != checksum {
                    self.write_byte(NAK)?;
                    return ioerr!(Interrupted, "checksum mismatch");
                }

                self.write_byte(ACK)?;
                // Report current packet before incrementing
                (self.progress)(Progress::Packet(packet_num));
                self.packet = self.packet.wrapping_add(1);
                Ok(128)
            }
            EOT => {
                self.write_byte(NAK)?;
                let byte = self.read_byte(false)?;
                if byte != EOT {
                    return ioerr!(InvalidData, "expected second EOT");
                }
                self.write_byte(ACK)?;
                Ok(0)
            }
            _ => {
                let next_byte = self.read_byte(false)?;
                if next_byte == CAN {
                    return ioerr!(ConnectionAborted, "received CAN");
                } else {
                    self.write_byte(NAK)?;
                    return ioerr!(InvalidData, "expected SOH or EOT");
                }
            }
        }
    }
    
    pub fn write_packet(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() != 128 && !buf.is_empty() {
            return ioerr!(UnexpectedEof, "buffer length must be 128 or 0");
        }
    
        if !self.started {
            (self.progress)(Progress::Waiting);
            self.expect_byte(NAK, "expected NAK to start transmission")?;
            self.started = true;
            (self.progress)(Progress::Started);
        }
    
        if buf.is_empty() {
            self.write_byte(EOT)?;
            self.expect_byte(NAK, "expected NAK after first EOT")?;
            self.write_byte(EOT)?;
            self.expect_byte(ACK, "expected ACK after second EOT")?;
            return Ok(0);
        }
    
        self.write_byte(SOH)?;
        self.write_byte(self.packet)?;
        self.write_byte(!self.packet)?;
        self.inner.flush()?;
    
        self.inner.write_all(buf)?;
        self.write_byte(get_checksum(buf))?;
    
        match self.read_byte(false)? {
            ACK => {
                self.packet = self.packet.wrapping_add(1);
                (self.progress)(Progress::Packet(self.packet));
                Ok(128)
            }
            NAK => ioerr!(Interrupted, "checksum failed"),
            CAN => ioerr!(ConnectionAborted, "connection aborted by receiver"),
            _ => ioerr!(InvalidData, "expected ACK, NAK, or CAN"),
        }
    }   

    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
