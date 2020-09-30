use std::path::PathBuf;
use teloxide::prelude::*;

mod handle_message;
mod video;

pub async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting ping_pong_bot!");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, |message| async move {
                match &message.update.text() {
                    Some(link) => {
                        // TODO delete fucking unwrap
                        let wait_message = message.answer("OK I got it, wait a little").send().await.unwrap();

                        match video::Video::new(&link).await {
                        Ok(video) => {
                            let path_to_result = PathBuf::from(&video.filename);
                            message
                                .answer_video(teloxide::types::InputFile::File(path_to_result))
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                        Err(err) => {
                            message
                                .answer(
                                    "Probably your message is not a valid url to 9gag video, I can't handle it:\n".to_owned() + &err.to_string(),
                                )
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                    }

                        // Delete message "wait pls"
                        message.delete_message().message_id(wait_message.id).send().await.log_on_error().await;
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
