/// This part of the operation primary focuses on making
/// sure the request is successful, and the it's to
/// the targetted domain.
use crate::{consts::VALID_SOUNDCLOUD_DOMAIN, error::ScloudError};
use reqwest::{
    header::{HeaderMap, HeaderValue, IntoHeaderName},
    Response, Url,
};
use scraper::Html;
use std::fmt::Display;

use super::response::ScloudResponse;

#[derive(Debug)]
pub struct ScloudRequest {
    pub url: Url,
    pub cookies: String,
    pub headers: HeaderMap,
}

impl ScloudRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            cookies: String::new(),
            headers: HeaderMap::new(),
        }
    }

    pub fn add_cookie<T: Display + ToString>(mut self, name: T, value: T) -> Self {
        self.cookies += &format!("{}={}", name, value);
        self
    }

    pub fn add_header<K: IntoHeaderName>(mut self, name: K, value: HeaderValue) -> Self {
        self.headers.append(name, value);
        self
    }

    fn validate_url(&self) -> Result<(), ScloudError> {
        let host_str = self
            .url
            .host()
            .ok_or(ScloudError::invalid_msg("Host name not found"))?;

        if host_str.to_string() == VALID_SOUNDCLOUD_DOMAIN {
            return Ok(());
        }

        Err(ScloudError::invalid_host(&host_str.to_string()))
    }

    fn get_request_headers(&self) -> Result<HeaderMap, ScloudError> {
        let mut headers = self.headers.clone();
        // TODO: Properly handle the error
        let cookie_value = HeaderValue::from_str(&self.cookies)
            .map_err(|_err| ScloudError::invalid_msg("Cookie format invalid"))?;

        headers.append("Cookie", cookie_value);

        Ok(headers)
    }

    async fn send_request(&self) -> Result<Response, ScloudError> {
        let client = reqwest::ClientBuilder::new()
            .build()
            .map_err(ScloudError::Reqwest)?;

        let headers = self.get_request_headers()?;

        client
            .get(self.url.clone())
            .headers(headers)
            .send()
            .await
            .map_err(ScloudError::Reqwest)
    }

    async fn get_text(&self, response: Response) -> Result<String, ScloudError> {
        response.text().await.map_err(ScloudError::Reqwest)
    }

    fn parse_text(&self, text: String) -> Html {
        Html::parse_document(&text)
    }

    pub async fn send(self) -> Result<ScloudResponse, ScloudError> {
        self.validate_url()?;
        let response = self.send_request().await?;
        let document = self.parse_text(self.get_text(response).await?);

        Ok(ScloudResponse::new(self, document))
    }
}
