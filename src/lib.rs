use std::path::PathBuf;
use teloxide::prelude::*;

type TeloxideMessage = teloxide::dispatching::DispatcherHandlerCx<Message>;

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
                        answer_to_user(&message, link).await;
                    }
                    None => {
                        not_plain_text(&message).await;
                    }
                }
            })
        })
        .dispatch()
        .await;
}

async fn answer_to_user(message: &TeloxideMessage, link: &str) {
    if let Ok(wait_message) = message.answer("OK, I got it, wait a little").send().await {
        match video::Video::new(&link).await {
            Ok(video) => {
                send_video_to_user(&video, &message).await;
            }
            Err(err) => {
                send_err_to_user(&err, &message).await;
            }
        }
        delete_wait_message(&message, &wait_message).await;
    }
}

async fn not_plain_text(message: &TeloxideMessage) {
    message
        .answer("Your message is not plain text, I can`t handle it")
        .send()
        .await
        .log_on_error()
        .await;
}

async fn send_video_to_user(video: &video::Video, message: &TeloxideMessage) {
    let path_to_result = PathBuf::from(&video.filename);
    message
        .answer_video(teloxide::types::InputFile::File(path_to_result))
        .send()
        .await
        .log_on_error()
        .await;
}

async fn send_err_to_user(err: &anyhow::Error, message: &TeloxideMessage) {
    message
        .answer(
            "Probably your message is not a valid url to 9gag video, I can't handle it:\n"
                .to_owned()
                + &err.to_string(),
        )
        .send()
        .await
        .log_on_error()
        .await;
}

async fn delete_wait_message(message: &TeloxideMessage, wait_message: &Message) {
    message
        .delete_message()
        .message_id(wait_message.id)
        .send()
        .await
        .log_on_error()
        .await;
}
