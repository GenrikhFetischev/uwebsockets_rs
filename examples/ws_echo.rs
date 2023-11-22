use uwebsockets_rs::app::App;
use uwebsockets_rs::http_response::HttpResponseStruct;
use uwebsockets_rs::listen_socket::ListenSocket;
use uwebsockets_rs::us_socket_context_options::UsSocketContextOptions;
use uwebsockets_rs::websocket_behavior::{CompressOptions, WebSocketBehavior};

fn main() {
    let config = UsSocketContextOptions {
        key_file_name: None,
        cert_file_name: None,
        passphrase: None,
        dh_params_file_name: None,
        ca_file_name: None,
        ssl_ciphers: None,
        ssl_prefer_low_memory_usage: None,
    };

    let compressor: u32 = CompressOptions::SharedCompressor.into();
    let decompressor: u32 = CompressOptions::SharedDecompressor.into();
    let echo_behavior = WebSocketBehavior {
        compression: compressor | decompressor,
        max_payload_length: 1024,
        idle_timeout: 111,
        max_backpressure: 10,
        close_on_backpressure_limit: false,
        reset_idle_timeout_on_send: true,
        send_pings_automatically: true,
        max_lifetime: 111,
        upgrade: Some(Box::new(HttpResponseStruct::<false>::default_upgrade)),
        open: None,
        message: Some(Box::new(|ws, message, opcode| {
            ws.send_with_options(message, opcode, false, true);
        })),
        ping: Some(Box::new(|_, message| {
            println!("Got PING, message: {message:#?}");
        })),
        pong: Some(Box::new(|_, message| {
            println!("Got PONG,  message: {message:#?}");
        })),
        close: Some(Box::new(|_, code, message| {
            println!("WS closed, code: {code}, message: {message:#?}")
        })),
        drain: Some(Box::new(|_| {
            println!("DRAIN");
        })),
        subscription: None,
    };

    App::new(config)
        .ws("/", echo_behavior)
        .listen(3001, None::<fn(ListenSocket)>)
        .run();
}
