#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use shell::shell;
use console::kprintln;

/// The entry point of the kernel.
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    kprintln!("Starting kernel shell...");
    shell("> "); // Start the shell with a simple prompt
}