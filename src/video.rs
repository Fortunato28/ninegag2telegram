use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Video {
    pub filename: String,
    body: Bytes,
}

impl Video {
    pub async fn new(link: &str) -> Result<Video> {
        let response = Self::download_resource(link).await;
        let filename = Self::get_filename(&response.url());
        let body = Self::get_body(response).await?;
        let filename = Self::save_to_fs(&filename, &body)?;

        Ok(Video { filename, body })
    }

    fn save_to_fs(filename: &str, body: &[u8]) -> Result<String> {
        let mut destination = File::create(filename).context("Unable to create file")?;
        destination
            .write_all(body)
            .context("Unable write data to the file")?;
        Self::to_mp4(filename)
    }

    async fn get_body(response: reqwest::Response) -> Result<Bytes> {
        let body = response
            .bytes()
            .await
            .context("Problem while getting response body");
        body
    }

    fn to_mp4(filename: &str) -> Result<String> {
        match filename.find(".mp4") {
            Some(_) => return Ok(filename.to_owned()),
            None => {}
        }

        let result_filename = filename.replace("webm", "mp4");

        let coding_result = Command::new("ffmpeg")
            .stdout(Stdio::null())
            .arg("-i")
            .arg(filename)
            .arg(&result_filename)
            .status()
            .context("Failed to execute process")?;

        if !coding_result.success() {
            return Err(anyhow::Error::msg("Unable to decode webm to mp4"));
        }

        fs::remove_file(filename)?;
        Ok(result_filename)
    }

    fn get_filename(response_url: &reqwest::Url) -> String {
        response_url
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::path;

    #[test]
    fn correct_filename() -> Result<()> {
        // Make test url
        let url =
            reqwest::Url::parse("https://img-9gag-fun.9cache.com/photo/a2WL8RE_460svav1.mp4")?;
        let test_filename = Video::get_filename(&url);

        assert_eq!(test_filename, "a2WL8RE_460svav1.mp4");

        Ok(())
    }

    #[test]
    fn empty_filename() -> Result<()> {
        // Make test url
        let url = reqwest::Url::parse("https://img-9gag-fun.9cache.com/photo/")?;
        let test_filename = Video::get_filename(&url);

        assert_eq!(test_filename, "tmp.bin");

        Ok(())
    }

    #[test]
    fn mp4_to_fs_filename() -> Result<()> {
        // That filenames have to be different in each test
        let filename = "some_file_1.mp4";
        let file_data: [u8; 5] = [0; 5];
        let result_filename = Video::save_to_fs(filename, &file_data)?;

        assert_eq!(filename, result_filename);

        std::fs::remove_file(result_filename)?;
        Ok(())
    }

    #[test]
    fn mp4_to_fs_saved_properly() -> Result<()> {
        let filename = "some_file_2.mp4";
        let file_data: [u8; 5] = [0; 5];
        let result_filename = Video::save_to_fs(filename, &file_data)?;

        assert!(path::Path::new(&result_filename).exists());

        std::fs::remove_file(result_filename)?;
        Ok(())
    }

    #[test]
    fn webm_to_fs_conding_fail() -> Result<()> {
        // That filenames have to be different in each test
        let filename = "some_file_3.webm";
        // TODO make good file_data
        let file_data: [u8; 5] = [0; 5];
        let result_filename = Video::save_to_fs(filename, &file_data);

        std::fs::remove_file(&filename)?;

        match result_filename {
            Ok(_) => Err(anyhow!("Here had to be Result with coding error")),
            Err(err) => {
                assert_eq!(err.to_string(), "Unable to decode webm to mp4");
                Ok(())
            }
        }
    }

    #[test]
    fn webm_to_fs_filename() -> Result<()> {
        // That filenames have to be different in each test
        let filename = "some_file_4.webm";
        // TODO make good file_data
        //let file_data: [u8; 5] = [0; 5];
        //let result_filename = Video::save_to_fs(filename, &file_data)?;

        //assert_eq!("some_file_2.mp4", result_filename);

        Ok(())
    }
}
