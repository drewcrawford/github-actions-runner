use std::path::PathBuf;

/**A place runners can be installed.

 Runners can be associated with an entire org, or just a repo (for personal runners).*/
pub trait OrgOrRepo: 'static {
    fn fragment_specifier(&self) -> String;
    fn runner_fragment(&self) -> String;
    //github runners need to be installed to different paths
    //on a per-repo (or as configured on github's side) basis
    fn install_path(&self) -> PathBuf;
}

/**Refers to a repository.

Includes the org of the repository (which may be a username) */
pub struct Repo {
    org: String,
    repo: String
}

impl Repo {
    pub fn new(org: String, repo: String) -> Self {
        Self { org, repo}
    }
}

impl OrgOrRepo for Repo {
    fn fragment_specifier(&self) -> String {
        format!("https://api.github.com/repos/{ORG}/{REPO}", ORG = self.org, REPO=self.repo)
    }
    fn runner_fragment(&self) -> String {
        format!("https://github.com/{ORG}/{REPO}", ORG=self.org, REPO=self.repo)
    }
    fn install_path(&self) -> PathBuf {
        let mut path = PathBuf::from(std::env::var("HOME").unwrap());
        path.push("github-actions-runner");
        path.push(format!("{ORG}_{REPO}", ORG=self.org, REPO=self.repo));
        path
    }
}
