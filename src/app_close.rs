use libuwebsockets_sys::uws_app_close;

use crate::app::NativeApp;

pub fn app_close<const SSL: bool>(app: NativeApp) {
    unsafe { uws_app_close(SSL as i32, app.app_ptr) }
}
