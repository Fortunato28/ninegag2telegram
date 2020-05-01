use bytes::Bytes;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use teloxide::prelude::*;
// TODO error handling with ParseError
use std::error;
use std::io::Write;
use url::Url;

// new have to create video on the filesystem, drop should remove it
pub struct Video {
    pub filename: String,
    body: Bytes,
}

impl Video {
    pub async fn new(link: &str) -> Video {
        let response = Self::download_resource(link).await;
        let filename = Self::get_filename(&response);
        let body = Self::get_body(response).await;
        Self::save_to_fs(&filename, &body);

        Video { filename, body }
    }

    fn save_to_fs(filename: &str, body: &[u8]) {
        let mut destination = File::create(filename).expect("Problem while create file");
        destination.write_all(body);
    }

    async fn get_body(response: reqwest::Response) -> Bytes {
        let body = response
            .bytes()
            .await
            .expect("Problem while getting response body");
        body
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

// TODO not pub!
pub fn handle_message(message: &str) -> Result<String, Box<dyn error::Error>> {
    let parsed_link = Url::parse(message)?;

    let mut path_segments = parsed_link
        .path_segments()
        .ok_or_else(|| "cannot be base")?
        .skip(1); // Skip "photo"
    let filename = path_segments
        .next()
        .ok_or_else(|| "Error while getting filename")?;

    // Remove vp9 and av1 from filename if contains
    let result_filename = filename.replace("vp9", "");
    let result_filename = result_filename.replace("av1", "");

    let result_link = parsed_link.join(&result_filename)?;
    Ok(result_link.into_string())
}

pub async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting ping_pong_bot!");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each(|message| async move {
                let link = &message.update.text().unwrap_or("This is not a text dude");

                let respond =
                    handle_message(link).unwrap_or("Sorry, I can`t handle this message".to_owned());
                let video = Video::new(&respond).await;

                let path_to_result = PathBuf::from(&video.filename);
                message
                    .answer_video(teloxide::types::InputFile::File(path_to_result))
                    .send()
                    .await
                    .log_on_error()
                    .await;
            })
        })
        .dispatch()
        .await;
}
