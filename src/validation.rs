use regex::Regex;

pub fn validate_username(username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    re.is_match(username)
}
pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

pub fn validate_generic(input: &str) -> bool {
    //* disallows dangerous characters (example: <, >, ', ", &, etc.)
    let re = Regex::new("^[^<>'\"&]+$").unwrap();
    re.is_match(input)
}
