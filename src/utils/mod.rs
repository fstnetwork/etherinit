pub mod env_var;
mod retry_future;

pub use self::retry_future::RetryFuture;

pub fn clean_0x(s: &str) -> &str {
    if s.starts_with("0x") {
        &s[2..]
    } else {
        s
    }
}
