//! Finds/downloads the github release
use requestr::{Request, Downloaded};
use crate::{Error,InputSnafu,FetchingGithubRunnerSnafu,FetchingGithubRunnerDecodeSnafu};
use snafu::ResultExt;
use crate::Error::FetchingGithubRunnerStatus;
use pcore::pstr;
use pcore::release_pool::autoreleasepool;

/* Parse the types from the response */
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

///Finds the newest release of github actions runner by consulting the releases via network
pub(crate) async fn find_release() -> Result<FoundRelease,Error> {
    let r1 = autoreleasepool(|pool| {
        Request::new(pstr!("https://api.github.com/repos/actions/runner/releases"), pool).context(InputSnafu)
    })?;
    let r1 = autoreleasepool(|pool| {
        r1
            .header(pstr!("Accept"),Some(pstr!("application/vnd.github.v3+json")), pool)
            .perform(pool)
    });
    let r = r1.await
        .context(FetchingGithubRunnerSnafu)?;
    let data = autoreleasepool(|pool| {
        r.check_status(pool).map_err(|e| FetchingGithubRunnerStatus {code: e.0})
    });
    let data = data?;
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
///Downloads the newest release.
pub (crate) async fn download_release(found_release: FoundRelease) -> Result<Downloaded, Error> {
    let t = autoreleasepool(|pool| {
        Ok(Request::new(found_release.url, pool).unwrap()
               .header(pstr!("Accept"),Some(pstr!("application/octet-stream")), pool)
               .download(pool)
        )});
    t?.await.context(FetchingGithubRunnerSnafu)
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
    println!("{:?}",r.unwrap().copy_path().as_path());
}