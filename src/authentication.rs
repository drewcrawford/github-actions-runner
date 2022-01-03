/*! Authentication behavior*/

/**Any authentication type.
*/
pub trait Authentication: 'static {
    ///Returns the header to use for the request.
    fn header(self) -> String;
}
///A personal authentication token
#[derive(Clone)]
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