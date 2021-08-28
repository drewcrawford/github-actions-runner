use std::path::{PathBuf};
use crate::find_release::{download_release,find_release};
use crate::Error;
use snafu::ResultExt;
use crate::{MakeDirectorySnafu,UntarSnafu};
use command_rs::Command;
use std::io::ErrorKind;

///Installs the runner to some a path in here
pub async fn install_runner_if_needed(within_path: PathBuf) -> Result<(),Error> {
    let found_release = find_release().await?;
    if is_runner_uptodate(within_path.clone(), found_release.cli_version()).await {
        return Ok(())
    }
    std::fs::create_dir(within_path.clone())
        //ignore "already exists" error
        .map_or_else(|e|
                         if e.kind() == ErrorKind::AlreadyExists {
                             Ok(())
                         }
                         else {
                             Err(e)
                         },
                             |_| std::io::Result::Ok(()))
        .context(MakeDirectorySnafu)?;
    std::env::set_current_dir(within_path.clone()).context(MakeDirectorySnafu)?;
    let download = download_release(found_release).await?;
    Command::new("tar")
        .arg("xzf").arg(download.as_path())
        .status().await.context(UntarSnafu)?;
    println!("Installed to {:?}",within_path);
    Ok(())
}
pub async fn is_runner_uptodate(mut within_path: PathBuf,latest_version: &str) -> bool {
    within_path.push("config.sh");
    let result = Command::new(within_path)
        .arg("--version")
        .output(kiruna::Priority::UserWaiting)
        .await;
    match result {
        Ok(output) => {
            let str = std::str::from_utf8(output.stdout.as_slice()).unwrap();
            if str.starts_with(latest_version) {
                true
            }
            else {
                false
            }

        }
        Err(_) => {false}
    }
}

#[test] fn is_runner_up2date() {
    use std::str::FromStr;
    let r = is_runner_uptodate(PathBuf::from_str("/Users/drew/github-actions-runner/drewcrawford_objr").unwrap(), "2.280.3");
    let r2 = kiruna::test::test_await(r, std::time::Duration::from_secs(2));
    println!("{:?}",r2);
}