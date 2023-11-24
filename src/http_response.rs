use std::ffi::{c_char, c_int, c_void};
use std::ptr::{null, null_mut};

use libuwebsockets_sys::{
  uws_res_cork, uws_res_end, uws_res_end_without_body, uws_res_get_remote_address,
  uws_res_get_remote_address_as_text, uws_res_get_write_offset, uws_res_has_responded,
  uws_res_on_aborted, uws_res_on_data, uws_res_on_writable, uws_res_override_write_offset,
  uws_res_pause, uws_res_resume, uws_res_t, uws_res_try_end, uws_res_upgrade, uws_res_write,
  uws_res_write_continue, uws_res_write_header, uws_res_write_header_int, uws_res_write_status,
  uws_try_end_result_t,
};

use crate::http_request::HttpRequest;
use crate::utils::{read_buf_from_ptr, read_str_from_with_ssl};
use crate::websocket_behavior::UpgradeContext;

pub(crate) type OnDataHandler = Box<dyn Fn(&[u8], bool)>;
pub(crate) type OnWritableHandler = Box<dyn Fn(u64) -> bool>;

pub type HttpResponse = HttpResponseStruct<false>;
pub type HttpResponseSSL = HttpResponseStruct<true>;

#[derive(Clone)]
pub struct HttpResponseStruct<const SSL: bool> {
    pub(crate) on_abort_ptr: Option<*mut Box<dyn Fn()>>,
    pub(crate) on_data_ptr: Option<*mut OnDataHandler>,
    pub(crate) on_writable_ptr: Option<*mut OnWritableHandler>,
    pub(crate) on_cork_ptr: Option<*mut dyn FnOnce()>,
    pub(crate) native: *mut uws_res_t,
}

unsafe impl<const SSL: bool> Sync for HttpResponseStruct<SSL> {}
unsafe impl<const SSL: bool> Send for HttpResponseStruct<SSL> {}

impl<const SSL: bool> HttpResponseStruct<SSL> {
    pub fn new(native: *mut uws_res_t) -> Self {
        HttpResponseStruct::<SSL> {
            native,
            on_abort_ptr: None,
            on_data_ptr: None,
            on_writable_ptr: None,
            on_cork_ptr: None,
        }
    }
}

#[cfg(feature = "native-access")]
impl<const SSL: bool> HttpResponseStruct<SSL> {
    pub fn get_native(&self) -> *mut uws_res_t {
        self.native
    }
}

impl<const SSL: bool> HttpResponseStruct<SSL> {
    pub fn default_upgrade(res: HttpResponse, req: HttpRequest, context: UpgradeContext) {
        let ws_key_string = req
            .get_header("sec-websocket-key")
            .expect("There is no sec-websocket-key in req headers");
        let ws_protocol = req.get_header("sec-websocket-protocol");
        let ws_extensions = req.get_header("sec-websocket-extensions");

        res.upgrade(
            ws_key_string,
            ws_protocol,
            ws_extensions,
            context,
            None::<&mut ()>,
        );
    }
}

impl<const SSL: bool> HttpResponseStruct<SSL> {
    pub fn cork(&mut self, handler: impl Fn() + Sized + 'static) {
        let user_data: Box<dyn FnOnce()> = Box::new(Box::new(handler));
        let user_data = Box::into_raw(user_data);
        self.on_cork_ptr = Some(user_data);

        unsafe {
            uws_res_cork(
                SSL as c_int,
                self.native,
                Some(on_cork),
                user_data as *mut c_void,
            )
        }
    }

    pub fn on_data(&mut self, handler: impl Fn(&[u8], bool) + Sized + 'static) {
        let user_data: Box<OnDataHandler> = Box::new(Box::new(handler));
        let user_data = Box::into_raw(user_data);

        self.on_data_ptr = Some(user_data);
        unsafe {
            uws_res_on_data(
                SSL as c_int,
                self.native,
                Some(on_data),
                user_data as *mut c_void,
            );
        }
    }

    pub fn on_writable(&mut self, handler: impl Fn(u64) -> bool + Sized + 'static) {
        let user_data: Box<OnWritableHandler> = Box::new(Box::new(handler));
        let user_data = Box::into_raw(user_data);

        self.on_writable_ptr = Some(user_data);
        unsafe {
            uws_res_on_writable(
                SSL as c_int,
                self.native,
                Some(on_writable),
                user_data as *mut c_void,
            );
        }
    }

    pub fn on_aborted(&mut self, handler: impl Fn() + Sized + 'static) -> &Self {
        let user_callback: Box<Box<dyn Fn()>> = Box::new(Box::new(handler));
        let user_callback_ptr = Box::into_raw(user_callback);
        self.on_abort_ptr = Some(user_callback_ptr);
        let http_response = Box::into_raw(Box::new(self.clone()));

        let native_callback = if SSL { ssl_on_abort } else { on_abort };

        unsafe {
            uws_res_on_aborted(
                SSL as i32,
                self.native,
                Some(native_callback),
                http_response as *mut c_void,
            )
        }

        self
    }

    pub fn deinit(&self) {
        unsafe {
            let _ = self.on_writable_ptr.map(|p| Box::from_raw(p));
            let _ = self.on_cork_ptr.map(|p| Box::from_raw(p));
        }
    }

    pub fn end(&self, data: Option<&[u8]>, close_connection: bool) {
        unsafe {
            let (data, length) = match data {
                Some(data) => (data.as_ptr(), data.len()),
                None => (null(), 0),
            };
            uws_res_end(
                SSL as i32,
                self.native,
                data as *const c_char,
                length,
                close_connection,
            )
        }
        self.deinit();
        unsafe {
            let _ = self.on_abort_ptr.map(|p| Box::from_raw(p));
        }
    }

    pub fn try_end(
        &self,
        data: Option<&[u8]>,
        total_size: u64,
        close_connection: bool,
    ) -> TryEndResult<SSL> {
        let res: TryEndResult<SSL> = unsafe {
            let (data, length) = match data {
                Some(data) => (data.as_ptr(), data.len()),
                None => (null(), 0),
            };
            uws_res_try_end(
                SSL as i32,
                self.native,
                data as *const c_char,
                length,
                total_size,
                close_connection,
            )
        }
        .into();

        if res.has_responded {
            self.deinit();
            unsafe {
                let _ = self.on_abort_ptr.map(|p| Box::from_raw(p));
            }
        }

        res
    }

    pub fn pause(&self) {
        unsafe { uws_res_pause(SSL as c_int, self.native) }
    }

    pub fn resume(&self) {
        unsafe { uws_res_resume(SSL as c_int, self.native) }
    }

    pub fn write(&self, data: &[u8]) -> bool {
        let data_len = data.len();
        let data_ptr = data.as_ptr() as *const c_char;
        unsafe { uws_res_write(SSL as c_int, self.native, data_ptr, data_len) }
    }

    pub fn write_continue(&self) {
        unsafe { uws_res_write_continue(SSL as c_int, self.native) }
    }

    pub fn write_status(&self, status: &str) {
        let len = status.len();
        let status_ptr = status.as_ptr() as *const c_char;
        unsafe {
            uws_res_write_status(SSL as c_int, self.native, status_ptr, len);
        }
    }

    pub fn write_header(&self, key: &str, value: &str) {
        let key_len = key.len();
        let key_ptr = key.as_ptr() as *const c_char;
        let value_len = value.len();
        let value_ptr = value.as_ptr() as *const c_char;
        unsafe {
            uws_res_write_header(
                SSL as c_int,
                self.native,
                key_ptr,
                key_len,
                value_ptr,
                value_len,
            );
        }
    }

    pub fn write_header_int(&self, key: &str, value: u64) {
        let key_len = key.len();
        let key_ptr = key.as_ptr() as *const c_char;
        unsafe {
            uws_res_write_header_int(SSL as c_int, self.native, key_ptr, key_len, value);
        }
    }

    pub fn end_without_body(&self, close_connection: bool) {
        unsafe {
            uws_res_end_without_body(SSL as c_int, self.native, close_connection);
        }
    }

    pub fn get_write_offset(&self) -> u64 {
        unsafe { uws_res_get_write_offset(SSL as c_int, self.native) }
    }

    pub fn override_write_offset(&self, offset: u64) {
        unsafe {
            uws_res_override_write_offset(SSL as c_int, self.native, offset);
        }
    }

    pub fn has_responded(&self) -> bool {
        unsafe { uws_res_has_responded(SSL as c_int, self.native) }
    }

    pub fn get_remote_address(&self) -> &str {
        unsafe { read_str_from_with_ssl::<SSL, uws_res_t>(self.native, uws_res_get_remote_address) }
    }

    pub fn get_remote_address_as_text(&self) -> &str {
        unsafe {
            read_str_from_with_ssl::<SSL, uws_res_t>(
                self.native,
                uws_res_get_remote_address_as_text,
            )
        }
    }

    pub fn upgrade<T>(
        &self,
        ws_key: &str,
        ws_protocol: Option<&str>,
        ws_extensions: Option<&str>,
        context: UpgradeContext,
        user_data: Option<&mut T>,
    ) where
        T: Sized,
    {
        let user_data = user_data
            .map(|data| data as *mut _ as *mut c_void)
            .unwrap_or(null_mut());

        let protocol_ptr = ws_protocol.map(|ext| ext.as_ptr()).unwrap_or(null());
        let protocol_len = ws_protocol.map(|ext| ext.len()).unwrap_or(0);

        let extensions_ptr = ws_extensions.map(|ext| ext.as_ptr()).unwrap_or(null());
        let extensions_len = ws_extensions.map(|ext| ext.len()).unwrap_or(0);

        unsafe {
            uws_res_upgrade(
                SSL as c_int,
                self.native,
                user_data,
                ws_key.as_ptr() as *const c_char,
                ws_key.len(),
                protocol_ptr as *const c_char,
                protocol_len,
                extensions_ptr as *const c_char,
                extensions_len,
                context.context,
            )
        }
    }
}

unsafe extern "C" fn on_abort(_res: *mut uws_res_t, user_data: *mut c_void) {
    let http_response = Box::from_raw(user_data as *mut HttpResponseStruct<false>);

    let user_handler = http_response.on_abort_ptr.unwrap();
    let user_handler = user_handler.as_ref().unwrap();

    user_handler();
    http_response.deinit()
}

unsafe extern "C" fn ssl_on_abort(_res: *mut uws_res_t, user_data: *mut c_void) {
    let http_response = Box::from_raw(user_data as *mut HttpResponseStruct<true>);

    let user_handler = Box::from_raw(http_response.on_abort_ptr.unwrap());
    let user_handler = user_handler.as_ref();

    user_handler();
    http_response.deinit()
}

unsafe extern "C" fn on_data(
    _: *mut uws_res_t,
    chunk: *const c_char,
    chunk_length: usize,
    is_end: bool,
    optional_data: *mut c_void,
) {
    if is_end {
        let user_handler = Box::from_raw(optional_data as *mut OnDataHandler);
        let user_handler = user_handler.as_ref();
        let buf = read_buf_from_ptr(chunk, chunk_length);
        user_handler(buf, is_end);
    } else {
        let user_handler = optional_data as *mut OnDataHandler;
        let user_handler = user_handler.as_ref().unwrap();
        let buf = read_buf_from_ptr(chunk, chunk_length);
        user_handler(buf, is_end);
    };
}

unsafe extern "C" fn on_writable(_: *mut uws_res_t, arg1: u64, optional_data: *mut c_void) -> bool {
    let user_handler = optional_data as *mut Box<dyn Fn(u64) -> bool>;
    let user_handler = user_handler.as_ref().unwrap();
    user_handler(arg1)
}

unsafe extern "C" fn on_cork(_: *mut uws_res_t, user_data: *mut c_void) {
    let user_handler = user_data as *mut Box<dyn Fn()>;
    let user_handler = user_handler.as_ref().unwrap();
    user_handler()
}

pub struct TryEndResult<const SSL: bool> {
    pub ok: bool,
    pub has_responded: bool,
    // TODO: consider take ownership of self in try end & end, then use that field
    // pub(crate) res: Option<HttpResponseStruct<SSL>>,
}

impl<const SSL: bool> From<uws_try_end_result_t> for TryEndResult<SSL> {
    fn from(result: uws_try_end_result_t) -> Self {
        TryEndResult {
            ok: result.ok,
            has_responded: result.has_responded,
        }
    }
}
