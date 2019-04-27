#[macro_use]
extern crate cpython;
extern crate crypto;
extern crate goji;
#[macro_use]
extern crate lando;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate envy;
#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate regex;

// Third party
use crypto::{
    hmac::Hmac,
    mac::{Mac, MacResult},
    sha1::Sha1,
};
use hex::FromHex;
use lando::RequestExt;

mod github;
mod jira;
mod metric;

#[derive(Deserialize)]
struct Config {
    jira_host: String,
    jira_username: String,
    jira_password: String,
    github_token: String,
    github_webhook_secret: String,
}

fn authenticated(request: &lando::Request, secret: &String) -> bool {
    request
        .headers()
        .get("X-Hub-Signature")
        .iter()
        .filter_map(|value| {
            // strip off `sha1=` and get hex bytes
            Vec::from_hex(value.to_str().expect("invalid header")[5..].as_bytes()).ok()
        })
        .any(|signature| {
            let mut mac = Hmac::new(Sha1::new(), &secret.as_bytes());
            mac.input(&request.body());
            mac.result() == MacResult::new(&signature)
        })
}

fn incr_auth_fail() -> Option<String> {
    metric::incr("jirabars.fail", vec!["reason:invalid_authentication".into()])
}

fn incr_patched(repo: &String, branch: &String) -> Option<String> {
    metric::incr(
        "jirabars.patched",
        vec![format!("repo:{}", repo), format!("branch:{}", branch)],
    )
}

fn incr_patch_fail(reason: &String, repo: &String, branch: &String) -> Option<String> {
    metric::incr(
        "jirabars.fail",
        vec![
            format!("repo:{}", repo),
            format!("branch:{}", branch),
            format!("reason:{}", reason),
        ],
    )
}

#[cfg_attr(tarpaulin, skip)]
gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(Some(payload)) = request.payload::<github::Payload>() {
            if payload.updatable() {
                if let Some((_, updated)) = jira::body(
                    config.jira_host,
                    config.jira_username,
                    config.jira_password,
                    &payload.pull_request.head.branch,
                    &payload.pull_request.body.clone().unwrap_or_default(),
                ) {
                    match github::patch(&config.github_token, &payload.pull_url(), &updated) {
                        Ok(_) => {
                            for metric in incr_patched(
                                &payload.pull_request.head.repo.full_name,
                                &payload.pull_request.head.branch,
                            ) {
                                println!("{}", metric);
                            }
                        }
                        Err(e) => {
                            for metric in incr_patch_fail(
                                &e.to_string(),
                                &payload.pull_request.head.repo.full_name,
                                &payload.pull_request.head.branch,
                            ) {
                                println!("{}", metric);
                            }
                        }
                    }
                }
            }
        }
    } else {
        incr_auth_fail();
    }

    Ok(lando::Response::new(()))
});

#[cfg(test)]
mod tests {
    use super::{authenticated, lando};

    #[test]
    fn missing_header_is_authenticated() {
        assert!(!authenticated(&lando::Request::new("{}".into()), &"secret".to_string()))
    }
}
