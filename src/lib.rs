use teloxide::prelude::*;
// TODO error handling with ParseError
use url::Url;

// TODO remove all panics!
pub fn handle_message(message: &str) -> String {
    let parsed_link = Url::parse(message).expect("Parsing Error!");

    dbg!(&parsed_link.as_str());

    let mut path_segments = parsed_link.path_segments()
        .ok_or_else(|| "cannot be base")
        .expect("Error while find filename")
        .skip(1); // Skip "photo"
    let filename = path_segments.next().expect("Error while getting filename");

    // Remove vp9 and av1 from filename if contains
    let result_filename = filename.replace("vp9", "");
    let result_filename = result_filename.replace("av1", "");

    dbg!(&result_filename);
    let result_link = parsed_link.join(&result_filename).expect("Error while join");
    dbg!(&result_link.as_str());
    result_link.into_string()
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
                let respond = handle_message(link);
                message.answer(&respond).send().await.log_on_error().await;

            })
        })
        .dispatch()
        .await;
}
