use teloxide::prelude::*;
// TODO error handling with ParseError
use url::Url;
use std::error;

// TODO remove all panics!
pub fn handle_message(message: &str) -> Result<String, Box<dyn error::Error>> {
    let parsed_link = Url::parse(message)?;

    dbg!(&parsed_link.as_str());

    let mut path_segments = parsed_link.path_segments()
        .ok_or_else(|| "cannot be base")
        ?
        .skip(1); // Skip "photo"
    let filename = path_segments.next().ok_or_else(|| "Error while getting filename")?;

    // Remove vp9 and av1 from filename if contains
    let result_filename = filename.replace("vp9", "");
    let result_filename = result_filename.replace("av1", "");

    dbg!(&result_filename);
    let result_link = parsed_link.join(&result_filename)?;
    dbg!(&result_link.as_str());
    Ok(result_link.into_string())
}

pub async fn run() {
teloxide::enable_logging!();
    log::info!("Starting ping_pong_bot!");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each(|message| async move {
                let link = &message.update.text()
                    .unwrap_or("This is not a text dude");

                let respond = handle_message(link)
                    .unwrap_or("Sorry, I can`t handle this message".to_owned());
                message.answer(&respond).send().await.log_on_error().await;

            })
        })
        .dispatch()
        .await;
}
