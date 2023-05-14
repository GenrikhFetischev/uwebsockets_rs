use crate::utils::{read_buf_from_ptr, read_str_from_ptr};
use libuwebsockets_sys::{
    uws_websocket_t, uws_ws_close, uws_ws_cork, uws_ws_end, uws_ws_get_buffered_amount,
    uws_ws_get_remote_address, uws_ws_get_remote_address_as_text, uws_ws_get_user_data,
    uws_ws_is_subscribed, uws_ws_iterate_topics, uws_ws_publish, uws_ws_publish_with_options,
    uws_ws_send, uws_ws_send_first_fragment, uws_ws_send_first_fragment_with_opcode,
    uws_ws_send_fragment, uws_ws_send_last_fragment, uws_ws_send_with_options, uws_ws_subscribe,
    uws_ws_unsubscribe,
};
use std::ffi::{c_char, c_int, c_void};
use std::ptr::{null, null_mut};

pub type WebSocket = WebSocketStruct<false>;
pub type WebSocketSSL = WebSocketStruct<true>;

pub struct WebSocketStruct<const SSL: bool> {
    native: *mut uws_websocket_t,
    pub(crate) cork_handler_ptr: Option<*mut dyn Fn()>,
    pub(crate) topics: Option<Vec<&'static str>>,
}

impl<const SSL: bool> WebSocketStruct<SSL> {
    pub fn new(native: *mut uws_websocket_t) -> Self {
        WebSocketStruct {
            native,
            cork_handler_ptr: None,
            topics: None,
        }
    }
}

impl<const SSL: bool> WebSocketStruct<SSL> {
    pub fn close(&self) {
        unsafe {
            uws_ws_close(SSL as c_int, self.native);
        }
    }

    pub fn send_first_fragment_with_opcode(
        &self,
        message: &[u8],
        compress: bool,
        opcode: Opcode,
    ) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        let send_status = unsafe {
            uws_ws_send_first_fragment_with_opcode(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                opcode.into(),
                compress,
            )
        };
        send_status.into()
    }

    pub fn send_first_fragment(&self, message: &[u8], compress: bool) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        unsafe {
            uws_ws_send_first_fragment(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                compress,
            )
            .into()
        }
    }

    pub fn send_fragment(&self, message: &[u8], compress: bool) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        unsafe {
            uws_ws_send_fragment(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                compress,
            )
            .into()
        }
    }
    pub fn send_last_fragment(&self, message: &[u8], compress: bool) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        unsafe {
            uws_ws_send_last_fragment(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                compress,
            )
            .into()
        }
    }

    pub fn send(&self, message: &[u8], opcode: Opcode) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        unsafe {
            uws_ws_send(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                opcode.into(),
            )
            .into()
        }
    }

    pub fn send_with_options(
        &self,
        message: &[u8],
        opcode: Opcode,
        compress: bool,
        fin: bool,
    ) -> SendStatus {
        let message_ptr = message.as_ptr() as *const c_char;
        let message_len = message.len();
        unsafe {
            uws_ws_send_with_options(
                SSL as c_int,
                self.native,
                message_ptr,
                message_len,
                opcode.into(),
                compress,
                fin,
            )
            .into()
        }
    }

    pub fn end(&self, code: i32, message: Option<&str>) {
        let (message_ptr, message_len) = if let Some(message) = message {
            let message_ptr = message.as_ptr() as *const c_char;
            let message_len = message.len();
            (message_ptr, message_len)
        } else {
            (null(), 0)
        };

        unsafe { uws_ws_end(SSL as c_int, self.native, code, message_ptr, message_len) }
    }

    pub fn cork(&mut self, handler: impl Fn() + 'static) {
        let handler: Box<dyn Fn()> = Box::new(handler);
        let user_data = Box::into_raw(handler);
        self.cork_handler_ptr = Some(user_data);
        unsafe {
            uws_ws_cork(
                SSL as c_int,
                self.native,
                Some(on_cork),
                user_data as *mut c_void,
            )
        }
    }

    pub fn subscribe(&self, topic: &str) -> bool {
        let topic_ptr = topic.as_ptr() as *const c_char;
        let topic_len = topic.len();
        unsafe { uws_ws_subscribe(SSL as c_int, self.native, topic_ptr, topic_len) }
    }

    pub fn unsubscribe(&self, topic: &str) -> bool {
        let topic_ptr = topic.as_ptr() as *const c_char;
        let topic_len = topic.len();
        unsafe { uws_ws_unsubscribe(SSL as c_int, self.native, topic_ptr, topic_len) }
    }

    pub fn is_subscribed(&self, topic: &str) -> bool {
        let topic_ptr = topic.as_ptr() as *const c_char;
        let topic_len = topic.len();
        unsafe { uws_ws_is_subscribed(SSL as c_int, self.native, topic_ptr, topic_len) }
    }

    pub fn iterate_topics(&self) -> &Vec<&'static str> {
        if self.topics.is_none() {
            let mut buf = Vec::new();
            let buf_ptr: *mut Vec<&str> = &mut buf;
            unsafe {
                uws_ws_iterate_topics(
                    SSL as c_int,
                    self.native,
                    Some(topic_iterator),
                    buf_ptr as *mut c_void,
                );
            }
        }

        self.topics.as_ref().unwrap()
    }

    pub fn publish(&self, topic: &str, message: &[u8]) -> bool {
        unsafe {
            let topic_ptr = topic.as_ptr() as *const c_char;
            let topic_len = topic.len();
            let message_ptr = message.as_ptr() as *const c_char;
            let message_len = message.len();
            uws_ws_publish(
                SSL as c_int,
                self.native,
                topic_ptr,
                topic_len,
                message_ptr,
                message_len,
            )
        }
    }

    pub fn publish_with_options(
        &self,
        topic: &str,
        message: &[u8],
        opcode: Opcode,
        compress: bool,
    ) -> bool {
        unsafe {
            let topic_ptr = topic.as_ptr() as *const c_char;
            let topic_len = topic.len();
            let message_ptr = message.as_ptr() as *const c_char;
            let message_len = message.len();
            uws_ws_publish_with_options(
                SSL as c_int,
                self.native,
                topic_ptr,
                topic_len,
                message_ptr,
                message_len,
                opcode.into(),
                compress,
            )
        }
    }

    pub fn get_buffered_amount(&self) -> u32 {
        unsafe { uws_ws_get_buffered_amount(SSL as c_int, self.native) }
    }

    pub fn get_remote_address(&self) -> &[u8] {
        let mut buf: *const c_char = null_mut();
        let len = unsafe {
            uws_ws_get_remote_address(SSL as c_int, self.native, &mut buf as *mut *const c_char)
        };

        unsafe { read_buf_from_ptr(buf, len) }
    }

    pub fn get_remote_address_as_text(&self) -> &str {
        let mut buf: *const c_char = null_mut();
        let len = unsafe {
            uws_ws_get_remote_address_as_text(
                SSL as c_int,
                self.native,
                &mut buf as *mut *const c_char,
            )
        };

        unsafe { read_str_from_ptr(buf, len) }
    }

    pub fn get_user_data<T: Sized>(&self) -> Option<&mut T> {
        let user_data_ptr = unsafe { uws_ws_get_user_data(SSL as c_int, self.native) };
        let user_data_ptr = user_data_ptr as *mut T;
        unsafe { user_data_ptr.as_mut() }
    }
}

impl<const SSL: bool> Drop for WebSocketStruct<SSL> {
    fn drop(&mut self) {
        unsafe { self.cork_handler_ptr.map(|ptr| Box::from_raw(ptr)) };
    }
}

#[derive(Debug)]
pub enum SendStatus {
    Backpressure,
    Success,
    Dropped,
}

impl From<u32> for SendStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => SendStatus::Backpressure,
            1 => SendStatus::Success,
            2 => SendStatus::Dropped,
            _ => panic!("Unknown send status"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Opcode {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

impl From<Opcode> for u32 {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::Continuation => 0,
            Opcode::Text => 1,
            Opcode::Binary => 2,
            Opcode::Close => 8,
            Opcode::Ping => 9,
            Opcode::Pong => 10,
        }
    }
}

impl From<u32> for Opcode {
    fn from(value: u32) -> Self {
        match value {
            0 => Opcode::Continuation,
            1 => Opcode::Text,
            2 => Opcode::Binary,
            8 => Opcode::Close,
            9 => Opcode::Ping,
            10 => Opcode::Pong,
            _ => panic!("Unknown opcode"),
        }
    }
}

unsafe extern "C" fn on_cork(user_data: *mut c_void) {
    let user_handler: Box<dyn Fn()> = Box::from_raw(user_data as *mut Box<dyn Fn()>);
    let user_handler = user_handler.as_ref();

    user_handler();
}

unsafe extern "C" fn topic_iterator(topic: *const c_char, length: usize, user_data: *mut c_void) {
    let topic = read_str_from_ptr(topic, length);
    let buf = user_data as *mut Vec<&str>;
    let buf = buf.as_mut().unwrap();
    buf.push(topic);
}
