use libuwebsockets_sys::{us_listen_socket_close, us_listen_socket_t};

#[derive(Clone, Copy, Debug)]
pub struct ListenSocket {
    pub(crate) listen_socket_ptr: *mut us_listen_socket_t,
}
unsafe impl Send for ListenSocket {}
unsafe impl Sync for ListenSocket {}

#[cfg(feature = "native-access")]
impl ListenSocket {
    pub fn get_native(&self) -> *mut us_listen_socket_t {
        self.listen_socket_ptr
    }
}

pub fn listen_socket_close<const SSL: bool>(listen_socket: ListenSocket) {
    unsafe {
        us_listen_socket_close(SSL.into(), listen_socket.listen_socket_ptr);
    }
}
