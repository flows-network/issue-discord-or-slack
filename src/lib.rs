use discord_flows::http::HttpBuilder;
use dotenv::dotenv;
use github_flows::{
    listen_to_event, octocrab::models::events::payload::IssuesEventAction, EventPayload,
    GithubLogin::Default,
};
use serde_json;
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
        let labels_str = issue
            .labels
            .into_iter()
            .map(|lab| lab.name)
            .collect::<Vec<String>>()
            .join(",");

        match (
            labels_str.contains("good first issue"),
            labels_str.contains("bug"),
        ) {
            (true, _) => {
                let body = format!("{user} submitted good first issue: {issue_title}\n{issue_url}");
                // follow this to get discord_channel_id, a 19-digit number like 1091003237827608650
                // Open Discord and go to the server where the channel is located.
                // Make sure you have the necessary permissions to view channel details.
                // Find the channel in the server's channel list on the left-hand side.
                // Right-click on the channel name and select "Copy ID" from the context menu.
                match env::var("discord_channel_id") {
                    Ok(val) => {
                        if val.len() == 19 {
                            let channel_id = val.parse::<u64>().unwrap();
                            _ = client
                                .send_message(
                                    channel_id,
                                    &serde_json::json!({
                                        "content": body,
                                    }),
                                )
                                .await;
                        } else {
                            send_message_to_channel(&slack_workspace, &slack_channel, format!("Please check if the discord_channel_id: {val} is incorrect, if true, please go to flows network to modify."));
                        }
                    }
                    Err(_e) => {
                        send_message_to_channel(&slack_workspace, &slack_channel, "You've probably forgot to set a discord_channel_id on flows server, so the bot failed to notify you on a good first issue on discord, please go to flows network to modify.".to_string());
                    }
                }
            }

            (_, true) => {
                let body = format!("{user} submitted bug issue: {issue_title}\n{issue_url}");
                send_message_to_channel(&slack_workspace, &slack_channel, body);
            }
            _ => {}
        }
    }
}
