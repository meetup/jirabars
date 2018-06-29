#[macro_use]
extern crate cpython;
#[macro_use]
extern crate lando;
extern crate hubcaps;
#[macro_use]
extern crate serde_derive;

// extends http::Request type with api gateway info
use lando::RequestExt;

#[derive(Deserialize, Debug)]
struct Payload {
    action: String,
    number: usize,
    pull_request: PullRequest
}

#[derive(Deserialize, Debug)]
struct PullRequest {
    html_url: String,
    title: String,
    state: String,
    body: Option<String>,
    head: Ref
}

#[derive(Deserialize, Debug)]
struct Ref {
    #[serde(rename="ref")]
    branch: String
}

gateway!(|request, _| {
    if let Ok(payload) = request.payload::<Payload>() {
        println!("{:?}", payload);
    }
    
    Ok(lando::Response::new(()))
});