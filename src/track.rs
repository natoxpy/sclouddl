use crate::utils::gen_key;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub progressive: String,
    pub hls: String,
}

impl Media {
    pub async fn get_progressive(&self, client_id: &str) -> String {
        let prog = reqwest::get(format!("{}?client_id={}", self.progressive, client_id))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        prog.split("\"")
            .collect::<Vec<&str>>()
            .get(3)
            .unwrap()
            .to_string()
    }

    pub async fn get_urls(&self, client_id: &str) -> Media {
        let prog = reqwest::get(format!("{}?client_id={}", self.progressive, client_id))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let hls = reqwest::get(format!("{}?client_id={}", self.hls, client_id))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        Media {
            progressive: prog
                .split("\"")
                .collect::<Vec<&str>>()
                .get(3)
                .unwrap()
                .to_string(),
            hls: hls
                .split("\"")
                .collect::<Vec<&str>>()
                .get(3)
                .unwrap()
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub username: String,
    pub avatar_url: String,
}

impl Author {
    pub fn get_avatar(content: String) -> String {
        let regex =
            Regex::new("\"hydratable\":\"user\",\"data\":\\{\"avatar_url[\\w\":\\/\\.\\-\\_]+")
                .unwrap();

        let mut locs = regex.capture_locations();

        regex.captures_read(&mut locs, &content);

        let thl = locs.get(0).unwrap();

        content[thl.0..thl.1]
            .split("\"")
            .collect::<Vec<&str>>()
            .get(9)
            .unwrap()
            .to_string()
    }

    pub fn get_username(content: String) -> String {
        let regex = Regex::new("username\":\"[\\w\\s\\-\\_]+").unwrap();

        let mut locs = regex.capture_locations();

        regex.captures_read(&mut locs, &content);
        let thl = locs.get(0).unwrap();

        content[thl.0..thl.1]
            .split("\"")
            .collect::<Vec<&str>>()
            .get(2)
            .unwrap()
            .to_string()
    }

    pub fn get(content: String) -> Self {
        Self {
            username: Author::get_username(content.clone()),
            avatar_url: Author::get_avatar(content),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: Duration,
    pub media: Media,
    pub client_id: String,
    pub author: Author,
}

impl Track {
    async fn get(url: &str) -> String {
        reqwest::get(url).await.unwrap().text().await.unwrap()
    }

    fn get_title(content: String) -> String {
        let regex = Regex::new("(property=\"og:title\" content=\")[\\w\\s\\,\\/:\\.-]+").unwrap();
        let mut locs = regex.capture_locations();
        regex.captures_read(&mut locs, &content);

        let thl = locs.get(0).unwrap();

        content[thl.0..thl.1]
            .split("\"")
            .collect::<Vec<&str>>()
            .get(3)
            .unwrap()
            .to_string()
    }

    fn get_thumbnail(content: String) -> String {
        let regex = Regex::new("(property=\"og:image\" content=\")[\\w\\s\\/:\\.-]+").unwrap();
        let mut locs = regex.capture_locations();
        regex.captures_read(&mut locs, &content);

        let thl = locs.get(0).unwrap();
        content[thl.0..thl.1]
            .split("\"")
            .collect::<Vec<&str>>()
            .get(3)
            .unwrap()
            .to_string()
    }

    fn get_duration(content: String) -> Duration {
        let regex = Regex::new("full_duration\":\\w+").unwrap();
        let mut locs = regex.capture_locations();
        regex.captures_read(&mut locs, &content);

        let thl = locs.get(0).unwrap();
        let duration_str = content[thl.0..thl.1]
            .split(":")
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap()
            .to_string();

        Duration::from_millis(duration_str.parse().unwrap())
    }

    fn get_permanent_url(content: String) -> String {
        let regex = Regex::new("(<link rel=\"canonical\" href=\")[\\w/:\\.-]+").unwrap();
        let mut locs = regex.capture_locations();
        regex.captures_read(&mut locs, &content);

        let thl = locs.get(0).unwrap();
        content[thl.0..thl.1]
            .split("\"")
            .collect::<Vec<&str>>()
            .get(3)
            .unwrap()
            .to_string()
    }

    fn get_media(content: String) -> Media {
        let track_url_base = content
            .split("},{\"url\":\"")
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap()
            .to_string();

        let track_url = track_url_base
            .split("\",\"")
            .collect::<Vec<&str>>()
            .get(0)
            .unwrap()
            .to_string();

        Media {
            hls: track_url.replace("/progressive", "/hls").to_string(),
            progressive: track_url,
        }
    }

    pub async fn get_song(url: &str) -> Self {
        let content = Self::get(url).await;
        let title = Self::get_title(content.clone());
        let thumbnail = Self::get_thumbnail(content.clone());
        let duration = Self::get_duration(content.clone());
        let media = Self::get_media(content.clone());
        let client_id = gen_key().await.unwrap();
        let permalink_url = Self::get_permanent_url(content.clone());
        let author = Author::get(content);

        Self {
            title,
            url: permalink_url,
            thumbnail,
            duration,
            media,
            client_id,
            author,
        }
    }
}
