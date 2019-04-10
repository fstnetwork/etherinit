use std::str::FromStr;

error_chain! {
    foreign_links {
        UrlParse(url::ParseError);
        StringParse(std::string::ParseError);
    }

    errors {
        InvalidScheme(s: String) {
            description("Invalid scheme")
            display("Invalid scheme: {}", s)
        }
        InvalidNodeId(s: String) {
            description("Invalid node id")
            display("Invalid node id: {}", s)
        }
        InvalidHostName(s: String) {
            description("Invalid host name")
            display("Invalid host name: {}", s)
        }
        InvalidPort(s: String) {
            description("Invalid scheme")
            display("Invalid scheme: {}", s)
        }
        EmptyHostName
        EmptyPort
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    fn from_str(s: &str) -> Result<Self> {
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

    pub fn from_url(url: &url::Url) -> Result<EthereumNodeUrl> {
        if url.scheme() != "enode" {
            return Err(Error::from(ErrorKind::InvalidScheme(
                url.scheme().to_string(),
            )));
        }

        if url.username().is_empty() {
            return Err(Error::from(ErrorKind::InvalidNodeId(
                url.username().to_string(),
            )));
        }

        if url.host().is_none() {
            return Err(Error::from(ErrorKind::EmptyHostName));
        }

        if url.port().is_none() {
            return Err(Error::from(ErrorKind::EmptyPort));
        }

        Ok(EthereumNodeUrl {
            node_id: url.username().to_owned(),
            host: url.host_str().unwrap().parse()?,
            port: url.port().unwrap_or(30303),
        })
    }
}
