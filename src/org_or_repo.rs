pub trait OrgOrRepo: 'static {
    fn fragment_specifier(self) -> String;
}

pub struct Repo {
    fragment: String
}

impl Repo {
    pub fn new(org: String, repo: String) -> Self {
        Self { fragment: format!("https://api.github.com/repos/{ORG}/{REPO}",ORG=org,REPO=repo)}
    }
}

impl OrgOrRepo for Repo {
    fn fragment_specifier(self) -> String {
        self.fragment
    }
}
