mod controller;
mod error;

pub use self::controller::Controller as EthereumController;
pub use self::error::Error;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum RestartPolicy {
    No,
    Always,
    OnFailure,
}
