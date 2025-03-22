use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use core::iter::once;
use core::ptr::{null, null_mut};
use min32::winmain;
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, GENERIC_READ, GENERIC_WRITE, HINSTANCE, INVALID_HANDLE_VALUE, TRUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, ReadFile, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_ALWAYS};
use crate::do_it;

unsafe extern "C" {
    pub fn printf(fmt: *const c_char, ...) -> i32;
}

#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {{
        // Safety: It's null-terminated, and we aren't using it as a format string.
        #[allow(unused_unsafe)]
        unsafe {
            let a = alloc::format!($($args)*) + "\x00";
            crate::os::printf(c"%s\n".as_ptr(), a.as_ptr());
        }
    }};
}

pub fn read_file(input_path_utf8: &str) -> Result<Vec<u8>, String> {
    let input_path: Vec<u16> = input_path_utf8.to_string().encode_utf16().chain(once(0)).collect();

    let handle = unsafe { CreateFileW(
        input_path.as_ptr(),
        GENERIC_READ,
        FILE_SHARE_READ,
        null(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        null_mut()
    ) };

    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("Failed to open {input_path_utf8} for reading."));
    }

    let mut data: Vec<u8> = Vec::new();
    loop {
        let chunk_size_to_read = 65536;
        if data.try_reserve(chunk_size_to_read).is_err() {
            return Err(format!("Ran out of RAM."));
        }
        let old_len = data.len();
        data.resize(chunk_size_to_read + old_len, 0);

        let write_to = &mut data[old_len..];
        let mut bytes_read = 0;
        let success = unsafe { ReadFile(
            handle,
            write_to.as_mut_ptr(),
            write_to.len() as u32,
            &mut bytes_read,
            null_mut()
        ) };
        if success == FALSE {
            return Err(format!("I/O error."));
        }
        data.truncate(old_len + (bytes_read as usize));
        if bytes_read == 0 {
            break
        }
    }
    unsafe { CloseHandle(handle) };
    Ok(data)
}

pub fn write_file(output_path_utf8: &str, data: &[u8]) -> Result<(), String> {
    let output_path: Vec<u16> = output_path_utf8.to_string().encode_utf16().chain(once(0)).collect();

    let handle = unsafe { CreateFileW(
        output_path.as_ptr(),
        GENERIC_WRITE,
        FILE_SHARE_WRITE,
        null(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        null_mut()
    ) };

    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("Failed to open {output_path_utf8} for writing."));
    }

    let success = unsafe { WriteFile(handle, data.as_ptr(), data.len() as u32, &mut 0, null_mut()) };
    if success != TRUE {
        return Err(format!("Failed to write to {output_path_utf8}."));
    }
    unsafe { CloseHandle(handle) };
    Ok(())
}


#[winmain]
unsafe fn main(
    _: HINSTANCE,
    _: HINSTANCE,
    cmd: *const c_char,
    _: i32
) -> i32 {
    let cmd = unsafe { CStr::from_ptr(cmd).to_string_lossy().to_string() };
    let args: Vec<&str> = cmd.split_whitespace().collect();
    if (args.len() != 2 && args.len() != 3) || (args[0] != "compress" && args[0] != "decompress") {
        println!("Usage: chimera-compress-rs.exe <compress|decompress> <path/to/map.map> [output.map]");
        return 1;
    }

    let compressing = args[0] == "compress";
    let input_path_utf8 = args[1];
    let output_path_utf8 = args.get(2).copied().unwrap_or(args[1]);

    do_it(compressing, input_path_utf8, output_path_utf8)
}
