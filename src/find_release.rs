//! Finds/downloads the github release
use requestr::{Request, Downloaded};
use objr::bindings::objc_nsstring;
use crate::{Error,InputSnafu,FetchingGithubRunnerSnafu,FetchingGithubRunnerDecodeSnafu};
use snafu::ResultExt;
use crate::Error::FetchingGithubRunnerStatus;

#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    url: String
}
#[derive(serde::Deserialize)]
struct Release {
    assets: Vec<Asset>
}

async fn find_release() -> Result<String,Error> {
    let r = Request::new(objc_nsstring!("https://api.github.com/repos/actions/runner/releases")).context(InputSnafu)?
        .header(objc_nsstring!("Accept"),Some(objc_nsstring!("application/vnd.github.v3+json")))
        .perform().await
        .context(FetchingGithubRunnerSnafu)?;
    let data = r.check_status().map_err(|e| FetchingGithubRunnerStatus {code: e.0})?;
    let response: Vec<Release> = serde_json::from_slice(data.as_slice()).context(FetchingGithubRunnerDecodeSnafu)?;
    let release = response.first().ok_or(Error::FetchingGithubRunnerNoReleases {})?;
    let asset = release.assets.iter().find(|item| {
        item.name.contains("osx-x64")
    }).ok_or(Error::FetchingGithubRunnerNoReleases {})?;
    Ok(asset.url.clone())
}

pub async fn find_and_download_release() -> Result<Downloaded, Error> {
    let url = find_release().await?;
    Ok(Request::new(url).unwrap()
        .header(objc_nsstring!("Accept"),Some(objc_nsstring!("application/octet-stream")))
        .download().await
        .context(FetchingGithubRunnerSnafu)?)
}

#[test] fn find_release_test() {
    let f = find_release();
    let result = kiruna::test::test_await(f, std::time::Duration::from_secs(10));
    println!("{}",result.unwrap());
}
#[test] fn download_release() {
    let f = find_and_download_release();
    let r = kiruna::test::test_await(f,std::time::Duration::from_secs(30));
    println!("{:?}",r.unwrap().as_path());
}