use std::error;
use std::path::PathBuf;
use teloxide::prelude::*;
use url::Url;

mod video;

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
                        Ok(respond) => match video::Video::new(&respond).await {
                            Ok(video) => {
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
                                .answer(
                                    "Probably your message is not a valid url, I can't handle it.",
                                )
                                .send()
                                .await
                                .log_on_error()
                                .await;
                            }
                        },
                        Err(_) => {
                            message
                                .answer(
                                    "Probably your message is not a valid url, I can't handle it.",
                                )
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
