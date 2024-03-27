use validator::ValidateUrl;

#[derive(Debug)]
pub struct LongUrl(String);

impl LongUrl {
    pub fn parse(s: Option<String>) -> Result<LongUrl, String> {
        if s.is_none() {
            return Err("Invalid JSON data: missing field longUrl".to_string());
        }
        let s = s.unwrap();
        if s.trim().is_empty() {
            return Err("Invalid JSON data: field longUrl cannot be empty".to_string());
        }
        if !ValidateUrl::validate_url(&s) {
            return Err(format!("Invalid JSON data: '{s}' is not a valid long URL"));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for LongUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::LongUrl;
    use claims::{assert_err, assert_ok};

    #[test]
    fn empty_string_is_rejected() {
        let url = Option::from("".to_string());
        assert_err!(LongUrl::parse(url));
    }

    #[test]
    fn correct_url_format_is_accepted() {
        let url = Option::from("http://localhost.com".to_string());
        assert_ok!(LongUrl::parse(url));
    }

    #[test]
    fn non_url_is_rejected() {
        let url = Option::from("hello".to_string());
        assert_err!(LongUrl::parse(url));
    }
}
