use serde::Serialize;
use chrono::DateTime;
use super::gitlab;

mod color {
    pub const INFO: u32 = 0x1F78D1;
    pub const ALERT: u32 = 0xFC9403;
    pub const GOOD: u32 = 0x1AAA555;
    pub const BAD: u32 = 0xDB3B21;
}

#[derive(Debug, Serialize)]
pub struct Embed {
    pub author: Author,
    pub title: String,
    pub url: String,
    pub color: u32,
    pub footer: Footer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<chrono::FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Embed {
    pub const DESCRIPTION_MAX_LENGTH: usize = 2048;
    pub const AUTHOR_MAX_LENGTH: usize = 256;
    pub const TITLE_MAX_LENGTH: usize = 256;
}

#[derive(Debug, Serialize)]
pub struct Author {
    pub name: String,
    pub icon_url: String,
}

#[derive(Debug, Serialize)]
pub struct Footer {
    pub text: String,
    pub icon_url: String,
}

impl From<gitlab::PushEvent> for Embed {
    fn from(ev: gitlab::PushEvent) -> Self {
        let title = format!(
            "{} - {count:} new commits in {branch:}",
            ev.project.name,
            count = ev.total_commits_count,
            branch = ev.git_ref.split('/').last().unwrap()
        );

        Self {
            author: Author {
                name: ev.username,
                icon_url: ev.user_avatar,
            },
            title,
            url: ev.project.web_url,
            color: color::INFO,
            footer: Footer {
                text: ev.project.path_with_namespace,
                icon_url: ev.project.avatar_url,
            },
            timestamp: None,
            description: Some(join_commit_lines(&ev.commits)),
        }
    }
}

impl From<gitlab::IssueEvent> for Embed {
    fn from(ev: gitlab::IssueEvent) -> Self {
        let issue = ev.attributes;
        let timestamp = issue.created_at.parse().unwrap();

        Self {
            author: Author {
                name: ev.user.username,
                icon_url: ev.user.avatar_url,
            },
            title: format!(
                "{} - Issue {}: #{} {}", ev.project.name, issue.state, issue.issue_id, issue.title
            ),
            url: issue.url,
            footer: Footer {
                text: ev.project.path_with_namespace,
                icon_url: ev.project.avatar_url,
            },
            timestamp: Some(timestamp),
            color: if issue.state == "closed" { color::GOOD } else { color::INFO },
            description: None,
        }
    }
}

impl From<gitlab::MergeRequestEvent> for Embed {
    fn from(ev: gitlab::MergeRequestEvent) -> Self {
        let mr = ev.attributes;
        let timestamp = mr.created_at.parse().unwrap();

        Self {
            author: Author {
                name: ev.user.username,
                icon_url: ev.user.avatar_url,
            },
            title: format!(
                "{} - Merge request {}: !{} {}", ev.project.name, mr.state, mr.issue_id, mr.title
            ),
            url: mr.url,
            footer: Footer {
                text: ev.project.path_with_namespace,
                icon_url: ev.project.avatar_url,
            },
            timestamp: Some(timestamp),
            color: match &*mr.state {
                "closed" => color::ALERT,
                "merged" => color::GOOD,
                _ => color::INFO,
            },
            description: None,
        }
    }
}

impl From<gitlab::PipelineEvent> for Embed {
    fn from(ev: gitlab::PipelineEvent) -> Self {
        let pipeline = ev.attributes;
        let timestamp = pipeline.created_at.parse().unwrap();

        Self {
            author: Author {
                name: ev.user.username,
                icon_url: ev.user.avatar_url,
            },
            title: format!(
                "{} - Pipeline for {} {} ({})",
                ev.project.name, pipeline.git_ref, pipeline.detailed_status, pipeline.id
            ),
            url: ev.commit.url,
            footer: Footer {
                text: ev.project.path_with_namespace,
                icon_url: ev.project.avatar_url,
            },
            timestamp: Some(timestamp),
            color: match &*pipeline.status {
                "success" => color::GOOD,
                "failed" => color::BAD,
                _ => color::INFO,
            },
            description: None,
        }
    }
}

fn join_commit_lines(commits: &[gitlab::Commit]) -> String {
    let mut chars = 0;

    commits.iter()
        .filter_map(|commit| {
            let line = commit_line(commit);
            if chars + line.len() + 1 > Embed::DESCRIPTION_MAX_LENGTH {
                None
            } else {
                chars += line.len() + 1;
                Some(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn commit_line(commit: &gitlab::Commit) -> String {
    format!(
        "[`{id:.8}`]({url:}) {msg:} - **{author:}**",
        id = commit.id,
        url = commit.url,
        msg = commit.message.lines().next().unwrap(),
        author = commit.author.name,
    )
}
