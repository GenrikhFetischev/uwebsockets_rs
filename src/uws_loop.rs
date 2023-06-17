use std::ffi::c_void;

use libuwebsockets_sys::{us_loop_t, uws_get_loop, uws_loop_defer};

#[derive(Clone, Copy, Debug)]
pub struct UwsLoop {
    pub(crate) loop_ptr: *mut us_loop_t,
}

#[cfg(feature = "native-access")]
impl UwsLoop {
    pub fn get_native(&self) -> *mut us_loop_t {
        self.loop_ptr
    }
}

unsafe impl Send for UwsLoop {}

unsafe impl Sync for UwsLoop {}

pub fn get_loop() -> UwsLoop {
    let loop_ptr = unsafe { uws_get_loop() };

    UwsLoop { loop_ptr }
}

pub fn loop_defer(uws_loop: UwsLoop, cb: impl FnOnce() + 'static) {
    let boxed_cb = Box::new(Box::new(cb));
    let cb_ptr = Box::into_raw(boxed_cb);

    unsafe {
        uws_loop_defer(
            uws_loop.loop_ptr,
            Some(loop_defer_callback),
            cb_ptr as *mut c_void,
        );
    }
}

unsafe extern "C" fn loop_defer_callback(user_data: *mut c_void) {
    println!("Inside native callback");
    let user_handler = user_data as *mut Box<dyn Fn()>;
    let user_handler = user_handler.as_ref().unwrap();
    user_handler();
    println!("executed");
}
