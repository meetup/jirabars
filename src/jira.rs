// Third party
use goji::{Credentials, Jira};
use regex::Regex;

struct Issue {
  key: String,
  url: String,
  summary: String,
  description: String,
}

pub fn body(
  host: String,
  user: String,
  pass: String,
  branch: &String,
  body: &String,
) -> Option<String> {
  keys(branch).and_then(|extracted| {
    let jira =
      Jira::new(host.clone(), Credentials::Basic(user, pass)).expect("failed to initialize client");
    match jira.search().iter(
      format!("issuekey in ({keys})", keys = extracted.join(",")),
      &Default::default(),
    ) {
      Ok(issues) => issues.fold(None as Option<String>, |_, issue| {
        render(
          body,
          &Issue {
            key: issue.key.clone(),
            url: format!("{host}/browse/{key}", host = host.as_str(), key = issue.key),
            summary: issue.summary().unwrap_or_else(|| String::new()),
            description: issue.description().unwrap_or_else(|| String::new()),
          },
        )
      }),
      _ => None,
    }
  })
}

fn render(body: &String, issue: &Issue) -> Option<String> {
  lazy_static! {
    static ref LONG: Regex = Regex::new("\\{\\{ JIRA_INFO \\}\\}").expect("invalid regex");
  }
  lazy_static! {
    static ref SHORT: Regex = Regex::new("\\{\\{ JIRA_INFO_SHORT \\}\\}").expect("invalid regex");
  }
  let long = Some(body).filter(|b| LONG.is_match(b)).map(|body| {
    LONG
      .replace_all(
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
      )
      .to_string()
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
    caps
      .iter()
      .filter_map(|m| m.map(|m| m.as_str().to_string()))
      .collect::<Vec<_>>()
  })
}

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
        "[FOO-123](https://company.atlassian.net/browse/FOO-123) - __fix the thing__".to_string()
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
