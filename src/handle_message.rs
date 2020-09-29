use anyhow::Result;
use url::Url;

pub fn handle_message(message: &str) -> Result<String> {
    let parsed_link = Url::parse(message)?;
    let filename = extract_filename(&parsed_link)?;
    dbg!(&filename);

    let result_filename = transform_ninegag_name(&filename);

    let result_link = parsed_link.join(&result_filename)?;
    Ok(result_link.into_string())
}

fn extract_filename(parsed_link: &Url) -> Result<String> {
    Ok(parsed_link
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| {
            if name.contains("webm") || name.contains("mp4") {
                Some(name)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("Filename is not webm or mp4"))?
        .to_owned())
}

fn transform_ninegag_name(filename: &str) -> String {
    // Remove vp9 and av1 from filename if contains
    // That suffixes are some custom encoding ninegag specific
    let result_filename = filename.replace("vp9", "");
    let result_filename = result_filename.replace("av1", "");

    result_filename
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn handle_correct_message() {
        let message = "https://img-9gag-fun.9cache.com/photo/aeDQMYq_460svav1.mp4";
        let handled = handle_message(message).unwrap();
        assert_eq!(
            handled,
            "https://img-9gag-fun.9cache.com/photo/aeDQMYq_460sv.mp4"
        );
    }

    #[test]
    fn handle_not_url_message() {
        let message = "owowoowo";
        match handle_message(message) {
            Ok(_) => panic!(),
            Err(err) => assert_eq!(
                err.downcast::<url::ParseError>().unwrap(),
                url::ParseError::RelativeUrlWithoutBase
            ),
        };
    }

    #[test]
    fn handle_empty_host_message() {
        let message = "https://";
        match handle_message(message) {
            Ok(_) => panic!(),
            Err(err) => assert_eq!(
                err.downcast::<url::ParseError>().unwrap(),
                url::ParseError::EmptyHost
            ),
        };
    }

    #[test]
    fn filename_form_url() {
        let fullname = Url::parse("https://img-9gag-fun.9cache.com/photo/aeDQMYq_460svav1.mp4")
            .expect("Error while parse url");
        let filename = extract_filename(&fullname).expect("Error while unwrap filename");
        assert_eq!(filename, "aeDQMYq_460svav1.mp4");
    }

    #[test]
    fn filename_error() -> Result<()> {
        let fullname = Url::parse("https://img-9gag-fun.9cache.com/")?;
        match extract_filename(&fullname) {
            Ok(_) => Err(anyhow::anyhow!("Here had to be filename error")),
            Err(err) => {
                assert_eq!(err.to_string(), "Filename is not webm or mp4");
                Ok(())
            }
        }
    }

    #[test]
    fn transform_vp9_name() {
        let name_webm = "aXgnj6P_460svvp9.webm";
        let result_name = transform_ninegag_name(name_webm);
        assert_eq!(result_name, "aXgnj6P_460sv.webm");
    }

    #[test]
    fn transform_av1_name() {
        let name_mp4 = "aeDQMYq_460svav1.mp4";
        let result_name = transform_ninegag_name(name_mp4);
        assert_eq!(result_name, "aeDQMYq_460sv.mp4");
    }

    #[test]
    fn transform_good_name() {
        let good_name = "arVmMEy_460sv.mp4";
        let result_name = transform_ninegag_name(good_name);
        assert_eq!(result_name, good_name);
    }

    #[test]
    fn not_video_filename() -> Result<()> {
        // Make test url
        let url = reqwest::Url::parse("https://img-9gag-fun.9cache.com/photo/some_file.jpg")?;
        let error = extract_filename(&url);

        match error {
            Ok(_) => Err(anyhow::anyhow!(
                "Here had to be Result with wrong filename error"
            )),
            Err(err) => {
                assert_eq!(err.to_string(), "Filename is not webm or mp4");
                Ok(())
            }
        }
    }
}
