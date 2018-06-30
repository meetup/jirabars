#[macro_use]
extern crate cpython;
extern crate crypto;
extern crate goji;
#[macro_use]
extern crate lando;
extern crate hubcaps;
#[macro_use]
extern crate serde_derive;
extern crate envy;

// Third party
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
// extends http::Request type with api gateway info
use goji::Jira;
use hubcaps::Github;
use lando::RequestExt;

#[derive(Deserialize)]
struct Config {
    jira_host: String,
    jira_username: String,
    jira_password: String,
    github_webhook_secret: String,
}

#[derive(Deserialize, Debug)]
struct Payload {
    action: String,
    number: usize,
    pull_request: PullRequest,
}

#[derive(Deserialize, Debug)]
struct PullRequest {
    html_url: String,
    title: String,
    state: String,
    body: Option<String>,
    head: Ref,
}

#[derive(Deserialize, Debug)]
struct Ref {
    #[serde(rename = "ref")]
    branch: String,
}

fn authenticated(request: &lando::Request, secret: &String) -> bool {
    let headers = request.headers().clone();
    if let (Some(event), Some(signature)) = (
        headers.get("X-Github-Event"),
        headers.get("X-Hub-Signature"),
    ) {
        let sbytes = secret.as_bytes();
        let mut mac = Hmac::new(Sha1::new(), &sbytes);
        mac.input(&request.body());
        // constant time comparison
        mac.result() == MacResult::new(&sbytes)
    } else {
        false
    }
}

gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(payload) = request.payload::<Payload>() {
            println!("{:?}", payload);
        }
    } else {
        eprintln!("recieved unauthenticated request");
    }

    Ok(lando::Response::new(()))
});
