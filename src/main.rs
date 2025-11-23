#![no_std]
#![no_main]
#![windows_subsystem = "console"]

extern crate core;

use core::ffi::c_void;
use core::ptr;

use busybox_like::{message_for, parse_command};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCommandLineW() -> *mut u16;
    fn WideCharToMultiByte(
        CodePage: u32,
        dwFlags: u32,
        lpWideCharStr: *const u16,
        cchWideChar: i32,
        lpMultiByteStr: *mut u8,
        cbMultiByte: i32,
        lpDefaultChar: *const u8,
        lpUsedDefaultChar: *mut i32,
    ) -> i32;
    fn GetStdHandle(nStdHandle: i32) -> *mut c_void;
    fn WriteFile(
        hFile: *mut c_void,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> i32;
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;
    fn ExitProcess(uExitCode: u32) -> ();
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn CommandLineToArgvW(lpCmdLine: *const u16, pNumArgs: *mut i32) -> *mut *mut u16;
}

const CP_UTF8: u32 = 65001;
const STD_OUTPUT_HANDLE: i32 = -11i32;
const STD_ERROR_HANDLE: i32 = -12i32;

fn write_handle(handle: *mut c_void, bytes: &[u8]) {
    unsafe {
        let mut written: u32 = 0;
        let _ = WriteFile(handle, bytes.as_ptr(), bytes.len() as u32, &mut written as *mut u32, ptr::null_mut());
    }
}

fn write_stdout(s: &str) {
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        write_handle(h, s.as_bytes());
        write_handle(h, b"\n");
    }
}

fn write_stderr(s: &str) {
    unsafe {
        let h = GetStdHandle(STD_ERROR_HANDLE);
        write_handle(h, s.as_bytes());
        write_handle(h, b"\n");
    }
}

fn wide_to_utf8_trunc(dst: &mut [u8], wptr: *const u16) -> usize {
    unsafe {
        if wptr.is_null() {
            return 0;
        }
        // ask for required size (including null)
        let needed = WideCharToMultiByte(CP_UTF8, 0, wptr, -1, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());
        if needed <= 0 {
            return 0;
        }
        let buf_len = dst.len() as i32;
        let to_write = if needed > buf_len { buf_len } else { needed };
        let written = WideCharToMultiByte(CP_UTF8, 0, wptr, -1, dst.as_mut_ptr(), to_write, ptr::null(), ptr::null_mut());
        if written <= 0 {
            return 0;
        }
        // written includes null terminator when space allowed
        let mut len = (written as usize).saturating_sub(1);
        if len > dst.len() { len = dst.len(); }
        len
    }
}

fn file_stem_from_path_bytes(s: &str) -> &str {
    // reuse prior logic: trim trailing separators, find last sep, then dot
    let s = s.trim_end_matches(|c| c == '/' || c == '\\');
    let last_sep = s.rfind(|c| c == '/' || c == '\\').map(|i| i + 1).unwrap_or(0);
    let fname = &s[last_sep..];
    match fname.rfind('.') {
        Some(dot) if dot > 0 => &fname[..dot],
        _ => fname,
    }
}

// No custom panic handler here; allow the toolchain/std to provide one or
// rely on `panic = "abort"` in Cargo.toml.

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        ExitProcess(1);
    }
    loop {}
}

#[unsafe(export_name = "mainCRTStartup")]
pub extern "C" fn start() -> ! {
    // small stack buffers for conversion
    let mut utf8buf1 = [0u8; 2048];
    let mut utf8buf2 = [0u8; 2048];

    unsafe {
        let cmd = GetCommandLineW();
        let mut argc: i32 = 0;
        let argv = CommandLineToArgvW(cmd as *const u16, &mut argc as *mut i32);
        if !argv.is_null() && argc > 0 {
            // first arg
            let first_w = *argv.offset(0);
            let len1 = wide_to_utf8_trunc(&mut utf8buf1, first_w as *const u16);
            let first = core::str::from_utf8_unchecked(&utf8buf1[..len1]);
            let first_stem = file_stem_from_path_bytes(first);

            match parse_command(first_stem) {
                Some(cmd) => {
                    write_stdout(message_for(&cmd));
                    ExitProcess(0);
                }
                None => {
                    // continue and process next
                    // write_stderr(first_stem);
                    if argc > 1 {
                        let second_w = *argv.offset(1);
                        let len2 = wide_to_utf8_trunc(&mut utf8buf2, second_w as *const u16);
                        let second = core::str::from_utf8_unchecked(&utf8buf2[..len2]);
                        if let Some(cmd2) = parse_command(second) {
                            write_stdout(message_for(&cmd2));
                        } else {
                            write_stderr("missing command");
                        }
                    }
                    ExitProcess(0);
                }
            }

            LocalFree(argv as *mut c_void);
        } else {
            write_stderr("no first arg?!!?");
            ExitProcess(1);
        }
    }

    loop {}
}

// Provide a small memcmp shim that some runtime code may expect when
// building core/alloc without the full libstd. This prevents a linker
// undefined reference to `memcmp`.
#[unsafe(export_name = "memcmp")]
pub extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0usize;
        while i < n {
            let va = *a.add(i);
            let vb = *b.add(i);
            if va != vb {
                return (va as i32) - (vb as i32);
            }
            i += 1;
        }
        0
    }
}
