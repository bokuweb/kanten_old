// use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use async_trait::async_trait;
use cloudwatchlogs::model::{QueryStatus, ResultField};

use super::*;

const DEFAULT_LIMIT: i32 = 10_000;

#[async_trait]
impl FilterLogClient for Client {
    async fn filter_logs(&self) -> Result<FilterOutput> {
        let res = self
            .client
            .filter_log_events()
            .log_group_name("/aws/lambda/test".to_owned())
            .log_stream_name_prefix("/".to_owned())
            .start_time(1618773069000)
            .end_time(1638773069000)
            .limit(10000)
            .send()
            .await?;
        log::debug!("start query response is {:?}", res);
        todo!();
    }
}

#[async_trait]
impl QueryClient for Client {
    async fn start_default_query<'a>(&self, input: StartQueryInput) -> Result<QueryId> {
        log::trace!("start query");
        // The list of log groups to be queried. You can include up to 20 log groups.
        // See also https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_StartQuery.html
        let res = self
            .client
            .start_query()
            .set_log_group_names(Some(input.groups))
            .start_time(input.start)
            .end_time(input.end)
            .query_string(format!("fields @timestamp, @message, @log | sort @timestamp desc | filter @message like /{}/", input.filter))
            .limit(DEFAULT_LIMIT)
            .send()
            .await?;
        log::trace!("start query response is {:?}", res);
        Ok(QueryId::new(res.query_id.expect("there is no query id.")))
    }

    async fn stop_query(&self, id: &QueryId) -> Result<()> {
        let res = self
            .client
            .stop_query()
            .set_query_id(Some(id.into()))
            .send()
            .await?;
        log::trace!("stop query response is {:?}", res);
        Ok(())
    }

    async fn get_default_query_results(&self, query_id: &QueryId) -> Result<SearchResult> {
        log::trace!("get query results");
        let res = self
            .client
            .get_query_results()
            .query_id(query_id)
            .send()
            .await
            .context("failed to get query result.")?;

        let items = res.results.expect("there is no results.");

        if let Some(status) = res.status {
            log::trace!("response status is {:?}", &status);
            let items = items
                .into_iter()
                .map(|item| {
                    let mut message = String::default();
                    let mut timestamp = String::default();
                    for ResultField { value, field, .. } in item {
                        let field = field.unwrap();
                        match field.as_str() {
                            "@timestamp" => timestamp = value.unwrap(),
                            "@message" => message = value.unwrap(),
                            _ => {}
                        }
                    }
                    SearchResultItem { timestamp, message }
                })
                .collect();

            // Complete
            if status == QueryStatus::Complete {
                return Ok(SearchResult::Complete(items));
            }

            // Running
            if status == QueryStatus::Running && items.len() >= DEFAULT_LIMIT as usize {
                return Ok(SearchResult::Complete(items));
            }
            // TODO: error handling
        }
        Ok(SearchResult::Running(query_id.clone()))
    }
}
