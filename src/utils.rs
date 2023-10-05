use std::ffi::{c_char, c_int};
use std::ptr::null;
use std::slice::from_raw_parts;

pub(crate) unsafe fn read_str_from<'a, T>(
    native: *mut T,
    reader: unsafe extern "C" fn(*mut T, *mut *const c_char) -> usize,
) -> &'a str {
    let mut buf: *const c_char = null();
    let size = reader(native, &mut buf);
    read_str_from_ptr(buf, size)
}

pub(crate) unsafe fn read_str_from_with_ssl<'a, const SSL: bool, T>(
    native: *mut T,
    reader: unsafe extern "C" fn(ssl: c_int, *mut T, *mut *const c_char) -> usize,
) -> &'a str {
    let mut buf: *const c_char = null();
    let size = reader(SSL as c_int, native, &mut buf);
    read_str_from_ptr(buf, size)
}

pub(crate) unsafe fn read_str_from_ptr<'a>(ptr: *const c_char, length: usize) -> &'a str {
    read_valid_string_from_ptr(ptr, length)
}

pub(crate) unsafe fn read_buf_from_ptr<'a>(buf: *const c_char, len: usize) -> &'a [u8] {
    from_raw_parts(buf as *const u8, len)
}

pub unsafe fn read_valid_string_from_ptr<'a>(ptr: *const c_char, len: usize) -> &'a str {
    let bytes = from_raw_parts(ptr as *const u8, len);
    match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            let valid_len = e.valid_up_to();
            let invalid_len = len - valid_len;
            println!("{e:#?}");

            std::str::from_utf8(&bytes[..valid_len - invalid_len])
                .expect("[uwebsockets_rs] Can't read string from ptr")
        }
    }
}
