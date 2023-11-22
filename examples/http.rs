use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use uwebsockets_rs::app::App;
use uwebsockets_rs::http_request::HttpRequest;
use uwebsockets_rs::http_response::HttpResponse;
use uwebsockets_rs::listen_socket::ListenSocket;
use uwebsockets_rs::us_socket_context_options::UsSocketContextOptions;

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

    App::new(config)
        .get("/get", |res: HttpResponse, mut req| {
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

            res.end(Some("Some response".as_bytes()), true);
        })
        .post("/long", body)
        .get("/async", async_http_handler)
        .listen(3001, None::<fn(ListenSocket)>)
        .run();
}

fn body(mut res: HttpResponse, req: HttpRequest) {
    res.on_aborted(move || {
        println!("ABORTED");
    });

    res.on_writable(|a| {
        println!("Writable: {a}");
        true
    });

    let content_type = req
        .get_header("content-type")
        .unwrap_or("unknown")
        .to_string();

    let res_to_move = res.clone();
    res.on_data(move |data, end| {
        let data_len = data.len();

        if &content_type == "application/json" {
            let data_str = std::str::from_utf8(data).unwrap_or("failed to parse as utf8");
            println!("{data_str:#?}");
        } else {
            println!("{data_len}, end: {end}");
        }

        if end {
            res_to_move.end(Some("Got it".as_bytes()), true);
        }
    });
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
            res.end(Some("result".as_bytes()), true);
        } else {
            println!("Request is aborted, will not answer");
        }
    });
}
