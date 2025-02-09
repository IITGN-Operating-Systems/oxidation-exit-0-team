# Volatile Rust Crate Q&A

## Why does `Unique<Volatile>` exist? (`unique-volatile`)
Both `Volatile<T>` and `Unique<Volatile<T>>` allow read/write volatile access to an underlying pointer, but they differ in synchronization guarantees:

- `Volatile<T>` is **not Sync**, meaning it cannot be shared between threads safely.
- `Unique<Volatile<T>>` is **Sync if the inner type `T` is Sync**, allowing it to be shared safely across threads.

This distinction ensures that volatile memory accesses can be synchronized properly when used in a multithreaded environment.

---

## What’s with `#[repr(C)]`?
The `#[repr(C)]` annotation ensures that Rust lays out the structure’s fields in the same order and alignment as a C struct. This is particularly important for:

- **Interoperability with C**: If the struct is passed to C code, it needs to have a predictable memory layout.
- **Memory-mapped hardware access**: Many low-level operations (like volatile memory access) depend on precise memory layouts.
- **Preventing compiler optimizations**: Rust may reorder or pad struct fields for optimization, but `#[repr(C)]` prevents this.

---

## How are read-only and write-only accesses enforced? (`enforcing`)
The `ReadVolatile` and `WriteVolatile` wrapper types restrict how the underlying pointer can be accessed:

- `ReadVolatile<T>` only implements the `Readable<T>` trait but *not* the `Writeable<T>` trait, ensuring that values can only be **read**, not modified.
- `WriteVolatile<T>` only implements the `Writeable<T>` trait but *not* the `Readable<T>` trait, ensuring that values can only be **written**, not read.

---

## What do the macros do? (`macros`)
The macros **`readable!`**, **`writeable!`**, and **`readable_writeable!`** implement the respective traits for a given type.

- **`readable!($type, |$self| $f)`**  
  - Implements the `Readable` trait for the specified type `$type`.  
  - Defines the inner function to return a **read-only (`*const T`)** pointer.  
  - Ensures **volatile reads**.

- **`writeable!($type, |$self| $f)`**  
  - Implements the `Writeable` trait for the specified type `$type`.  
  - Defines the inner function to return a **mutable (`*mut T`)** pointer.  
  - Ensures **volatile writes**.

- **`readable_writeable!($type)`**  
  - Implements the `ReadableWriteable` trait for the specified type `$type`.  
  - The `ReadableWriteable` trait combines `Readable` and `Writeable` traits.  
  - Allows **bitwise operations** using `&` and `|`.
