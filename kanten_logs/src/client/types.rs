use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, PartialEq, Clone)]
pub struct QueryId(String);

impl QueryId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&QueryId> for String {
    fn from(id: &QueryId) -> String {
        id.0.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GetGroupsOutput {
    pub items: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GetStreamsOutput {
    pub items: Vec<String>,
}

#[async_trait]
pub trait GroupsClient {
    async fn get_group_names(&self) -> Result<GetGroupsOutput>;
    async fn get_streams(&self, group_name: &str, since: usize) -> Result<GetStreamsOutput>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct SearchResultItem {
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SearchResult {
    Running(QueryId),
    Complete(Vec<SearchResultItem>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StartQueryInput {
    pub start: i64,
    pub end: i64,
    pub filter: String,
    pub groups: Vec<String>,
}

#[async_trait]
pub trait QueryClient {
    async fn start_default_query<'a>(&self, input: StartQueryInput) -> Result<QueryId>;
    async fn get_default_query_results(&self, query_id: &QueryId) -> Result<SearchResult>;
    async fn stop_query(&self, id: &QueryId) -> Result<()>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct FilterOutputItem {
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FilterOutput {
    pub items: Vec<FilterOutputItem>,
}

#[async_trait]
pub trait FilterLogClient {
    async fn filter_logs(&self) -> Result<FilterOutput>;
}
