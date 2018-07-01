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
#[macro_use]
extern crate lazy_static;
extern crate regex;

// Third party
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use regex::Regex;
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

struct Issue {
    key: String,
    url: String,
    summary: String,
    description: String,
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

fn render(body: &String, issue: &Issue) -> Option<String> {
    lazy_static! {
        static ref LONG: Regex = Regex::new("\\{\\{ JIRA_INFO \\}\\}").expect("invalid regex");
    }
    lazy_static! {
        static ref SHORT: Regex =
            Regex::new("\\{\\{ JIRA_INFO_SHORT \\}\\}").expect("invalid regex");
    }
    let long = Some(body).filter(|b| LONG.is_match(b)).map(|body| {
        LONG.replace_all(
            body,
            format!(
                "[{key}]({url}) - __{summary}__

                {description}
                ",
                key = issue.key,
                url = issue.url,
                summary = issue.summary,
                description = issue.description
            ).as_str(),
        ).to_string()
    });

    Some(long.clone().unwrap_or_else(|| body.clone()))
        .filter(|b| SHORT.is_match(b))
        .map(|body| {
            SHORT
                .replace_all(
                    &body,
                    format!(
                        "[{key}]({url}) - __{summary}__",
                        key = issue.key,
                        url = issue.url,
                        summary = issue.summary
                    ).as_str(),
                )
                .to_string()
        })
        .or(long)
}

fn keys(branch: &str) -> Option<Vec<String>> {
    lazy_static! {
        static ref KEY: Regex = Regex::new("[A-Z]{2,}-[0-9]{1,}").expect("invalid regex");
    }
    KEY.captures(&branch).map(|caps| {
        caps.iter()
            .filter_map(|m| m.map(|m| m.as_str().to_string()))
            .collect::<Vec<_>>()
    })
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

#[cfg(test)]
mod tests {
    use super::{keys, render, Issue};

    #[test]
    fn extracts_keys_from_branches_that_have_them() {
        assert_eq!(keys("FO-1-bar-baz"), Some(vec!["FO-1".to_string()]))
    }

    #[test]
    fn fails_to_extracts_keys_from_branches_that_dont_have_them() {
        assert_eq!(keys("FO-bar-baz"), None)
    }

    #[test]
    fn renders_short_issue_template() {
        let rendered = render(
            &"{{ JIRA_INFO_SHORT }}".to_string(),
            &Issue {
                key: "FOO-123".into(),
                url: "https://company.atlassian.net/browse/FOO-123".into(),
                summary: "fix the thing".into(),
                description: "thing is broke".into(),
            },
        );
        assert_eq!(
            rendered,
            Some(
                "[FOO-123](https://company.atlassian.net/browse/FOO-123) - __fix the thing__"
                    .to_string()
            )
        )
    }

    #[test]
    fn renders_long_issue_template() {
        let rendered = render(
            &"{{ JIRA_INFO }}".to_string(),
            &Issue {
                key: "FOO-123".into(),
                url: "https://company.atlassian.net/browse/FOO-123".into(),
                summary: "fix the thing".into(),
                description: "thing is broke".into(),
            },
        );
        assert_eq!(
            rendered,
            Some(
                "[FOO-123](https://company.atlassian.net/browse/FOO-123) - __fix the thing__

                thing is broke
                "
                    .to_string()
            )
        )
    }
}
