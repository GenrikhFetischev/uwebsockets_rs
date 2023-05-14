use std::ffi::CString;

use libuwebsockets_sys::us_socket_context_options_t;

pub struct UsSocketContextOptions {
    pub key_file_name: Option<&'static str>,
    pub cert_file_name: Option<&'static str>,
    pub passphrase: Option<&'static str>,
    pub dh_params_file_name: Option<&'static str>,
    pub ca_file_name: Option<&'static str>,
    pub ssl_ciphers: Option<&'static str>,
    pub ssl_prefer_low_memory_usage: Option<bool>,
}

pub struct UsSocketContextOptionsCRepr {
    pub key_file_name: Option<CString>,
    pub cert_file_name: Option<CString>,
    pub passphrase: Option<CString>,
    pub dh_params_file_name: Option<CString>,
    pub ca_file_name: Option<CString>,
    pub ssl_ciphers: Option<CString>,
    pub ssl_prefer_low_memory_usage: Option<bool>,
}

impl From<UsSocketContextOptions> for UsSocketContextOptionsCRepr {
    fn from(ctx_options: UsSocketContextOptions) -> Self {
        let key_file_name = ctx_options
            .key_file_name
            .map(|val| CString::new(val).expect("key_file_name contains 0 byte"));
        let cert_file_name = ctx_options
            .cert_file_name
            .map(|val| CString::new(val).expect("cert_file_name contains 0 byte"));
        let passphrase = ctx_options
            .passphrase
            .map(|val| CString::new(val).expect("passphrase contains 0 byte"));
        let dh_params_file_name = ctx_options
            .dh_params_file_name
            .map(|val| CString::new(val).expect("dh_params_file_name contains 0 byte"));
        let ca_file_name = ctx_options
            .ca_file_name
            .map(|val| CString::new(val).expect("ca_file_name contains 0 byte"));
        let ssl_ciphers = ctx_options
            .ssl_ciphers
            .map(|val| CString::new(val).expect("ssl_ciphers contains 0 byte"));

        UsSocketContextOptionsCRepr {
            key_file_name,
            cert_file_name,
            passphrase,
            dh_params_file_name,
            ca_file_name,
            ssl_ciphers,
            ssl_prefer_low_memory_usage: ctx_options.ssl_prefer_low_memory_usage,
        }
    }
}

impl UsSocketContextOptionsCRepr {
    pub fn to_ffi(&self) -> us_socket_context_options_t {
        us_socket_context_options_t {
            key_file_name: self
                .key_file_name
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            cert_file_name: self
                .cert_file_name
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            passphrase: self
                .passphrase
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            dh_params_file_name: self
                .dh_params_file_name
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            ca_file_name: self
                .ca_file_name
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            ssl_ciphers: self
                .ssl_ciphers
                .as_ref()
                .map(|val| val.as_ptr())
                .unwrap_or(std::ptr::null()),
            ssl_prefer_low_memory_usage: self
                .ssl_prefer_low_memory_usage
                .map(|val| if val { 1 } else { 0 })
                .unwrap_or(0),
        }
    }
}
