#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String
}
#[derive(Debug)]
pub struct Token(String);
impl Token {
    pub fn from_response(data: &[u8]) -> Result<Self,serde_json::Error> {
        let response: TokenResponse = serde_json::from_slice(data)?;
        Ok(Token(response.token))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

