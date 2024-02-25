use std::{fmt::Display, time::Duration};

use reqwest::IntoUrl;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ScloudError;

pub struct TrackHydration {
    pub user: Value,
    pub sound: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artwork {
    pub url: String,
    pub base: String,
    pub extension: String,
}

impl ToString for Artwork {
    fn to_string(&self) -> String {
        self.url.clone()
    }
}

impl Artwork {
    pub fn from<T: IntoUrl>(url: T) -> Result<Self, ScloudError> {
        let base = url.into_url().map_err(|_err| {
            ScloudError::invalid_msg("todo: base url error Artwork new better error message")
        })?;

        let base_host = base.host_str().ok_or(ScloudError::invalid_msg(
            "todo: base host Artwork new better error message",
        ))?;

        let base_path = base.path().to_string();

        let extension = base_path
            .split(".")
            .collect::<Vec<&str>>()
            .get(1)
            .ok_or(ScloudError::invalid_msg(
                "todo: base extension Artwork new better error message",
            ))?
            .to_string();

        if !(extension == "jpg" || extension == "png") {
            return Err(ScloudError::invalid_msg(
                "todo: Artwork not valid extension",
            ));
        }

        // Removes the resolution out of the image
        let mut base_no_resolution = base_path.split('-').collect::<Vec<&str>>();
        base_no_resolution.pop();

        if !base_no_resolution
            .get(0)
            .ok_or(ScloudError::invalid_msg(
                "todo: Artwork new better error message",
            ))?
            .ends_with("artworks")
        {
            return Err(ScloudError::invalid_msg(
                "todo: Artwork not valid url better message",
            ));
        }

        Ok(Self {
            url: base.to_string(),
            base: format!("https://{}{}", base_host, base_no_resolution.join("-")),
            extension,
        })
    }

    pub fn raw<T: ToString + Display, V: ToString + Display>(
        &self,
        resolution: T,
        extension: V,
    ) -> String {
        format!("{}-{}.{}", self.base, resolution, extension)
    }

    pub fn original(&self) -> String {
        self.raw("original", &self.extension)
    }

    /// USE `original`
    pub fn t500x500(&self) -> String {
        self.raw("t500x500", &self.extension)
    }

    /// USE `original`
    pub fn t200x200(&self) -> String {
        self.raw("t200x200", &self.extension)
    }

    /// USE `original`
    pub fn t120x120(&self) -> String {
        self.raw("t120xt120", &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Format {
    pub protocol: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transcoding {
    pub url: String,
    pub preset: String,
    pub duration: u64,
    pub snipped: bool,
    pub format: Format,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub url: String,
    pub artwork: Artwork,
    pub author: Author,
    pub duration: Duration,
    pub media: Vec<Transcoding>,
}

impl Track {
    fn get_hydration_script_selector() -> Result<Selector, ScloudError> {
        Selector::parse("script")
            .map_err(|_err| ScloudError::invalid_msg("todo: Track hydration better error message"))
    }

    fn get_hydration_script(document: &Html) -> Result<ElementRef<'_>, ScloudError> {
        let script_selector = Self::get_hydration_script_selector()?;

        let scripts = document
            .select(&script_selector)
            .into_iter()
            .collect::<Vec<ElementRef>>();

        scripts
            .get(11)
            .ok_or(ScloudError::invalid_msg(
                "todo: Track hydration better error message",
            ))
            .cloned()
    }

    fn get_hydration_json(script: ElementRef<'_>) -> Result<Value, ScloudError> {
        let script_html = script.inner_html();

        let hydration_text = &script_html[24..script_html.len() - 1];

        serde_json::from_str::<Value>(hydration_text)
            .map_err(|_err| ScloudError::invalid_msg("todo: Track hydration better error message"))
    }

    fn get_user_from(hydration_data: &Vec<Value>) -> Result<Value, ScloudError> {
        hydration_data
            .get(6)
            .ok_or(ScloudError::invalid_msg(
                "todo: Track hydration better error message",
            ))?
            .clone()
            .get("data")
            .ok_or(ScloudError::invalid_msg(
                "todo: Track hydration better error message",
            ))
            .cloned()
    }

    fn get_sound_from(hydration_data: &Vec<Value>) -> Result<Value, ScloudError> {
        hydration_data
            .get(7)
            .ok_or(ScloudError::invalid_msg(
                "todo: Track hydration better error message",
            ))?
            .clone()
            .get("data")
            .ok_or(ScloudError::invalid_msg(
                "todo: Track hydration better error message",
            ))
            .cloned()
    }

    fn hydration(document: &Html) -> Result<TrackHydration, ScloudError> {
        let hydration_json = Self::get_hydration_json(Self::get_hydration_script(document)?)?;

        if let Value::Array(data) = hydration_json {
            let user = Self::get_user_from(&data)?;
            let sound = Self::get_sound_from(&data)?;

            return Ok(TrackHydration { user, sound });
        }

        Err(ScloudError::invalid_msg(
            "todo: hydration better error message",
        ))
    }

    fn get_title(hydration: &TrackHydration) -> Result<String, ScloudError> {
        let title = hydration
            .sound
            .get("title")
            .ok_or(ScloudError::invalid_msg(
                "todo: get title better error message",
            ))?
            .as_str()
            .ok_or(ScloudError::invalid_msg(
                "todo: get artwork better error message",
            ))?
            .to_owned();

        Ok(title)
    }

    fn get_artwork(hydration: &TrackHydration) -> Result<Artwork, ScloudError> {
        let artwork_url = hydration
            .sound
            .get("artwork_url")
            .ok_or(ScloudError::invalid_msg(
                "todo: get artwork better error message",
            ))?
            .as_str()
            .ok_or(ScloudError::invalid_msg(
                "todo: get artwork better error message",
            ))?;

        Artwork::from(&artwork_url.to_string())
    }

    fn get_url(hydration: &TrackHydration) -> Result<String, ScloudError> {
        let url = hydration
            .sound
            .get("permalink_url")
            .ok_or(ScloudError::invalid_msg(
                "todo: get url better error message",
            ))?
            .as_str()
            .ok_or(ScloudError::invalid_msg(
                "todo: get artwork better error message",
            ))?
            .to_owned();

        Ok(url)
    }

    fn get_duration(hydration: &TrackHydration) -> Result<Duration, ScloudError> {
        let duration = hydration
            .sound
            .get("duration")
            .ok_or(ScloudError::invalid_msg(
                "todo: not found duration better error message",
            ))?
            .as_u64()
            .ok_or(ScloudError::invalid_msg(
                "todo: duration not a string error message",
            ))?;

        Ok(Duration::from_millis(duration))
    }

    fn get_media(hydration: &TrackHydration) -> Result<Vec<Transcoding>, ScloudError> {
        let transcodings = hydration
            .sound
            .get("media")
            .ok_or(ScloudError::invalid_msg(
                "todo: get url better error message",
            ))?
            .get("transcodings")
            .ok_or(ScloudError::invalid_msg(
                "todo: get url better error message",
            ))?
            .clone();

        serde_json::from_value::<Vec<Transcoding>>(transcodings)
            .map_err(|_err| ScloudError::invalid_msg("todo: get media from value fail"))
    }

    pub fn from_document(document: &Html) -> Result<Self, ScloudError> {
        let track_hydration = Self::hydration(document)?;

        Ok(Self {
            title: Self::get_title(&track_hydration)?,
            url: Self::get_url(&track_hydration)?,
            artwork: Self::get_artwork(&track_hydration)?,
            author: Author {},
            duration: Self::get_duration(&track_hydration)?,
            media: Self::get_media(&track_hydration)?,
        })
    }
}
