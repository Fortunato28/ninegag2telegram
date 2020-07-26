use anyhow::{anyhow, Result};
use url::Url;

pub fn handle_message(message: &str) -> Result<String> {
    let parsed_link = Url::parse(message)?;

    let mut path_segments = parsed_link
        .path_segments()
        .ok_or_else(|| "Cannot be base")
        .map_err(|err| anyhow!(err))?
        .skip(1); // Skip "photo/"
    let filename = path_segments
        .next()
        .ok_or_else(|| "Error while getting filename")
        .map_err(|err| anyhow!(err))?;

    // Remove vp9 and av1 from filename if contains
    let result_filename = filename.replace("vp9", "");
    let result_filename = result_filename.replace("av1", "");

    let result_link = parsed_link.join(&result_filename)?;
    Ok(result_link.into_string())
}
