/*! Install/configuration */
use crate::token::Token;
use std::path::PathBuf;
use command_rs::{Command, ExitStatus};
use crate::org_or_repo::OrgOrRepo;

/**Determines if the runner is already registered or not.*/
pub fn is_runner_registered(mut within_path: PathBuf) -> bool {
    within_path.push(".runner");
    within_path.exists()
}

/**Runs `config.sh`*/
pub async fn configure_runner<O: OrgOrRepo>(mut install_path: PathBuf, token: &Token,target: &O, priority: kiruna::Priority) -> Result<(),command_rs::Error> {
    install_path.push("config.sh");
    println!("Installing to {:?}",install_path);
    Command::new(install_path)
        /*
        Config Options:
 --unattended           Disable interactive prompts for missing arguments. Defaults will be used for missing options
 --url string           Repository to add the runner to. Required if unattended
 --token string         Registration token. Required if unattended
 --name string          Name of the runner to configure (default shadowfax)
 --runnergroup string   Name of the runner group to add this runner to (defaults to the default runner group)
 --labels string        Extra labels in addition to the default: 'self-hosted,OSX,X64'
 --work string          Relative runner work directory (default _work)
 --replace              Replace any existing runner with the same name (default false)
 --pat                  GitHub personal access token used for checking network connectivity when executing `./run.sh --check`
         */
        .arg("--unattended")
        .arg("--url")
        .arg(target.runner_fragment())
        .arg("--token")
        .arg(token.as_str())
        .status(priority)
        .await?.check_err()

}