/*! Rust-language install script for [github actions runner](https://github.com/actions/runner).

Useful in wrangling self-hosted runners.  Provides a rich API in the Rust typesystem for configuration.

# Status

Works on macOS 12+.

 # Example
```
use github_actions_runner::{ensure_actions_runner,Repo,PersonalAuthenticationToken};
async fn example() -> Result<(),Box<dyn std::error::Error>> {
    let repo = Repo::new("drewcrawford".to_owned(),"github-actions-runner".to_owned());
    let token = PersonalAuthenticationToken::new("invalid".to_owned());
    let f = ensure_actions_runner(repo,token, kiruna::Priority::Testing);
    f.await?;
    Ok(())
}
 ```

 # Similar libraries

Since Rust is a compiled language, its binaries are self-contained.  Therefore you can write tools to bring up a production or
development environment in Rust itself, compile them, and shoot them over to new servers via SSH.

You might be interested my expanded universe of sysadmin libraries:

* [rustupr](https://github.com/drewcrawford/rustupr), which installs Rust
* [mac-install](https://github.com/drewcrawford/mac-install) which installs mac packages
* [dmg](https://github.com/drewcrawford/dmg) to mount DMG images
*/
use requestr::{Request};
use snafu::{ResultExt, Snafu};
use pcore::pstr;

pub use org_or_repo::Repo;
pub use authentication::PersonalAuthenticationToken;

use org_or_repo::OrgOrRepo;
use crate::authentication::{Authentication};
use crate::token::Token;
use crate::Error::{RegistrationStatusError, ConfigureRunnerError};
use crate::install_runner::install_runner_if_needed;
use kiruna::join::{try_join2};
use pcore::release_pool::autoreleasepool;
use crate::configure_runner::is_runner_registered;
use crate::service::install_start_as_needed;

mod org_or_repo;
mod authentication;
mod token;
mod find_release;
mod install_runner;
mod configure_runner;
mod service;

#[non_exhaustive]
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
    let request = autoreleasepool(|pool| {
        Request::new(url, pool)
    }).context(InputSnafu)?;
    let fut1 = autoreleasepool(|pool| {
        request
            .header(pstr!("Accept"), Some(pstr!("application/vnd.github.v3+json")), pool)
            .header(pstr!("Authorization"), Some(authentication.header()), pool)
            .method(pstr!("POST"),pool)
            .perform(pool)
    });
    let r1 = fut1.await.context(RegistrationSnafu)?;
   let response_checked = autoreleasepool(|pool| {
        r1.check_status(pool).map_err(|m| RegistrationStatusError {code: m.0})
    })?;

    Ok(Token::from_response(response_checked.as_slice()).context(DecodingSnafu)?)
}


/**
Installs and sets up the github actions runner, if it is not already installed.

# Example
```
use github_actions_runner::{ensure_actions_runner,Repo,PersonalAuthenticationToken};
async fn example() -> Result<(),Box<dyn std::error::Error>> {
    let repo = Repo::new("drewcrawford".to_owned(),"github-actions-runner".to_owned());
    let token = PersonalAuthenticationToken::new("invalid".to_owned());
    let f = ensure_actions_runner(repo,token, kiruna::Priority::Testing);
    f.await?;
    Ok(())
}
```
*/
pub async fn ensure_actions_runner<O: OrgOrRepo,A: Authentication>(o: O, a: A, priority: kiruna::Priority) -> Result<(),Error> {
    let installation_task = install_runner_if_needed(o.install_path(), priority);
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
        configure_runner::configure_runner(o.install_path(),&token, &o, priority).await.map_err(|e| ConfigureRunnerError {source: e})?;
    }
    install_start_as_needed(o.install_path(), priority).await
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
        let f = ensure_actions_runner(repo,token, kiruna::Priority::Testing);
        let _result = kiruna::test::test_await(f, std::time::Duration::from_secs(20));
    }
}

