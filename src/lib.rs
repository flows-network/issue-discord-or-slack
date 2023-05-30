use discord_flows::http::HttpBuilder;
use dotenv::dotenv;
use github_flows::{
    listen_to_event, octocrab::models::events::payload::IssuesEventAction, EventPayload,
    GithubLogin::Default,
};
use serde_json::json;
use slack_flows::send_message_to_channel;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();

    let github_owner = env::var("github_owner").unwrap_or("alabulei1".to_string());
    let github_repo = env::var("github_repo").unwrap_or("a-test".to_string());

    listen_to_event(
        &Default,
        &github_owner,
        &github_repo,
        vec!["issues"],
        handler,
    )
    .await;
}

async fn handler(payload: EventPayload) {
    let client = HttpBuilder::new("DEFAULT_BOT").build();
    let discord_server = env::var("discord_server").unwrap_or("Vivian Hu's server".to_string());

    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    if let EventPayload::IssuesEvent(e) = payload {
        if e.action == IssuesEventAction::Closed {
            return;
        }
        let issue = e.issue;
        let issue_title = issue.title;
        let issue_url = issue.html_url;
        let user = issue.user.login;
        let labels = issue.labels;

        'outer: for label in labels {
            match label.name.as_str() {
                "good first issue" => {
                    let body =
                        format!("{user} submitted good first issue: {issue_title}\n{issue_url}");
                    match env::var("discord_channel_id") {
                        Ok(val) => {
                            if val.len() == 19 {   
                                //1091003237827608650
                                let channel_id = val.parse::<u64>().unwrap();
                                _ = client
                                    .send_message(
                                        channel_id,
                                        &serde_json::json!({
                                            "content": body,
                                        }),
                                    )
                                    .await;
                                continue 'outer;
                            }
                        }
                        Err(_e) => {}
                    }
                    send_message_to_channel(&slack_workspace, &slack_channel, "you've failed to set a discord_channel_id or set it incorrectly on flows server, so bot failed to noitify you on a good first issue on discord, you're advised to correct this as appropriate".to_string());
                }
                "bug" => {
                    let body = format!("{user} submitted bug issue: {issue_title}\n{issue_url}");
                    send_message_to_channel(&slack_workspace, &slack_channel, body);
                }
                _ => {}
            }
        }
    }
}
