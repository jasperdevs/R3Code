pub const APP_NAME: &str = "R3Code";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub title: String,
    pub project_name: String,
    pub status: ThreadStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle,
    Running,
    NeedsInput,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub author: MessageAuthor,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAuthor {
    User,
    Agent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub projects: Vec<ProjectSummary>,
    pub threads: Vec<ThreadSummary>,
    pub messages: Vec<ChatMessage>,
}

impl AppSnapshot {
    pub fn empty_reference_state() -> Self {
        Self {
            projects: Vec::new(),
            threads: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn mock_reference_state() -> Self {
        Self {
            projects: vec![ProjectSummary {
                name: "r3code".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
            }],
            threads: vec![
                ThreadSummary {
                    title: "Port T3Code UI shell".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Running,
                },
                ThreadSummary {
                    title: "Capture visual references".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Idle,
                },
            ],
            messages: vec![
                ChatMessage {
                    author: MessageAuthor::User,
                    body: "Make the Rust port match the original UI exactly.".to_string(),
                },
                ChatMessage {
                    author: MessageAuthor::Agent,
                    body: "Building a static GPUI shell first, then replacing mock data with Rust state.".to_string(),
                },
            ],
        }
    }
}
