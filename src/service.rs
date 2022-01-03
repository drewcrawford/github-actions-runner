use std::path::PathBuf;
use command_rs::Command;
use crate::{Error,ServiceOutputSnafu,ServiceStatusSnafu};
use snafu::ResultExt;
use command_rs::Output;

/** Runs `svc.sh status`. */
async fn collect_status(mut within_path: PathBuf) -> Result<Output,Error> {
    std::env::set_current_dir(within_path.clone()).unwrap();
    within_path.push("svc.sh");
    let i = Command::new(within_path)
        .arg("status")
        .output(kiruna::Priority::UserWaiting).await.context(ServiceOutputSnafu)?;
    Ok(i)
}
/**Determines if we need to install based on a given output. */
fn needs_install(output: &Output) -> bool {
    std::str::from_utf8(output.stdout.as_slice()).unwrap().contains("not installed")
}

/**Determines if we need to start based on a given output */
fn needs_start(output: &Output) -> bool {
    needs_install(output) || std::str::from_utf8(output.stdout.as_slice()).unwrap().contains("Stopped")
}

/**Install and/or start the service as needed */
pub async fn install_start_as_needed(within_path: PathBuf, priority: kiruna::Priority) -> Result<(),Error> {
    let status = collect_status(within_path.clone()).await?;
    let mut svc_exec = within_path;
    svc_exec.push("svc.sh");

    if needs_install(&status) {
        println!("installing");
        Command::new(svc_exec.clone()).arg("install").status(priority).await.context(ServiceStatusSnafu)?;
    }
    if needs_start(&status) {
        Command::new(svc_exec).arg("start").status(priority).await.context(ServiceStatusSnafu)?;
    }
    Ok(())
}