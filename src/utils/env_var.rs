error_chain! {
    errors {
        EnvValueNotPresent(var_name: String) {
            description("Value not present")
            display("Value not present: {}", var_name)
        }
        EnvValueNotUnicode(var_name: String) {
            description("Value is not valid unicode")
            display("Value is not valid unicode: {}", var_name)
        }
    }
}

pub fn from_env(var_name: &str) -> Result<String> {
    match std::env::var(var_name) {
        Ok(var) => Ok(var),
        Err(std::env::VarError::NotPresent) => Err(Error::from(ErrorKind::EnvValueNotPresent(
            var_name.to_owned(),
        ))),
        Err(std::env::VarError::NotUnicode(_)) => Err(Error::from(ErrorKind::EnvValueNotUnicode(
            var_name.to_owned(),
        ))),
    }
}
