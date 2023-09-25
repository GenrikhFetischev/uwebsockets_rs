use std::ffi::{c_char, c_int};
use std::ptr::null_mut;
use std::{
    ffi::{c_void, CString},
    pin::Pin,
};

use libuwebsockets_sys::{
    us_listen_socket_t, uws_app_any, uws_app_connect, uws_app_delete, uws_app_get, uws_app_listen,
    uws_app_listen_config_t, uws_app_options, uws_app_patch, uws_app_post, uws_app_put,
    uws_app_run, uws_app_t, uws_app_trace, uws_create_app, uws_method_handler, uws_req_t,
    uws_res_t, uws_ws,
};

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponseStruct;
use crate::us_socket_context_options::{UsSocketContextOptions, UsSocketContextOptionsCRepr};
use crate::websocket_behavior::WebSocketBehavior;

type RoutesData<const SSL: bool> = Vec<Pin<Box<Box<dyn Fn(HttpResponseStruct<SSL>, HttpRequest)>>>>;
type ListenHandler = Option<Pin<Box<Box<dyn Fn()>>>>;

pub struct Application<const SSL: bool> {
    routes_data: RoutesData<SSL>,
    _socket_context_options: UsSocketContextOptionsCRepr,
    listen_handler: ListenHandler,
    native: *mut uws_app_t,
}

impl<const SSL: bool> Application<SSL> {
    pub fn new(socket_config: UsSocketContextOptions) -> Self {
        let socket_context_options: UsSocketContextOptionsCRepr = socket_config.into();
        let native_config = socket_context_options.to_ffi();

        unsafe {
            Self {
                routes_data: Vec::new(),
                _socket_context_options: socket_context_options,
                native: uws_create_app(SSL as i32, native_config),
                listen_handler: None,
            }
        }
    }

    pub fn ws(&mut self, pattern: &str, websocket_behavior: WebSocketBehavior<SSL>) -> &mut Self {
        let pattern_c = CString::new(pattern).expect("key_file_name contains 0 byte");
        let (behavior, user_callbacks) = websocket_behavior.into();
        let user_callbacks = Box::into_raw(Box::new(user_callbacks));
        unsafe {
            uws_ws(
                SSL as i32,
                self.native,
                pattern_c.as_ptr(),
                behavior,
                user_callbacks as *mut c_void,
            );
        }
        self
    }

    fn register_http_handler<H>(
        &mut self,
        pattern: &str,
        handler: H,
        registrar: unsafe extern "C" fn(
            c_int,
            *mut uws_app_t,
            *const c_char,
            uws_method_handler,
            *mut c_void,
        ),
    ) -> &mut Self
    where
        H: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        let pattern_c = CString::new(pattern).expect("key_file_name contains 0 byte");

        unsafe {
            self.routes_data.push(Box::pin(Box::new(handler)));
            let handler = self.routes_data.last().unwrap();

            let user_data = Pin::as_ref(handler).get_ref();
            let user_data_ptr: *const Box<dyn Fn(HttpResponseStruct<SSL>, HttpRequest)> = user_data;

            let handler = if SSL { ssl_http_handler } else { http_handler };
            registrar(
                SSL as i32,
                self.native,
                pattern_c.as_ptr(),
                Some(handler),
                user_data_ptr as *mut c_void,
            )
        }
        self
    }

    pub fn get<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_get)
    }

    pub fn post<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_post)
    }

    pub fn patch<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_patch)
    }

    pub fn delete<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_delete)
    }

    pub fn options<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_options)
    }

    pub fn put<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_put)
    }

    pub fn trace<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_trace)
    }

    pub fn connect<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_connect)
    }

    pub fn any<T>(&mut self, pattern: &str, handler: T) -> &mut Self
    where
        T: Fn(HttpResponseStruct<SSL>, HttpRequest) + 'static + Unpin,
    {
        self.register_http_handler(pattern, handler, uws_app_any)
    }

    pub fn run(&mut self) {
        unsafe { uws_app_run(SSL as i32, self.native) }
    }

    pub fn listen(&mut self, port: i32, handler: Option<impl Fn() + 'static + Unpin>) -> &mut Self {
        let user_data = if let Some(handler) = handler {
            let listen_handler: Pin<Box<Box<dyn Fn()>>> = Box::pin(Box::new(handler));
            self.listen_handler = Some(listen_handler);
            let user_data = Pin::as_ref(self.listen_handler.as_ref().unwrap()).get_ref();
            let user_data_ptr: *const Box<dyn Fn()> = user_data;
            user_data_ptr as *mut c_void
        } else {
            null_mut()
        };

        unsafe {
            uws_app_listen(SSL as i32, self.native, port, Some(on_listen), user_data);
        }
        self
    }
}

unsafe extern "C" fn http_handler(
    response: *mut uws_res_t,
    request: *mut uws_req_t,
    user_data: *mut std::os::raw::c_void,
) {
    let req = HttpRequest::new(request);
    let response = HttpResponseStruct::<false>::new(response);

    let user_handler = user_data as *mut Box<dyn Fn(HttpResponseStruct<false>, HttpRequest)>;
    let user_handler = user_handler.as_ref().unwrap();
    user_handler(response, req);
}

unsafe extern "C" fn ssl_http_handler(
    response: *mut uws_res_t,
    request: *mut uws_req_t,
    user_data: *mut std::os::raw::c_void,
) {
    let req = HttpRequest::new(request);
    let response = HttpResponseStruct::<true>::new(response);
    let user_handler = user_data as *mut Box<dyn Fn(HttpResponseStruct<true>, HttpRequest)>;
    let user_handler = user_handler.as_ref().unwrap();
    user_handler(response, req);
}

unsafe extern "C" fn on_listen(
    _: *mut us_listen_socket_t,
    _: uws_app_listen_config_t,
    user_data: *mut std::os::raw::c_void,
) {
    if !user_data.is_null() {
        let listen_handler = user_data as *mut Box<dyn Fn()>;
        let listen_handler = listen_handler.as_ref().unwrap();
        listen_handler();
    }
}

pub type SSLApp = Application<true>;
pub type App = Application<false>;
