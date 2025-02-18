use stack_vec::StackVec;

use crate::console::{kprint, kprintln, CONSOLE};
use shim::io;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0] //first arg is always the path
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    let mut input_buf = [0u8; 512];
    let mut input_len = 0;

    loop {
        // Print the shell prompt
        kprint!("{}", prefix);

        // Reset input buffer and length for the current iteration
        input_len = 0;

        // Read each byte until newline
        let mut console = CONSOLE.lock();
        loop {
            let byte = console.read_byte();

            // Handle backspace/delete
            if byte == 8 || byte == 127 {
                if input_len > 0 {
                    input_len -= 1;
                    // Erase the character from the console
                    kprint!("\x08 \x08"); // Backspace, space, backspace
                }
                continue;
            }

            // Handle newline (end of input)
            if byte == b'\n' || byte == b'\r' {
                kprintln!(); // Move to a new line after Enter
                break;
            }

            // Echo the character and store in buffer if space is available
            if input_len < input_buf.len() {
                input_buf[input_len] = byte;
                input_len += 1;
                kprint!("{}", byte as char);
            } else {
                // Buffer is full, alert the user
                kprint!("\x07"); // Bell character
            }
        }

        // Convert input bytes to a string
        let input = core::str::from_utf8(&input_buf[..input_len])
            .unwrap_or("")

            // Trim the trailing newline and carriage return characters
            .trim_end_matches(|c| c == '\n' || c == '\r');

        if input.is_empty() {
            continue; // Skip processing if input is empty
        }

        // Create a new args buffer each iteration to avoid borrowing issues
        let mut args_buf = [""; 64]; // Moved inside the loop

        // Parse the command
        match Command::parse(input, &mut args_buf) {
            Ok(cmd) => {
                match cmd.path() {
                    "echo" => {
                        // Print all arguments after "echo"
                        let mut args_str = [0u8; 512]; // Buffer to hold the arguments as a string
                        let mut args_len = 0;

                        for &arg in cmd.args.iter().skip(1) {
                            for byte in arg.as_bytes() {
                                if args_len < args_str.len() {
                                    args_str[args_len] = *byte;
                                    args_len += 1;
                                }
                            }
                            // Add space between arguments
                            if args_len < args_str.len() {
                                args_str[args_len] = b' ';
                                args_len += 1;
                            }
                        }

                        // Convert the argument bytes to a string and print
                        let args = core::str::from_utf8(&args_str[..args_len]).unwrap_or("");
                        kprintln!("{}", args.trim_end());
                    }
                    // Add other commands here
                    _ => kprintln!("unknown command: {}", cmd.path()),
                }
            }
            Err(Error::Empty) => kprintln!("error: empty command"),
            Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
        }
    }
}
