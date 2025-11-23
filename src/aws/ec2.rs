use anyhow::Result;
use aws_sdk_ec2::Client as Ec2Client;

#[derive(Clone)]
pub struct Ec2Service {
    client: Ec2Client,
}

impl Ec2Service {
    pub fn new(client: Ec2Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> Result<Vec<String>> {
        let resp = self.client.describe_instances().send().await?;

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
}
