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
    #[serde(rename = "iid")]
    pub issue_id: u64,
    pub action: String,
    pub state: String,
    pub title: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub username: String,
    pub avatar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub id: String,
    pub url: String,
    pub message: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json as json;

    #[test]
    fn deserialize_push_event() {
        let event = include_str!("../../tests/data/push_event.json");
        json::from_str::<PushEvent>(&event).unwrap();
    }

    #[test]
    fn deserialize_issue_opened_event() {
        let event = include_str!("../../tests/data/issue_opened_event.json");
        json::from_str::<IssueEvent>(&event).unwrap();
    }

    #[test]
    fn deserialize_issue_closed_event() {
        let event = include_str!("../../tests/data/issue_closed_event.json");
        json::from_str::<IssueEvent>(&event).unwrap();
    }

    #[test]
    fn deserialize_mr_opened_event() {
        let event = include_str!("../../tests/data/mr_opened_event.json");
        json::from_str::<MergeRequestEvent>(&event).unwrap();
    }

    #[test]
    fn deserialize_mr_merged_event() {
        let event = include_str!("../../tests/data/mr_merged_event.json");
        json::from_str::<MergeRequestEvent>(&event).unwrap();
    }

    #[test]
    fn deserialize_mr_closed_event() {
        let event = include_str!("../../tests/data/mr_closed_event.json");
        json::from_str::<MergeRequestEvent>(&event).unwrap();
    }
}
