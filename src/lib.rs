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
extern crate regex;

// Third party
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
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
        .any(|signature| {
            // strip off `sha1=`
            let signature_value = &signature.to_str().unwrap()[5..signature.len()];
            let mut mac = Hmac::new(Sha1::new(), &secret.as_bytes());
            mac.input(&request.body());
            mac.result() == MacResult::new(&signature_value.as_bytes())
        })
}

#[cfg_attr(tarpaulin, skip)]
gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(Some(payload)) = request.payload::<github::Payload>() {
            println!("{:?}", payload);
            let updated = jira::body(
                config.jira_host,
                config.jira_username,
                config.jira_password,
                &payload.pull_request.head.branch,
                &payload.pull_request.body.unwrap_or_default(),
            );
            println!("updated {:?}", updated);
            for patched in updated {
                //github::patch(&config.github_token, &payload.pull_request.url, &patched);
            }
        }
    } else {
        eprintln!("recieved unauthenticated request");
    }

    Ok(lando::Response::new(()))
});
