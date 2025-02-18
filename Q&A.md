# Subphase B

## 1. Why can’t you write to CLO or CHI? (restricted-reads)

- The BCM2837 documentation says that CLO and CHI are read-only registers.  
- In Rust, we use `ReadVolatile<u32>` for these registers. This ensures that only reading is allowed and writing is not possible.

## 2. What prevents us from writing to CLO or CHI?

The `ReadVolatile<u32>` type is used for these registers, which means Rust will not allow us to call `.write()` on them. If we try, the compiler will give an error.

# Subphase E

## 1. Why should we never return an &mut T directly?

- If we return `&mut T` directly, it can lead to dangling references (pointing to memory that is no longer valid). This happens if the original data is dropped while the reference is still in use.  
- Instead, wrapping it in a container (which implements `Drop`) makes sure that the reference is properly managed and doesn't cause issues.

## 2. Where does the write_fmt call go?

The `_print` function calls `write_fmt` on `MutexGuard<Console>`. This means the actual `write_fmt` method being used belongs to the `Console` type. The method comes from the `core::fmt::Write` trait, which helps format and write text to the console.

## 3. How does your shell tie the many pieces together?

Our shell ties everything together by handling user input, parsing commands, executing them, and displaying output.

### User Input Handling
- It reads input from the `CONSOLE` one byte at a time and stores it in a buffer.
- It processes backspace/delete properly and handles newlines to detect when the user finishes typing.
- It converts the input into a string for easier processing.

### Command Parsing
- The `Command::parse` function splits the input into arguments and stores them in a `StackVec`.
- If there are no arguments, it returns an `Error::Empty`. If there are too many, it returns `Error::TooManyArgs`.

### Command Execution
- It checks the first argument to determine the command (e.g., "echo").
- If the command is recognized, it executes it (like printing the arguments for `echo`).
- If the command is unknown, it prints an error message.

### Output Handling
- It uses `kprint!` and `kprintln!` to display the shell prompt, command output, and error messages.

### Synchronization
- The shell locks the console using `CONSOLE.lock()` to prevent race conditions when reading input.

Overall, it reads input, processes commands, executes them, and prints output while handling errors efficiently.
