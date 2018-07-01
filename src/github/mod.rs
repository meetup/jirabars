#[derive(Deserialize, Debug)]
pub struct Payload {
  pub action: String,
  pub number: usize,
  pub pull_request: PullRequest,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
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
