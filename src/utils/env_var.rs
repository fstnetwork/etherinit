#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Value not present: {}", _0)]
    EnvValueNotPresent(String),

    #[fail(display = "Value is not valid unicode: {}", _0)]
    EnvValueNotUnicode(String),
}

pub fn from_env(var_name: &str) -> Result<String, Error> {
    match std::env::var(var_name) {
        Ok(var) => Ok(var),
        Err(std::env::VarError::NotPresent) => Err(Error::EnvValueNotPresent(var_name.to_owned())),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err(Error::EnvValueNotUnicode(var_name.to_owned()))
        }
    }
}
