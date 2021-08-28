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
    name: String,
    assets: Vec<Asset>
}

#[derive(Debug)]
pub(crate) struct FoundRelease {
    url: String,
    version: String
}
impl FoundRelease {
    pub(crate) fn cli_version(&self) -> &str {
       &self.version
    }
}

pub(crate) async fn find_release() -> Result<FoundRelease,Error> {
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
    Ok(
        FoundRelease {
            url: asset.url.clone(),
            version: release.name.strip_prefix("v").unwrap().to_owned()
        }
    )
}
pub (crate) async fn download_release(found_release: FoundRelease) -> Result<Downloaded, Error> {
    Ok(Request::new(found_release.url).unwrap()
        .header(objc_nsstring!("Accept"),Some(objc_nsstring!("application/octet-stream")))
        .download().await
        .context(FetchingGithubRunnerSnafu)?)
}


#[test] fn find_release_test() {
    let f = find_release();
    let result = kiruna::test::test_await(f, std::time::Duration::from_secs(10));
    println!("{:?}",result.unwrap());
}
#[test] fn test_download_release() {
    let r = FoundRelease {
        url: "https://sealedabstract.com".to_string(),
        version: "".to_string()
    };
    let f = download_release(r);
    let r = kiruna::test::test_await(f,std::time::Duration::from_secs(30));
    println!("{:?}",r.unwrap().as_path());
}