// Third party
use reqwest::Client;

#[derive(Deserialize, Debug)]
pub struct Payload {
  pub action: String,
  pub number: usize,
  pub pull_request: PullRequest,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
  pub url: String,
  pub html_url: String,
  pub title: String,
  pub state: String,
  pub body: Option<String>,
  pub head: Ref,
}

#[derive(Deserialize, Debug)]
pub struct Ref {
  #[serde(rename = "ref")]
  pub branch: String,
}

pub fn patch(token: &String, url: &String, body: &String) -> Option<()> {
  Client::new()
    .expect("failed to create client")
    .patch(url)
    .expect("failed to parse url")
    .basic_auth("", Some(token.clone()))
    .json(&json!({ "body": body }))
    .expect("failed to encode body")
    .send()
    .map(|_| ())
    .ok()
}
