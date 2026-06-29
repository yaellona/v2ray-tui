mod config;
mod process;

pub use process::{start_proxy, stop_proxy};

pub(crate) const LISTEN_PORT: u16 = 10808;

pub fn get_listen_port() -> u16 {
    LISTEN_PORT
}
