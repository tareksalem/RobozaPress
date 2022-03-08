#[derive(Debug, Clone)]
pub enum Error {
    APIError,
    LanguageError,
}
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        dbg!(error);

        Error::APIError
    }
}
impl From<link_preview::fetch::Error> for Error{
    fn from(error: link_preview::fetch::Error) -> Error {
        dbg!(error);
        Error::APIError
    }
}


pub fn truncate_with_dots(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => {
            s.to_string()
        },
        Some((idx, _)) => {
            let mut val = s[..idx].to_string();
            val.push_str("...");
            val
        },
    }
}
