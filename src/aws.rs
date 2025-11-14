use anyhow::Result;
use aws_config::BehaviorVersion;

#[derive(Clone)]
pub struct AwsClient {
    ec2_client: aws_sdk_ec2::Client,
    s3_client: aws_sdk_s3::Client,
    iam_client: aws_sdk_iam::Client,
    cloudwatch_client: aws_sdk_cloudwatch::Client,
}

impl AwsClient {
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

        Ok(Self {
            ec2_client: aws_sdk_ec2::Client::new(&config),
            s3_client: aws_sdk_s3::Client::new(&config),
            iam_client: aws_sdk_iam::Client::new(&config),
            cloudwatch_client: aws_sdk_cloudwatch::Client::new(&config),
        })
    }

    pub async fn list_ec2_instances(&self) -> Result<Vec<String>> {
        let resp = self.ec2_client.describe_instances().send().await?;

        let mut instances = Vec::new();
        for reservation in resp.reservations() {
            for instance in reservation.instances() {
                let id = instance.instance_id().unwrap_or("unknown");
                let state = instance.state()
                    .and_then(|s| s.name())
                    .map(|n| format!("{:?}", n))
                    .unwrap_or_else(|| "unknown".to_string());
                instances.push(format!("{} - {}", id, state));
            }
        }

        Ok(instances)
    }

    pub async fn list_s3_buckets(&self) -> Result<Vec<String>> {
        let resp = self.s3_client.list_buckets().send().await?;

        let buckets: Vec<String> = resp
            .buckets()
            .iter()
            .filter_map(|b| b.name().map(String::from))
            .collect();

        Ok(buckets)
    }

    pub async fn list_iam_users(&self) -> Result<Vec<String>> {
        let resp = self.iam_client.list_users().send().await?;

        let users: Vec<String> = resp
            .users()
            .iter()
            .map(|u| u.user_name().to_string())
            .collect();

        Ok(users)
    }

    pub async fn list_cloudwatch_alarms(&self) -> Result<Vec<String>> {
        let resp = self.cloudwatch_client.describe_alarms().send().await?;

        let alarms: Vec<String> = resp
            .metric_alarms()
            .iter()
            .filter_map(|a| a.alarm_name().map(String::from))
            .collect();

        Ok(alarms)
    }
}
