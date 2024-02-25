use super::{request::ScloudRequest, track::Track};
use crate::error::ScloudError;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub enum ScloudKind {
    Track(Track),
    Playlist,
    // TODO: Implementing the options bellow request OAuth ID from soundcloud
    Likes,
    Weekly,
    Daily,
    SecretSharePlaylist,
}

#[derive(Debug)]
pub struct ScloudContext {
    pub kind: ScloudKind,
}

#[derive(Debug)]
pub struct ScloudResponse {
    pub request: ScloudRequest,
    pub document: Html,
}

impl ScloudResponse {
    #[inline]
    pub fn new(request: ScloudRequest, document: Html) -> Self {
        Self { request, document }
    }

    fn get_meta_element_content(&self) -> Result<String, ScloudError> {
        let type_selector = Selector::parse("meta[property='og:type']").map_err(|_err| {
            ScloudError::invalid_msg("element content meta could not produce selector")
        })?;

        let type_meta_element =
            self.document
                .select(&type_selector)
                .next()
                .ok_or(ScloudError::invalid_msg(
                    "Document element meta property og:type not found",
                ))?;

        let meta_content = type_meta_element
            .attr("content")
            .ok_or(ScloudError::invalid_msg(
                "meta property og:type does not contain `content` attribute",
            ))?;

        Ok(meta_content.to_string())
    }

    fn is_playlist(&self) -> Result<bool, ScloudError> {
        let meta_content = self.get_meta_element_content()?;
        return Ok(meta_content == "music.playlist");
    }

    fn is_track(&self) -> Result<bool, ScloudError> {
        let meta_content = self.get_meta_element_content()?;
        return Ok(meta_content == "music.song");
    }

    fn handle_as_track(&self) -> Result<Track, ScloudError> {
        Track::from_document(&self.document)
    }

    async fn client_id_from_script(script_src: String) -> Result<String, ScloudError> {
        let req = reqwest::get(script_src)
            .await
            .map_err(|err| ScloudError::Reqwest(err))?
            .text()
            .await
            .map_err(|err| ScloudError::Reqwest(err))?;

        let client_id = req
            .split(",client_id:\"")
            .collect::<Vec<&str>>()
            .get(1)
            .ok_or(ScloudError::invalid_msg("get client id not found"))?
            .split("\"")
            .collect::<Vec<&str>>()
            .get(0)
            .ok_or(ScloudError::invalid_msg("get client id not found"))?
            .to_string();

        Ok(client_id)
    }

    pub async fn get_client_id(&self) -> Result<String, ScloudError> {
        let script_selector = Selector::parse("script").map_err(|_err| {
            ScloudError::invalid_msg("todo: Track hydration better error message")
        })?;

        let scripts = self
            .document
            .select(&script_selector)
            .collect::<Vec<ElementRef<'_>>>();

        let script_src = scripts
            .get(scripts.len() - 1)
            .ok_or(ScloudError::invalid_msg("get client id script not found"))?
            .attr("src")
            .ok_or(ScloudError::invalid_msg(
                "get client id script src not found",
            ))?
            .to_string();

        Self::client_id_from_script(script_src).await
    }

    pub fn context(&self) -> Result<ScloudContext, ScloudError> {
        if self.is_track()? {
            return Ok(ScloudContext {
                kind: ScloudKind::Track(self.handle_as_track()?),
            });
        };

        if self.is_playlist()? {
            // TODO: Handle playlist scenario
        }

        return Err(ScloudError::NoImplemented);
    }
}
