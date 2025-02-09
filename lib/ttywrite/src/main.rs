mod parsers;

use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::Xmodem;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader};

    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("Failed to open serial port");

    // Create and configure serial port settings
    let mut settings = port.read_settings().expect("failed to get settings");
    settings.set_baud_rate(opt.baud_rate).expect("Failed to set baud rate");
    settings.set_char_size(opt.char_width); 
    settings.set_stop_bits(opt.stop_bits);
    settings.set_flow_control(opt.flow_control); 
    port.set_timeout(Duration::from_secs(opt.timeout)).expect("failed to set timeout");; 
    port.write_settings(&settings).expect("failed to write settings");

    // Handle input source
    let mut input: Box<dyn io::Read> = match opt.input {
        Some(path) => Box::new(File::open(path).expect("Failed to open input file")),
        None => Box::new(io::stdin()),
    };

    // Handle transmission mode
    if opt.raw {
        let bytes_written = io::copy(&mut input, &mut port).expect("Failed to write data");
        println!("wrote {} bytes", bytes_written);
    } else {
        let progress = |p| println!("Progress: {:?}", p);
        let bytes_written = Xmodem::transmit_with_progress(
            &mut *input,
            &mut port,
            progress)
            .expect("XMODEM transmission failed");
        println!("wrote {} bytes", bytes_written);
    }
}