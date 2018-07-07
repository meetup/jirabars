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
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use lando::RequestExt;

mod github;
mod jira;

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

#[cfg_attr(tarpaulin, skip)]
gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(Some(payload)) = request.payload::<github::Payload>() {
            println!("{:?}", payload);
            if let Some(updated) = jira::body(
                config.jira_host,
                config.jira_username,
                config.jira_password,
                &payload.pull_request.head.branch,
                &payload.pull_request.body.unwrap_or_default(),
            ) {
                println!("updated {:?}", updated);
            }
        }
    } else {
        eprintln!("recieved unauthenticated request");
    }

    Ok(lando::Response::new(()))
});

#[cfg(test)]
mod tests {
    use super::authenticated;
    use super::lando;

    #[test]
    fn missing_header_is_authenticated() {
        assert!(!authenticated(
            &lando::Request::new("{}".into()),
            &"secret".to_string()
        ))
    }
}
