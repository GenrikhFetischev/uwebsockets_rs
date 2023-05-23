use libuwebsockets_sys::{us_loop_t, uws_get_loop, uws_loop_defer};
use std::ffi::c_void;

pub struct UwsLoop {
    pub(crate) loop_ptr: *mut us_loop_t,
}

unsafe impl Send for UwsLoop {}

unsafe impl Sync for UwsLoop {}

pub fn get_loop() -> UwsLoop {
    let loop_ptr = unsafe { uws_get_loop() };

    UwsLoop { loop_ptr }
}

pub fn loop_defer<C>(uws_loop: UwsLoop, cb: C)
where
    C: Fn(),
{
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
    let user_callback = Box::from_raw(user_data as *mut Box<dyn Fn()>);
    let user_callback = user_callback.as_ref().as_ref();
    user_callback();
}
