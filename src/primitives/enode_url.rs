use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumNodeUrl {
    pub node_id: String,
    pub host: String,
    pub port: u16,
}

impl ToString for EthereumNodeUrl {
    fn to_string(&self) -> String {
        format!("enode://{}@{}:{}", self.node_id, self.host, self.port)
    }
}

impl FromStr for EthereumNodeUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url = url::Url::parse(s)?;

        Ok(EthereumNodeUrl {
            node_id: url.username().to_owned(),
            host: url.host_str().unwrap().parse()?,
            port: url.port().unwrap_or(30303),
        })
    }
}

impl EthereumNodeUrl {
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn from_url(url: &url::Url) -> Result<EthereumNodeUrl, Error> {
        if url.scheme() != "enode" {
            return Err(Error::InvalidScheme(url.scheme().to_string()));
        }

        if url.username().is_empty() {
            return Err(Error::InvalidNodeId(url.username().to_string()));
        }

        if url.host().is_none() {
            return Err(Error::EmptyHostName);
        }

        if url.port().is_none() {
            return Err(Error::EmptyPort);
        }

        Ok(EthereumNodeUrl {
            node_id: url.username().to_owned(),
            host: url.host_str().unwrap().parse()?,
            port: url.port().unwrap_or(30303),
        })
    }
}

#[derive(Debug, Fail)]
#[fail(display = "EthereumNodeUrl Error")]
pub enum Error {
    #[fail(display = "Host name must not be empty")]
    EmptyHostName,

    #[fail(display = "Port must not be empty")]
    EmptyPort,

    #[fail(display = "Invaild scheme: {}", _0)]
    InvalidScheme(String),

    #[fail(display = "Invaild node ID: {}", _0)]
    InvalidNodeId(String),

    #[fail(display = "Invalid host name: {}", _0)]
    InvalidHostName(String),

    #[fail(display = "Invalid port: {}", _0)]
    InvalidPort(String),

    #[fail(display = "Url Parse Error: {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),

    #[fail(display = "String Parse: {}", _0)]
    StringParse(#[fail(cause)] std::string::ParseError),
}

impl From<std::string::ParseError> for Error {
    fn from(error: std::string::ParseError) -> Error {
        Error::StringParse(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Error {
        Error::UrlParse(error)
    }
}
