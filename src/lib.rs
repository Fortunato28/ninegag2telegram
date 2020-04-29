use teloxide::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use bytes::Bytes;
use std::process::{Command, Stdio};

pub struct Video {
    pub filename: String,
    body: Bytes,
}

impl Video {
    pub async fn new(link: &str) -> Video {
        let response = Self::download_resource(link).await;
        let filename = Self::get_filename(&response);
        let body = Self::get_body(response).await;

        Video {
            filename,
            body,
        }
    }

    pub fn save_to_fs(&self, folder: &str) {
        fs::create_dir(&folder);
        let full_filename = get_full_filename(folder, &self.filename);
        write_to_file(&full_filename, &self.body);
    }

    async fn get_body(response: reqwest::Response) -> Bytes {
        let body = response.bytes().await.expect("Problem while getting response body");
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
        client.get(link).send().await.expect("Problem while GET request")
    }
}

// TODO fix case when folder does not have slash
pub fn get_full_filename(folder: &str, filename: &str) -> String {
        let mut full_filename = String::from(folder);
        full_filename.push_str(&filename);
        full_filename
}

fn write_to_file(filename: &str, body: &[u8])
{
    let mut destination = File::create(filename).expect("Problem while create file");
    destination.write_all(body);
}

pub fn to_mp4(dir: &str, filename: &str, result_filename: &str) {
    Command::new("ffmpeg")
        .stdout(Stdio::null())
        .arg("-i")
        .arg(&get_full_filename(dir, filename))
        .arg(&get_full_filename(dir, result_filename))
        .output()
        .expect("Failed to execute process");
}

pub async fn run() {
teloxide::enable_logging!();
    log::info!("Starting ping_pong_bot!");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each(|message| async move {
                let link = &message.update.text().expect("Faild while read link");
                dbg!(&link);

                //let temp_dir = "tmp/";
                //let result_filename = "result.mp4";
                //let video = Video::new(link).await;
                //video.save_to_fs(temp_dir);
                //to_mp4(&temp_dir, &video.filename, result_filename);

                //let path_to_result = PathBuf::from(get_full_filename(&temp_dir, result_filename));
                //dbg!(&path_to_result);
                //message
                //    .answer_video(teloxide::types::InputFile::File(path_to_result))
                //    .send()
                //    .await
                //    .log_on_error()
                //    .await;

                //fs::remove_dir_all(temp_dir);
            })
        })
        .dispatch()
        .await;
}
