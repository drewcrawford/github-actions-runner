use objr::bindings::objc_nsstring;
use requestsr::{Request, with_client_async};
use snafu::{ResultExt, Snafu};

use org_or_repo::OrgOrRepo;
use crate::authentication::{Authentication};
use crate::token::Token;
use crate::Error::{RegistrationStatusError};

pub mod org_or_repo;
pub mod authentication;
mod token;



#[derive(Snafu,Debug)]
pub enum Error {
    InputError { source: requestsr::Error },
    RegistrationError{source: requestsr::Error },
    RegistrationStatusError{code: u16},
    DecodingError{source: serde_json::Error}
}

async fn register_runner_thunk<O: OrgOrRepo, A: Authentication>(client: &requestsr::ActiveClient, more_args: (O,A)) -> Result<Token,Error> {
    //at the time of this writing, with_client_async requires a bare fn argument
    let target = more_args.0;
    let authentication = more_args.1;
    let mut url = target.fragment_specifier();
    url.push_str("/actions/runners/registration-token");
    let response = Request::new(url, client).context(InputSnafu)?
        .header(objc_nsstring!("Accept"), Some(objc_nsstring!("application/vnd.github.v3+json")))
        .header(objc_nsstring!("Authorization"), Some(authentication.header()))
        .method(objc_nsstring!("POST"))
    .perform().await.context(RegistrationSnafu)?;
    let response_checked = response.check_status().map_err(|m| RegistrationStatusError {code: m.0})?;
    Ok(Token::from_response(response_checked.as_slice()).context(DecodingSnafu)?)
}


async fn register_runner<O: OrgOrRepo,A: Authentication>(target: O, authentication: A) -> Result<Token,Error> {
    with_client_async(register_runner_thunk,(target,authentication)).await
}
async fn is_runner_registered() -> bool {
    false //todo
}
pub async fn ensure_actions_runner<O: OrgOrRepo,A: Authentication>(o: O, a: A) -> Result<(),Error> {
    if !is_runner_registered().await {
        register_runner(o, a).await?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::register_runner;
    use crate::org_or_repo::Repo;
    use crate::authentication::PersonalAuthenticationToken;

    #[test]
    fn register() {

        let f = register_runner(Repo::new("drewcrawford".to_owned(),"objr".to_owned()), PersonalAuthenticationToken::new("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".to_owned()));
        let result = kiruna::test::test_await(f, std::time::Duration::from_secs(10)).unwrap();
        println!("{:?}",result);
    }
}

