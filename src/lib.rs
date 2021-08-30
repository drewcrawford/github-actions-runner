use objr::bindings::objc_nsstring;
use requestr::{Request};
use snafu::{ResultExt, Snafu};

use org_or_repo::OrgOrRepo;
use crate::authentication::{Authentication};
use crate::token::Token;
use crate::Error::{RegistrationStatusError, ConfigureRunnerError};
use crate::install_runner::install_runner_if_needed;
use kiruna::join::{try_join2};
use crate::configure_runner::is_runner_registered;
use crate::service::install_start_as_needed;

pub mod org_or_repo;
pub mod authentication;
mod token;
mod find_release;
mod install_runner;
mod configure_runner;
mod service;


#[derive(Snafu,Debug)]
pub enum Error {
    InputError { source: requestr::Error },
    RegistrationError{source: requestr::Error },
    RegistrationStatusError{code: u16},
    DecodingError{source: serde_json::Error},
    FetchingGithubRunner{source: requestr::Error },
    FetchingGithubRunnerStatus{code: u16 },
    FetchingGithubRunnerDecode{source: serde_json::Error},
    FetchingGithubRunnerNoReleases {},
    MakeDirectoryError{source: std::io::Error},
    ConfigureRunnerError{source: command_rs::Error},
    ServiceStatusError{source: command_rs::Error},
    ServiceOutputError{source: command_rs::Error},
    Untar{source:command_rs::Error}
}

async fn register_runner<O: OrgOrRepo, A: Authentication>(target: &O,authentication: A) -> Result<Token,Error> {
    let mut url = target.fragment_specifier().to_owned();
    url.push_str("/actions/runners/registration-token");
    let response = Request::new(url).context(InputSnafu)?
        .header(objc_nsstring!("Accept"), Some(objc_nsstring!("application/vnd.github.v3+json")))
        .header(objc_nsstring!("Authorization"), Some(authentication.header()))
        .method(objc_nsstring!("POST"))
    .perform().await.context(RegistrationSnafu)?;
    let response_checked = response.check_status().map_err(|m| RegistrationStatusError {code: m.0})?;
    Ok(Token::from_response(response_checked.as_slice()).context(DecodingSnafu)?)
}



pub async fn ensure_actions_runner<O: OrgOrRepo,A: Authentication>(o: O, a: A) -> Result<(),Error> {
    let installation_task = install_runner_if_needed(o.install_path());
    if is_runner_registered(o.install_path()) {
        //only need to do installation stage
        installation_task.await?;
    }
    else {
        //also need to do registration/configuration stage
        let registration_task = register_runner(&o, a);
        let (token,_) = try_join2(registration_task,installation_task).await.map_err(|e| {
            e.merge()
        })?;
        configure_runner::configure_runner(o.install_path(),&token, &o ).await.map_err(|e| ConfigureRunnerError {source: e})?;
    }
    install_start_as_needed(o.install_path()).await
}

#[cfg(test)]
mod test {
    use crate::{register_runner, ensure_actions_runner};
    use crate::org_or_repo::Repo;
    use crate::authentication::PersonalAuthenticationToken;

    #[test]
    fn register() {
        let target = Repo::new("drewcrawford".to_owned(),"objr".to_owned());
        let f = register_runner(&target, PersonalAuthenticationToken::new("invalid".to_owned()));
        let result = kiruna::test::test_await(f, std::time::Duration::from_secs(10));
        println!("{:?}",result);
    }
    #[test]
    fn integration() {
        let repo = Repo::new("drewcrawford".to_owned(),"objr".to_owned());
        let token = PersonalAuthenticationToken::new("invalid".to_owned());
        let f = ensure_actions_runner(repo,token);
        let _result = kiruna::test::test_await(f, std::time::Duration::from_secs(20));
    }
}

