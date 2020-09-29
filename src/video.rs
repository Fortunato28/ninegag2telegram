use anyhow::{Context, Result};
use bytes::Bytes;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::handle_message;

#[derive(Debug)]
pub struct Video {
    pub filename: String,
    body: Bytes,
}

impl Video {
    pub async fn new(link: &str) -> Result<Video> {
        let handled_link = handle_message::handle_message(link)?;
        let response = Self::download_resource(&handled_link).await;
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
            .arg("-i")
            .arg(filename)
            .arg(&result_filename)
            .output()
            .context("Failed to execute process")?;

        if !coding_result.status.success() {
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
        // Just suppressing possible error
        fs::remove_file(&self.filename).ok();
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
        let file_data: [u8; 5] = [0; 5];
        let result_filename = Video::save_to_fs(filename, &file_data);

        std::fs::remove_file(&filename)?;

        match result_filename {
            Ok(_) => Err(anyhow::anyhow!("Here had to be Result with coding error")),
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
        // Bytes below is just some dummy webm data allowing perform coding to mp4
        let valid_webm = hex::decode("1A45DFA3010000000000001F4286810142F7810142F2810442F381084282847765626D4287810242858102185380670100000000003DCF114D9B74403B4DBB8B53AB841549A96653AC81E54DBB8C53AB841654AE6B53AC8201234DBB8C53AB841254C36753AC8201774DBB8C53AB841C53BB6B53AC823DB2EC010000000000009B00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001549A96601000000000000322AD7B1830F42404D808D4C61766635382E32302E31303057418D4C61766635382E32302E31303044898840590000000000001654AE6B0100000000000048AE010000000000003FD7810173C581019C810022B59C83756E648685565F56503983810123E3838401FCA055E00100000000000013B0820196BA8202D055B08855B7810155B881021254C367010000000000015B7373010000000000009E63C0010000000000000067C8010000000000001545A38B4D414A4F525F4252414E4444878469736F6D67C8010000000000001645A38D4D494E4F525F56455253494F4E44878335313267C8010000000000002745A391434F4D50415449424C455F4252414E445344879069736F6D69736F32617663316D70343167C8010000000000001A45A387454E434F44455244878D4C61766635382E32302E3130307373010000000000006563C0010000000000000463C5810167C8010000000000001E45A38C48414E444C45525F4E414D4544878C566964656F48616E646C657267C8010000000000002545A387454E434F4445524487984C61766335382E33352E313030206C69627670782D7670397373010000000000003A63C0010000000000000463C5810167C8010000000000002245A3884455524154494F4E44879430303A30303A30302E31303030303030303000001F43B6750100000000003AC8E78100A378CF81000080824983420019502CF61238241C18EE0006B07FC9F9")?;
        let result_filename = Video::save_to_fs(filename, &valid_webm)?;

        assert_eq!("some_file_4.mp4", result_filename);

        std::fs::remove_file(&result_filename)?;

        Ok(())
    }
}
