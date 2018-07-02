# jirabars [![Build Status](https://travis-ci.org/meetup/jirabars.svg?branch=master)](https://travis-ci.org/meetup/jirabars) [![Coverage Status](https://coveralls.io/repos/github/meetup/jirabars/badge.svg?branch=master)](https://coveralls.io/github/meetup/jirabars?branch=master)

> like handlebars but for jira applied to github pr bodies

## 🤔 about

This application is a github webhook handler for pull request events that applies
jira informaiton to pull request body placeholders

## 👩‍🏭 development

This is a [rustlang](https://www.rust-lang.org/en-US/) application.
Go grab yourself a copy with [rustup](https://rustup.rs/).

## 🚀 deployment

This is a rust application deployed using ⚡ [serverless](https://serverless.com/) ⚡.

> 💡 To install serverless, run `make dependencies`

This lambda is configured through its environment variables.

| Name                    | Description                                       |
|-------------------------|---------------------------------------------------|
| `GITHUB_TOKEN`          | token used to update github pull request          |
| `GITHUB_WEBHOOK_SECRET` | shared secret used to authenticate requests       |
| `JIRA_HOST`             | jira installation host                            |
| `JIRA_USERNAME`         | username used to authenticate jira api requests   |
| `JIRA_PASSWORD`         | passworded used to authenticate jira api requests |

Run `AWS_PROFILE=prod make deploy` to deploy.