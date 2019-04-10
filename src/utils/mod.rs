pub mod env_var;
mod retry_future;

pub use self::retry_future::RetryFuture;

pub fn clean_0x(s: &str) -> &str {
    match s.starts_with("0x") {
        true => &s[2..],
        false => s,
    }
}
