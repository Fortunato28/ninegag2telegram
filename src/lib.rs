use bytes::Bytes;
use std::error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use teloxide::prelude::*;
use url::Url;

// TODO error handling with ParseError
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

fn handle_message(message: &str) -> Result<String, Box<dyn error::Error + Send + Sync + 'static>> {
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
            rx.for_each_concurrent(None, |message| async move {
                match &message.update.text() {
                    Some(link) => match &handle_message(link) {
                        Ok(respond) => {
                            let video = Video::new(&respond).await;

                            let path_to_result = PathBuf::from(&video.filename);
                            message
                                .answer_video(teloxide::types::InputFile::File(path_to_result))
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                        Err(_) => {
                            message
                                .answer("Probably your message is not a valid url, I can't handle it.")
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                    },
                    None => {
                        message
                            .answer("Your message is not plain text, I can`t handle it.")
                            .send()
                            .await
                            .log_on_error()
                            .await;
                    }
                }
            })
        })
        .dispatch()
        .await;
}
