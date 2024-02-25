use super::{request::ScloudRequest, track::Track};
use crate::error::ScloudError;
use scraper::{Html, Selector};

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
        let type_selector = Selector::parse("meta[property='og:type']").unwrap();

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
