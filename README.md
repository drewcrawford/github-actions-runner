# github-actions-runner

Rust-language install script for [github actions runner](https://github.com/actions/runner).

Useful in wrangling self-hosted runners.  Provides a rich API in the Rust typesystem for configuration.

# Status

Works on macOS 12+.

# Example
```rust
use github_actions_runner::{ensure_actions_runner,Repo,PersonalAuthenticationToken};
async fn example() -> Result<(),Box<dyn std::error::Error>> {
    let repo = Repo::new("drewcrawford".to_owned(),"github-actions-runner".to_owned());
    let token = PersonalAuthenticationToken::new("invalid".to_owned());
    let f = ensure_actions_runner(repo,token, kiruna::Priority::Testing);
    f.await?;
    Ok(())
}
 ```