use std::ffi::{c_char, c_int, c_void};

use libuwebsockets_sys::{
    uws_compress_options_t, uws_compress_options_t_DEDICATED_COMPRESSOR,
    uws_compress_options_t_DEDICATED_COMPRESSOR_128KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_16KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_256KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_32KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_3KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_4KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_64KB,
    uws_compress_options_t_DEDICATED_COMPRESSOR_8KB, uws_compress_options_t_DEDICATED_DECOMPRESSOR,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_16KB,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_1KB,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_2KB,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_32KB,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_4KB,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_512B,
    uws_compress_options_t_DEDICATED_DECOMPRESSOR_8KB, uws_compress_options_t_DISABLED,
    uws_compress_options_t_SHARED_COMPRESSOR, uws_compress_options_t_SHARED_DECOMPRESSOR,
    uws_opcode_t, uws_req_t, uws_res_t, uws_socket_behavior_t, uws_socket_context_t,
    uws_websocket_t,
};

use crate::utils::{read_buf_from_ptr, read_str_from_ptr};
use crate::websocket::{Opcode, WebSocketStruct};
use crate::{http_request::HttpRequest, http_response::HttpResponseStruct};

pub enum CompressOptions {
    /* Disabled, shared, shared are "special" values */
    Disabled,
    SharedCompressor,
    SharedDecompressor,
    /* Highest 4 Bits Describe Decompressor */
    DedicatedDecompressor32kb,
    DedicatedDecompressor16kb,
    DedicatedDecompressor8kb,
    DedicatedDecompressor4kb,
    DedicatedDecompressor2kb,
    DedicatedDecompressor1kb,
    DedicatedDecompressor512b,
    /* Same As 32kb */
    DedicatedDecompressor,

    /* Lowest 8 Bit Describe Compressor */
    DedicatedCompressor3kb,
    DedicatedCompressor4kb,
    DedicatedCompressor8kb,
    DedicatedCompressor16kb,
    DedicatedCompressor32kb,
    DedicatedCompressor64kb,
    DedicatedCompressor128kb,
    DedicatedCompressor256kb,
    /* Same As 256kb */
    DedicatedCompressor,
}

impl From<CompressOptions> for uws_compress_options_t {
    fn from(value: CompressOptions) -> Self {
        match value {
            CompressOptions::Disabled => uws_compress_options_t_DISABLED,
            CompressOptions::SharedCompressor => uws_compress_options_t_SHARED_COMPRESSOR,
            CompressOptions::SharedDecompressor => uws_compress_options_t_SHARED_DECOMPRESSOR,

            CompressOptions::DedicatedDecompressor32kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_32KB
            }
            CompressOptions::DedicatedDecompressor16kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_16KB
            }
            CompressOptions::DedicatedDecompressor8kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_8KB
            }
            CompressOptions::DedicatedDecompressor4kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_4KB
            }
            CompressOptions::DedicatedDecompressor2kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_2KB
            }
            CompressOptions::DedicatedDecompressor1kb => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_1KB
            }
            CompressOptions::DedicatedDecompressor512b => {
                uws_compress_options_t_DEDICATED_DECOMPRESSOR_512B
            }
            CompressOptions::DedicatedDecompressor => uws_compress_options_t_DEDICATED_DECOMPRESSOR,

            CompressOptions::DedicatedCompressor3kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_3KB
            }
            CompressOptions::DedicatedCompressor4kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_4KB
            }
            CompressOptions::DedicatedCompressor8kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_8KB
            }
            CompressOptions::DedicatedCompressor16kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_16KB
            }
            CompressOptions::DedicatedCompressor32kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_32KB
            }
            CompressOptions::DedicatedCompressor64kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_64KB
            }
            CompressOptions::DedicatedCompressor128kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_128KB
            }
            CompressOptions::DedicatedCompressor256kb => {
                uws_compress_options_t_DEDICATED_COMPRESSOR_256KB
            }
            CompressOptions::DedicatedCompressor => uws_compress_options_t_DEDICATED_COMPRESSOR,
        }
    }
}

pub struct WebSocketBehavior<const SSL: bool> {
    /* Disabled compression by default - probably a bad default */
    pub compression: u32, //  = DISABLED;
    // /* Maximum message size we can receive */
    pub max_payload_length: u32,
    // /* 2 minutes timeout is good */
    pub idle_timeout: u16,
    // /* 64kb backpressure is probably good */
    pub max_backpressure: u32,
    pub close_on_backpressure_limit: bool,
    // /* This one depends on kernel timeouts and is a bad default */
    pub reset_idle_timeout_on_send: bool,
    // /* A good default, esp. for newcomers */
    pub send_pings_automatically: bool,
    // /* Maximum socket lifetime in minutes before forced closure (defaults to disabled) */
    pub max_lifetime: u16,
    pub upgrade: Option<WsUpgradeHandler<SSL>>,
    pub open: Option<WsOpenHandler<SSL>>,
    pub message: Option<WsMessageHandler<SSL>>,
    pub ping: Option<WsPingPongHandler<SSL>>,
    pub pong: Option<WsPingPongHandler<SSL>>,
    pub close: Option<WsCloseHandler<SSL>>,
    pub drain: Option<Box<dyn Fn(WebSocketStruct<SSL>)>>,
    pub subscription: Option<WsSubscriptionHandler<SSL>>,
}

pub type WsUpgradeHandler<const SSL: bool> =
    Box<dyn Fn(HttpResponseStruct<SSL>, HttpRequest, UpgradeContext)>;
pub type WsOpenHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>)>;
pub type WsMessageHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>, &[u8], Opcode)>;
pub type WsPingPongHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>, Option<&[u8]>)>;
pub type WsCloseHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>, i32, Option<&str>)>;
pub type WsSubscriptionHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>, &str, i32, i32)>;
pub type WsDrainHandler<const SSL: bool> = Box<dyn Fn(WebSocketStruct<SSL>)>;

pub struct UserCallbacks<const SSL: bool> {
    pub upgrade: Option<WsUpgradeHandler<SSL>>,
    pub open: Option<WsOpenHandler<SSL>>,
    pub message: Option<WsMessageHandler<SSL>>,
    pub ping: Option<WsPingPongHandler<SSL>>,
    pub pong: Option<WsPingPongHandler<SSL>>,
    pub close: Option<WsCloseHandler<SSL>>,
    pub drain: Option<WsDrainHandler<SSL>>,
    pub subscription: Option<WsSubscriptionHandler<SSL>>,
}

impl<const SSL: bool> From<WebSocketBehavior<SSL>> for (uws_socket_behavior_t, UserCallbacks<SSL>) {
    fn from(mut value: WebSocketBehavior<SSL>) -> Self {
        let upgrade = value.upgrade.take();
        let open = value.open.take();
        let message = value.message.take();
        let ping = value.ping.take();
        let pong = value.pong.take();
        let close = value.close.take();
        let drain = value.drain.take();
        let subscription = value.subscription.take();
        let user_callbacks = UserCallbacks {
            upgrade,
            open,
            message,
            ping,
            pong,
            close,
            drain,
            subscription,
        };

        let upgrade = user_callbacks.upgrade.as_ref().map(|_| {
            if SSL {
                upgrade_handler_ssl
            } else {
                upgrade_handler
            }
        });

        let open =
            user_callbacks
                .open
                .as_ref()
                .map(|_| if SSL { open_handler_ssl } else { open_handler });
        let message = user_callbacks.message.as_ref().map(|_| {
            if SSL {
                message_handler_ssl
            } else {
                message_handler
            }
        });
        let drain = user_callbacks.drain.as_ref().map(|_| {
            if SSL {
                drain_handler_ssl
            } else {
                drain_handler
            }
        });
        let ping =
            user_callbacks
                .ping
                .as_ref()
                .map(|_| if SSL { ping_handler_ssl } else { ping_handler });
        let pong =
            user_callbacks
                .pong
                .as_ref()
                .map(|_| if SSL { pong_handler_ssl } else { pong_handler });
        let close = user_callbacks.close.as_ref().map(|_| {
            if SSL {
                close_handler_ssl
            } else {
                close_handler
            }
        });
        let subscription = user_callbacks.subscription.as_ref().map(|_| {
            if SSL {
                subscription_handler_ssl
            } else {
                subscription_handler
            }
        });

        let native = uws_socket_behavior_t {
            compression: value.compression,
            maxPayloadLength: value.max_payload_length,
            idleTimeout: value.idle_timeout,
            maxBackpressure: value.max_backpressure,
            closeOnBackpressureLimit: value.close_on_backpressure_limit,
            resetIdleTimeoutOnSend: value.reset_idle_timeout_on_send,
            sendPingsAutomatically: value.send_pings_automatically,
            maxLifetime: value.max_lifetime,
            upgrade,
            open,
            message,
            drain,
            ping,
            pong,
            close,
            subscription,
        };
        (native, user_callbacks)
    }
}

pub struct UpgradeContext {
    pub(crate) context: *mut uws_socket_context_t,
}

#[cfg(feature = "native-access")]
impl UpgradeContext {
    pub fn get_context_ptr(&self) -> *mut uws_socket_context_t {
        self.context
    }
}

unsafe extern "C" fn upgrade_handler(
    response: *mut uws_res_t,
    request: *mut uws_req_t,
    context: *mut uws_socket_context_t,
    user_data: *mut c_void,
) {
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let request = HttpRequest::new(request);
    let response = HttpResponseStruct::<false>::new(response);
    let upgrade = user_callbacks.upgrade.as_ref();
    if let Some(upgrade) = upgrade {
        let context = UpgradeContext { context };
        upgrade(response, request, context)
    }
}

unsafe extern "C" fn upgrade_handler_ssl(
    response: *mut uws_res_t,
    request: *mut uws_req_t,
    context: *mut uws_socket_context_t,
    user_data: *mut c_void,
) {
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let request = HttpRequest::new(request);
    let response = HttpResponseStruct::<true>::new(response);
    let upgrade = user_callbacks.upgrade.as_ref();
    if let Some(upgrade) = upgrade {
        let context = UpgradeContext { context };
        upgrade(response, request, context)
    }
}

unsafe extern "C" fn open_handler(ws: *mut uws_websocket_t, user_data: *mut c_void) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.open.as_ref();
    if let Some(user_handler) = user_handler {
        user_handler(ws)
    }
}

unsafe extern "C" fn open_handler_ssl(ws: *mut uws_websocket_t, user_data: *mut c_void) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.open.as_ref();
    if let Some(user_handler) = user_handler {
        user_handler(ws)
    }
}

unsafe extern "C" fn message_handler(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    opcode: uws_opcode_t,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.message.as_ref();
    let message = read_buf_from_ptr(message, length);
    if let Some(user_handler) = user_handler {
        user_handler(ws, message, opcode.into())
    }
}

unsafe extern "C" fn message_handler_ssl(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    opcode: uws_opcode_t,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.message.as_ref();
    let message = read_buf_from_ptr(message, length);
    if let Some(user_handler) = user_handler {
        user_handler(ws, message, opcode.into())
    }
}

unsafe extern "C" fn ping_handler(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.ping.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_buf_from_ptr(message, length))
    };

    if let Some(user_handler) = user_handler {
        user_handler(ws, message)
    }
}

unsafe extern "C" fn ping_handler_ssl(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.pong.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_buf_from_ptr(message, length))
    };
    if let Some(user_handler) = user_handler {
        user_handler(ws, message)
    }
}

unsafe extern "C" fn pong_handler(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.ping.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_buf_from_ptr(message, length))
    };
    if let Some(user_handler) = user_handler {
        user_handler(ws, message)
    }
}

unsafe extern "C" fn pong_handler_ssl(
    ws: *mut uws_websocket_t,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.pong.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_buf_from_ptr(message, length))
    };
    if let Some(user_handler) = user_handler {
        user_handler(ws, message)
    }
}

unsafe extern "C" fn drain_handler(ws: *mut uws_websocket_t, user_data: *mut c_void) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.drain.as_ref();
    if let Some(user_handler) = user_handler {
        user_handler(ws)
    }
}

unsafe extern "C" fn drain_handler_ssl(ws: *mut uws_websocket_t, user_data: *mut c_void) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.drain.as_ref();
    if let Some(user_handler) = user_handler {
        user_handler(ws)
    }
}

unsafe extern "C" fn close_handler(
    ws: *mut uws_websocket_t,
    code: c_int,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.close.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_str_from_ptr(message, length))
    };
    if let Some(user_handler) = user_handler {
        user_handler(ws, code, message)
    }
}

unsafe extern "C" fn close_handler_ssl(
    ws: *mut uws_websocket_t,
    code: c_int,
    message: *const c_char,
    length: usize,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.close.as_ref();
    let message = if message.is_null() {
        None
    } else {
        Some(read_str_from_ptr(message, length))
    };
    if let Some(user_handler) = user_handler {
        user_handler(ws, code, message)
    }
}

unsafe extern "C" fn subscription_handler(
    ws: *mut uws_websocket_t,
    topic_name: *const c_char,
    topic_name_length: usize,
    new_number_of_subscriber: c_int,
    old_number_of_subscriber: c_int,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<false> = &mut *(user_data as *mut UserCallbacks<false>);
    let user_handler = user_callbacks.subscription.as_ref();
    let topic_name = read_str_from_ptr(topic_name, topic_name_length);
    if let Some(user_handler) = user_handler {
        user_handler(
            ws,
            topic_name,
            new_number_of_subscriber,
            old_number_of_subscriber,
        )
    }
}

unsafe extern "C" fn subscription_handler_ssl(
    ws: *mut uws_websocket_t,
    topic_name: *const c_char,
    topic_name_length: usize,
    new_number_of_subscriber: c_int,
    old_number_of_subscriber: c_int,
    user_data: *mut c_void,
) {
    let ws = WebSocketStruct::new(ws);
    let user_callbacks: &UserCallbacks<true> = &mut *(user_data as *mut UserCallbacks<true>);
    let user_handler = user_callbacks.subscription.as_ref();
    let topic_name = read_str_from_ptr(topic_name, topic_name_length);
    if let Some(user_handler) = user_handler {
        user_handler(
            ws,
            topic_name,
            new_number_of_subscriber,
            old_number_of_subscriber,
        )
    }
}
