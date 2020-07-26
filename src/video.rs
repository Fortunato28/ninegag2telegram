use anyhow::{Context, Result};
use bytes::Bytes;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

// TODO error handling with ParseError
#[derive(Debug)]
pub struct Video {
    pub filename: String,
    body: Bytes,
}

impl Video {
    pub async fn new(link: &str) -> Result<Video> {
        let response = Self::download_resource(link).await;
        let filename = Self::get_filename(&response);
        let body = Self::get_body(response).await?;
        let filename = Self::save_to_fs(&filename, &body);

        Ok(Video { filename, body })
    }

    fn save_to_fs(filename: &str, body: &[u8]) -> String {
        let mut destination = File::create(filename).expect("Problem while create file");
        destination.write_all(body);
        Self::to_mp4(filename)
    }

    async fn get_body(response: reqwest::Response) -> Result<Bytes> {
        let body = response
            .bytes()
            .await
            .context("Problem while getting response body");
        body
    }

    pub fn to_mp4(filename: &str) -> String {
        match filename.find(".mp4") {
            Some(_) => return filename.to_owned(),
            None => {}
        }

        let result_filename = filename.replace("webm", "mp4");

        Command::new("ffmpeg")
            .stdout(Stdio::null())
            .arg("-i")
            .arg(filename)
            .arg(&result_filename)
            .output()
            .expect("Failed to execute process");

        fs::remove_file(filename);
        result_filename
    }

    fn get_filename(response: &reqwest::Response) -> String {
        response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin")
            .to_owned()
    }

    async fn download_resource(link: &str) -> reqwest::Response {
        let client = reqwest::Client::new();
        client
            .get(link)
            .send()
            .await
            .expect("Problem while GET request")
    }
}

impl Drop for Video {
    fn drop(&mut self) {
        dbg!(&self.filename);
        fs::remove_file(&self.filename);
    }
}
