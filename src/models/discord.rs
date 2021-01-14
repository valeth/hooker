use serde::{Serialize, Serializer};
use chrono::DateTime;
use super::gitlab;

macro_rules! build_project_title {
    [ $project:expr, $fmt:literal, $($args:expr),*] => {{
        let text = format!(concat!("{} - ", $fmt), $project.name, $($args),*);
        Title::new(text)
    }}
}

#[derive(Debug)]
pub struct Color(u8, u8, u8);

impl Color {
    pub const INFO: Self = Self(0x1F, 0x78, 0xD1);
    pub const ALERT: Self = Self(0xFC, 0x94, 0x03);
    pub const GOOD: Self = Self(0x1A, 0xAA, 0x55);
    pub const BAD: Self = Self(0xDB, 0x3B, 0x21);
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let value: u32 = 0x00_00_00_00
            | ((self.0 as u32) << 16)
            | ((self.1 as u32) << 8)
            | (self.2 as u32);
        serializer.serialize_u32(value)
    }
}

#[derive(Debug, Serialize)]
pub struct Embed {
    pub author: Author,
    pub title: Title,
    pub url: String,
    pub color: Color,
    pub footer: Footer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<chrono::FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Embed {
    pub const DESCRIPTION_MAX_LENGTH: usize = 2048;
}

#[derive(Debug, Serialize)]
pub struct Author {
    pub name: String,
    pub icon_url: String,
}

impl Author {
    pub const MAX_LENGTH: usize = 256;

    pub fn new(mut name: String, icon_url: String) -> Self {
        name.truncate(Self::MAX_LENGTH);
        Self { name, icon_url }
    }
}

#[derive(Debug, Serialize)]
pub struct Footer {
    pub text: String,
    pub icon_url: String,
}

impl Footer {
    pub const MAX_LENGTH: usize = 2048;

    pub fn new(mut text: String, icon_url: String) -> Self {
        text.truncate(Self::MAX_LENGTH);
        Self { text: text.into(), icon_url }
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Title(String);

impl Title {
    pub const MAX_LENGTH: usize = 256;

    pub fn new(mut text: String) -> Self {
        text.truncate(Self::MAX_LENGTH);
        Self(text)
    }
}

impl From<gitlab::PushEvent> for Embed {
    fn from(ev: gitlab::PushEvent) -> Self {
        Self {
            author: Author::new(ev.username, ev.user_avatar),
            title: build_project_title!(
                &ev.project, "{} new commits in {}", ev.total_commits_count, ev.git_ref.split('/').last().unwrap()
            ),
            url: ev.project.web_url,
            color: Color::INFO,
            footer: Footer::new(ev.project.path_with_namespace, ev.project.avatar_url),
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
            author: Author::new(ev.user.username, ev.user.avatar_url),
            title: build_project_title!(
                &ev.project, "Issue {}: #{} {}", issue.state, issue.issue_id, issue.title
            ),
            url: issue.url,
            footer: Footer::new(ev.project.path_with_namespace, ev.project.avatar_url),
            timestamp: Some(timestamp),
            color: if issue.state == "closed" { Color::GOOD } else { Color::INFO },
            description: None,
        }
    }
}

impl From<gitlab::MergeRequestEvent> for Embed {
    fn from(ev: gitlab::MergeRequestEvent) -> Self {
        let mr = ev.attributes;
        let timestamp = mr.created_at.parse().unwrap();

        Self {
            author: Author::new(ev.user.username, ev.user.avatar_url),
            title: build_project_title!(
                &ev.project, "Merge request {}: !{} {}", mr.state, mr.issue_id, mr.title
            ),
            url: mr.url,
            footer: Footer::new( ev.project.path_with_namespace, ev.project.avatar_url),
            timestamp: Some(timestamp),
            color: match &*mr.state {
                "closed" => Color::ALERT,
                "merged" => Color::GOOD,
                _ => Color::INFO,
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
            author: Author::new(ev.user.username, ev.user.avatar_url),
            title: build_project_title!(
                &ev.project, "Pipeline for {} {} ({})", pipeline.git_ref, pipeline.detailed_status, pipeline.id
            ),
            url: ev.commit.url,
            footer: Footer::new(ev.project.path_with_namespace, ev.project.avatar_url),
            timestamp: Some(timestamp),
            color: match &*pipeline.status {
                "success" => Color::GOOD,
                "failed" => Color::BAD,
                _ => Color::INFO,
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
