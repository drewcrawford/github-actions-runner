/*! Functionality for runner tokens */
///A type that holds token responses, parsed from the response.
#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String
}
///A newtype for a runner token
#[derive(Debug)]
pub struct Token(String);
impl Token {
    ///Parse token from response
    pub fn from_response(data: &[u8]) -> Result<Self,serde_json::Error> {
        let response: TokenResponse = serde_json::from_slice(data)?;
        Ok(Token(response.token))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

