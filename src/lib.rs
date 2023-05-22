use discord_flows::{get_client, model::Message};
use dotenv::dotenv;
use github_flows::{
    listen_to_event, octocrab::models::events::payload::IssuesEventAction, EventPayload,
    GithubLogin::Default,
};
use slack_flows::{create_text_message_in_channel, SlackLogin::SlackDefault};
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
    let discord_token = env::var("discord_token").expect("Expected a bot token.");

    let discord_server = env::var("discord_server").unwrap_or("Vivian Hu's server".to_string());
    let discord_channel = env::var("discord_channel").unwrap_or("general".to_string());

    let slack_workspace = env::var("slack_workspace").unwrap_or("secondstate".to_string());
    let slack_channel = env::var("slack_channel").unwrap_or("github-status".to_string());

    if let Some(EventPayload::IssuesEvent(e)) = payload {
        if e.action == IssuesEventAction::Closed || e.action == IssuesEventAction::Edited {
            return;
        }
        let issue = e.issue;
        let issue_title = issue.title;
        let issue_url = issue.html_url;
        let user = issue.user.login;
        let labels = issue.labels;

        for label in labels {
            match label.name {
                "good first issue" => {
                    let body = format!(
                        "good first issue: {issue_title}\n{comment_content}\n{comment_url}"
                    );
                    create_text_message_in_channel(&discord_server, &discord_channel, body, None);
                    // _ = client
                    //     .send_message(
                    //         msg.channel_id.into(),
                    //         &serde_json::json!({
                    //             "content": body,
                    //         }),
                    //     )
                    //     .await;
                }
                "bug" => {
                    let   body = format!(
                            "A bug issue has been submitted: {issue_title}\n{comment_content}\n{comment_url}"
                        );
                    send_message_to_channel(&slack_workspace, &slack_channel, body);
                }
                _ => {}
            }
        }
    }
}
