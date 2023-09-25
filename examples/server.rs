use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use uwebsockets_rs::app::App;
use uwebsockets_rs::http_request::HttpRequest;
use uwebsockets_rs::http_response::HttpResponse;
use uwebsockets_rs::us_socket_context_options::UsSocketContextOptions;
use uwebsockets_rs::websocket::Opcode;
use uwebsockets_rs::websocket_behavior::{CompressOptions, UpgradeContext, WebSocketBehavior};

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
    let websocket_behavior = WebSocketBehavior {
        compression: compressor | decompressor,
        max_payload_length: 1024,
        idle_timeout: 111,
        max_backpressure: 10,
        close_on_backpressure_limit: false,
        reset_idle_timeout_on_send: true,
        send_pings_automatically: true,
        max_lifetime: 111,
        upgrade: Some(Box::new(upgrade_handler)),
        open: Some(Box::new(|ws| {
            println!("WS is opened");

            let data = ws.get_user_data::<String>();
            println!("{data:#?}");
        })),
        message: Some(Box::new(|ws, message, opcode| {
            let user_data = ws.get_user_data::<()>();
            println!("User data: {user_data:#?}");
            println!("{message:#?}");
            if opcode == Opcode::Text {
                let message = std::str::from_utf8(message).unwrap();
                println!("Message: {message}");
            }

            ws.send_with_options(message, opcode, true, true);
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
        subscription: Some(Box::new(|_, topic, current_subs, prev_subs| {
            println!("SUBSCRIPTION: topic: {topic}, current_subs: {current_subs}, prev_subs: {prev_subs}");
        })),
    };

    App::new(config)
        .get("/get", |res: HttpResponse, mut req| {
            println!("Get request to /get path");
            println!("{}", req.get_full_url());
            let headers = req.get_headers();
            println!("Headers");
            for header in headers {
                println!("{header:#?}");
            }

            let header = req.get_header("host");
            println!("HOST: {header:#?}");
            let query = req.get_query("a");
            println!("query: {query:#?}");

            res.end(Some("Some response"), true);
        })
        .get("/long", long)
        .get("/async", async_http_handler)
        .ws("/ws", websocket_behavior)
        .listen(3000, None::<fn()>)
        .run();
}

fn long(mut res: HttpResponse, _: HttpRequest) {
    println!("LONG handler");

    res.on_aborted(move || {
        println!("ABORTED");
    });

    // res.on_data(|_data, _status| {
    //     println!("DATA");
    // });
    //
    // res.on_writable(|_data| {
    //     println!("WRITABLE");
    //     true
    // });
    thread::sleep(Duration::from_secs(2));
    res.end(Some("result"), true);
}

fn upgrade_handler(res: HttpResponse, req: HttpRequest, context: UpgradeContext) {
    let ws_key_string = req
        .get_header("sec-websocket-key")
        .expect("There is no sec-websocket-key in req headers");
    let ws_protocol = req.get_header("sec-websocket-protocol");
    let ws_extensions = req.get_header("sec-websocket-extensions");

    let user_data = "user_data".to_string();

    res.upgrade(
        ws_key_string,
        ws_protocol,
        ws_extensions,
        context,
        Some(user_data),
    );
}

fn async_http_handler(mut res: HttpResponse, _: HttpRequest) {
    let aborted = Arc::new(AtomicBool::new(false));
    let aborted_to_move = aborted.clone();

    res.on_aborted(move || aborted_to_move.store(true, Ordering::Relaxed));

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        let is_aborted = aborted.load(Ordering::Relaxed);
        if !is_aborted {
            println!("Answering");
            res.end(Some("result"), true);
        } else {
            println!("Request is aborted, will not answer");
        }
    });
}
