use anyhow::Result;
use aws_sdk_cloudwatch::Client as CloudwatchClient;

#[derive(Clone)]
pub struct CloudwatchService {
    client: CloudwatchClient,
}

impl CloudwatchService {
    pub fn new(client: CloudwatchClient) -> Self {
        Self { client }
    }

    pub async fn list_alarms(&self) -> Result<Vec<String>> {
        let resp = self.client.describe_alarms().send().await?;

        let alarms: Vec<String> = resp
            .metric_alarms()
            .iter()
            .filter_map(|a| a.alarm_name().map(String::from))
            .collect();

        Ok(alarms)
    }
}
