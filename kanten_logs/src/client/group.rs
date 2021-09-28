use anyhow::Result;
use async_trait::async_trait;

use super::*;

#[async_trait]
impl GroupsClient for Client {
    async fn get_group_names(&self) -> Result<GetGroupsOutput> {
        log::debug!("get group names");
        let mut items: Vec<String> = vec![];
        let mut next_token: Option<String> = None;
        loop {
            let res = self
                .client
                .describe_log_groups()
                .set_next_token(next_token.clone())
                .send()
                .await;
            if let Err(cloudwatchlogs::SdkError::ServiceError { ref err, .. }) = res {
                if let Some("ThrottlingException") = err.code() {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
            }

            let res = res.unwrap();

            items.extend(
                res.log_groups
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|g| g.log_group_name),
            );

            if res.next_token.is_none() {
                return Ok(GetGroupsOutput { items });
            }

            next_token = res.next_token;

            log::debug!("nextToken is {:?}", &next_token);
        }
    }

    async fn get_streams(&self, group_name: &str, since: usize) -> Result<GetStreamsOutput> {
        let mut items: Vec<String> = vec![];
        let mut next_token = None;
        loop {
            let res = self
                .client
                .describe_log_streams()
                .log_group_name(group_name)
                .order_by(cloudwatchlogs::model::OrderBy::LastEventTime)
                .descending(true)
                .set_next_token(next_token.clone())
                .limit(1)
                .send()
                .await;

            log::debug!("describe log streams response is {:?}", &res);

            if let Err(cloudwatchlogs::SdkError::ServiceError { ref err, .. }) = res {
                if let Some("ThrottlingException") = err.code() {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
            }

            let res = res.unwrap();

            next_token = res.next_token;

            log::debug!("nextToken is {:?}", &next_token);

            if let Some(stream) = res.log_streams {
                for s in stream {
                    if s.last_ingestion_time
                        .expect("should last ingestion_time exists.")
                        < since as i64
                    {
                        return Ok(GetStreamsOutput { items });
                    }
                    items.push(s.log_stream_name.expect("should stream name exists."))
                }
                if next_token.is_none() {
                    return Ok(GetStreamsOutput { items });
                }
            } else {
                return Ok(GetStreamsOutput { items });
            };
        }
    }
}
