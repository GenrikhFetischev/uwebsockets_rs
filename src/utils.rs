use std::ffi::{c_char, c_int};
use std::ptr::null;
use std::slice::from_raw_parts;

pub(crate) unsafe fn read_str_from<T>(
    native: *mut T,
    reader: unsafe extern "C" fn(*mut T, *mut *const c_char) -> usize,
) -> &'static str {
    let mut buf: *const c_char = null();
    let size = reader(native, &mut buf);
    read_str_from_ptr(buf, size)
}

pub(crate) unsafe fn read_str_from_with_ssl<const SSL: bool, T>(
    native: *mut T,
    reader: unsafe extern "C" fn(ssl: c_int, *mut T, *mut *const c_char) -> usize,
) -> &'static str {
    let mut buf: *const c_char = null();
    let size = reader(SSL as c_int, native, &mut buf);
    read_str_from_ptr(buf, size)
}

pub(crate) unsafe fn read_str_from_ptr(ptr: *const c_char, length: usize) -> &'static str {
    let str_slice = from_raw_parts(ptr as *const u8, length);
    // TODO: consider handling unwrap
    std::str::from_utf8(str_slice).unwrap()
}

pub(crate) unsafe fn read_buf_from_ptr(buf: *const c_char, len: usize) -> &'static [u8] {
    from_raw_parts(buf as *const u8, len)
}
