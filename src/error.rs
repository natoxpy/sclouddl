#[derive(Debug)]
pub enum ScloudInvalid {
    Host(String),
    Msg(String),
}

impl ScloudInvalid {
    pub fn host(domain: &str) -> Self {
        Self::Msg(String::from(domain))
    }

    pub fn msg(message: &str) -> Self {
        Self::Msg(String::from(message))
    }
}

#[derive(Debug)]
pub enum ScloudError {
    Invalid(ScloudInvalid),
    NoImplemented,
    Reqwest(reqwest::Error),
}

impl ScloudError {
    pub fn invalid_host(domain: &str) -> Self {
        Self::Invalid(ScloudInvalid::host(domain))
    }

    pub fn invalid_msg(message: &str) -> Self {
        Self::Invalid(ScloudInvalid::msg(message))
    }
}
