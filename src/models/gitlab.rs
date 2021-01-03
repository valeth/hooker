use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PipelineAttributes {
    pub id: u64,
    pub action: String,
    pub status: String,
    pub detailed_status: String,
    pub created_at: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueAttributes {
    #[serde(alias = "iid")]
    pub id: u64,
    pub action: String,
    pub state: String,
    pub title: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(alias = "username")]
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub id: String,
    pub url: String,
    pub message: String,
    pub author: User,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub web_url: String,
    pub path_with_namespace: String,
    pub avatar_url: String,
}


#[derive(Debug, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub git_ref: String,
    #[serde(rename = "user_username")]
    pub username: String,
    pub user_avatar: String,
    pub project: Project,
    pub commits: Vec<Commit>,
    pub total_commits_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct IssueEvent {
    pub user: User,
    pub project: Project,
    #[serde(rename = "object_attributes")]
    pub attributes: IssueAttributes,
}

#[derive(Debug, Deserialize)]
pub struct MergeRequestEvent {
    pub user: User,
    pub project: Project,
    #[serde(rename = "object_attributes")]
    pub attributes: IssueAttributes,
}

#[derive(Debug, Deserialize)]
pub struct PipelineEvent {
    pub user: User,
    pub project: Project,
    pub commit: Commit,
    #[serde(rename = "object_attributes")]
    pub attributes: PipelineAttributes,
}
