use crate::utils::{read_str_from, read_str_from_ptr};
use libuwebsockets_sys::{
    uws_req_for_each_header, uws_req_get_case_sensitive_method, uws_req_get_full_url,
    uws_req_get_header, uws_req_get_method, uws_req_get_parameter, uws_req_get_query,
    uws_req_get_url, uws_req_get_yield, uws_req_is_ancient, uws_req_set_yield, uws_req_t,
};
use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

pub struct HttpRequest {
    pub(crate) native: *mut uws_req_t,
    pub(crate) headers: Option<Vec<(&'static str, &'static str)>>,
}

impl HttpRequest {
    pub fn new(native: *mut uws_req_t) -> Self {
        HttpRequest {
            native,
            headers: None,
        }
    }
}

impl HttpRequest {
    pub fn get_full_url(&self) -> &str {
        unsafe { read_str_from(self.native, uws_req_get_full_url) }
    }

    pub fn get_url(&self) -> &str {
        unsafe { read_str_from(self.native, uws_req_get_url) }
    }

    pub fn get_method(&self) -> &str {
        unsafe { read_str_from(self.native, uws_req_get_method) }
    }

    pub fn get_case_sensitive_method(&self) -> &str {
        unsafe { read_str_from(self.native, uws_req_get_case_sensitive_method) }
    }

    pub fn get_query(&self, key: &str) -> Option<&str> {
        let key_ptr = key.as_ptr() as *const c_char;
        let key_len = key.len();
        let mut buf: *const c_char = null_mut();

        let len = unsafe {
            uws_req_get_query(
                self.native,
                key_ptr,
                key_len,
                &mut buf as *mut *const c_char,
            )
        };

        if buf.is_null() {
            return None;
        }
        Some(unsafe { read_str_from_ptr(buf as *const c_char, len) })
    }

    pub fn is_ancient(&self) -> bool {
        unsafe { uws_req_is_ancient(self.native) }
    }

    pub fn get_yield(&self) -> bool {
        unsafe { uws_req_get_yield(self.native) }
    }

    pub fn set_yield(&self, yield_opt: bool) {
        unsafe { uws_req_set_yield(self.native, yield_opt) }
    }

    pub fn get_header(&self, header_key: &str) -> Option<&str> {
        let header_key_ptr = header_key.as_ptr();
        let header_len = header_key.len();
        let mut buf: *const c_char = null_mut();
        let len = unsafe {
            uws_req_get_header(
                self.native,
                header_key_ptr as *const c_char,
                header_len,
                &mut buf as *mut *const c_char,
            )
        };

        if buf.is_null() {
            return None;
        }
        Some(unsafe { read_str_from_ptr(buf as *const c_char, len) })
    }

    pub fn get_parameter(&self, index: u16) -> Option<&str> {
        let mut buf: *const c_char = null_mut();
        let len =
            unsafe { uws_req_get_parameter(self.native, index, &mut buf as *mut *const c_char) };
        if buf.is_null() {
            return None;
        }

        Some(unsafe { read_str_from_ptr(buf as *const c_char, len) })
    }

    pub fn get_headers(&mut self) -> &Vec<(&str, &str)> {
        if self.headers.is_none() {
            let mut buf: Vec<(&str, &str)> = Vec::with_capacity(30);
            let buf_ptr: *mut Vec<(&str, &str)> = &mut buf;
            unsafe {
                uws_req_for_each_header(self.native, Some(header_iterator), buf_ptr as *mut c_void)
            }
            self.headers = Some(buf);
        }

        self.headers.as_ref().unwrap()
    }
}

unsafe extern "C" fn header_iterator(
    header_name: *const c_char,
    header_name_size: usize,
    header_value: *const c_char,
    header_value_size: usize,
    user_data: *mut c_void,
) {
    let user_data = user_data as *mut Vec<(&str, &str)>;
    let name = read_str_from_ptr(header_name, header_name_size);
    let value = read_str_from_ptr(header_value, header_value_size);

    let headers = user_data.as_mut().unwrap();
    headers.push((name, value));
}
