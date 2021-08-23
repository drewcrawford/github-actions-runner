pub trait Authentication: 'static {
    fn header(self) -> String;
}
pub struct PersonalAuthenticationToken {
    header: String
}
impl PersonalAuthenticationToken {
    pub fn new(token: String) -> Self {
        PersonalAuthenticationToken {
            header: format!("token {TOKEN}",TOKEN=token)
        }
    }
}
impl Authentication for PersonalAuthenticationToken {
    fn header(self) -> String {
        self.header
    }
}