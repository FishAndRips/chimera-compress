This tool lets you to compress/decompress maps for the Chimera mod.

There are two backends: std and win32. The std backend is the default, but you
can use the win32 backend for Windows platforms that the Rust Standard Library
does not support, such as Windows 2000 and XP.

To build for win32, run this command:

    cargo build --release --no-default-features --features win32
