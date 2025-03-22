#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;

#[macro_use]
mod os;

#[cfg(feature = "std")]
use os::main;

use alloc::vec::Vec;

fn do_it(
    compressing: bool,
    input_path_utf8: &str,
    output_path_utf8: &str
) -> i32 {
    let data: Vec<u8> = match os::read_file(input_path_utf8) {
        Ok(n) => n,
        Err(e) => {
            println!("Error reading {input_path_utf8}: {e}");
            return 1
        }
    };

    let Some((header, payload)) = data.split_at_checked(0x800) else {
        println!("Error reading {input_path_utf8}: Too small to be a cache file.");
        return 1;
    };

    let engine_fourcc = u32::from_le_bytes(header[4..8].try_into().unwrap());
    let high = engine_fourcc >> 16;
    let low = engine_fourcc & 0xFFFF;

    let compress_flag = 0x861A;
    let is_compressed = high == compress_flag;
    let is_not_compressed = high == 0;
    if !is_compressed && !is_not_compressed {
        println!("Error reading {input_path_utf8}: Invalid engine FourCC 0x{engine_fourcc:08X}.");
        return 1;
    }

    if low != 0x261 && low != 7 {
        println!("Error reading {input_path_utf8}: Unsupported engine FourCC 0x{engine_fourcc:08X}.");
        return 1;
    }

    let opposite = if is_compressed { low } else { low | (compress_flag << 16) };
    let mut output = header.to_vec();
    output[4..8].copy_from_slice(&opposite.to_le_bytes());

    if is_compressed == compressing {
        if is_compressed {
            println!("Skipping {input_path_utf8}: Already compressed.");
        }
        else {
            println!("Skipping {input_path_utf8}: Not compressed.");
        }
        return 0;
    }

    if compressing {
        let size = zstd_safe::compress_bound(payload.len());

        if output.try_reserve_exact(size).is_err() {
            println!("Cannot decompress {input_path_utf8}: Not enough RAM for {size} bytes");
            return 1;
        }

        let to_offset = output.len();
        output.resize(to_offset + size, 0);

        match zstd_safe::compress(&mut output[to_offset..], payload, 22) {
            Ok(u) => {
                output.resize(to_offset + u, 0);
            }
            Err(e) => {
                println!("Cannot decompress {input_path_utf8}: zstd error {e}");
                return 1;
            }
        }

        println!("Successfully compressed {input_path_utf8}");
    }
    else {
        let size = match zstd_safe::get_frame_content_size(payload) {
            Ok(Some(n)) => n,
            Ok(None) => {
                println!("Cannot decompress {input_path_utf8}: Can't get frame content size");
                return 1;
            },
            Err(e) => {
                println!("Cannot decompress {input_path_utf8}: Corrupted {e}");
                return 1;
            }
        };

        if size > isize::MAX as u64 {
            println!("Cannot decompress {input_path_utf8}: Too big (architectural limitations) - {size} bytes");
            return 1;
        }

        if output.try_reserve_exact(size as usize).is_err() {
            println!("Cannot decompress {input_path_utf8}: Not enough RAM for {size} bytes");
            return 1;
        }

        let to_offset = output.len();
        output.resize(to_offset + size as usize, 0);

        match zstd_safe::decompress(&mut output[to_offset..], payload) {
            Ok(u) => {
                output.resize(to_offset + u, 0);
            }
            Err(e) => {
                println!("Cannot decompress {input_path_utf8}: zstd error {e}");
                return 1;
            }
        }

        println!("Successfully decompressed {input_path_utf8}");
    }

    match os::write_file(output_path_utf8, output.as_slice()) {
        Ok(n) => n,
        Err(e) => {
            println!("Error reading {input_path_utf8}: {e}");
            return 1
        }
    };

    println!("Saved {output_path_utf8}");

    0
}

