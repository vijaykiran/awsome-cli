use anyhow::Result;
use aws_sdk_mwaa::Client as MwaaClient;

#[derive(Clone)]
pub struct MwaaService {
    client: MwaaClient,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MwaaItem {
    Header,
    Separator,
    Environment(String),
}

impl MwaaService {
    pub fn new(client: MwaaClient) -> Self {
        Self { client }
    }

    pub async fn list_environments(&self) -> Result<Vec<String>> {
        let resp = self.client.list_environments().send().await?;
        Ok(resp.environments)
    }

    pub async fn get_environment(&self, name: &str) -> Result<aws_sdk_mwaa::types::Environment> {
        let resp = self.client.get_environment().name(name).send().await?;
        resp.environment
            .ok_or_else(|| anyhow::anyhow!("Environment not found"))
    }

    pub fn format_environment_list(envs: &[String]) -> (Vec<String>, Vec<MwaaItem>) {
        if envs.is_empty() {
            return (
                vec!["No MWAA Environments found".to_string()],
                vec![MwaaItem::Header],
            );
        }

        let max_name_len = envs
            .iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}", "Environment Name", width = max_name_len);
        let separator = "-".repeat(max_name_len + 5).to_string();

        let mut items = vec![header, separator];
        let mut mwaa_items = vec![MwaaItem::Header, MwaaItem::Separator];

        for name in envs {
            items.push(format!("{:<width$}", name, width = max_name_len));
            mwaa_items.push(MwaaItem::Environment(name.clone()));
        }
        (items, mwaa_items)
    }

    pub fn get_environment_details_pairs(
        env: &aws_sdk_mwaa::types::Environment,
    ) -> Vec<(String, String)> {
        let created_at = env.created_at().map(|t| t.to_string());
        vec![
            (
                "Name".to_string(),
                env.name().unwrap_or("unknown").to_string(),
            ),
            (
                "ARN".to_string(),
                env.arn().unwrap_or("unknown").to_string(),
            ),
            (
                "Status".to_string(),
                env.status()
                    .map(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ),
            (
                "Airflow Version".to_string(),
                env.airflow_version().unwrap_or("unknown").to_string(),
            ),
            (
                "Execution Role".to_string(),
                env.execution_role_arn().unwrap_or("unknown").to_string(),
            ),
            (
                "Service Role".to_string(),
                env.service_role_arn().unwrap_or("unknown").to_string(),
            ),
            (
                "KMS Key".to_string(),
                env.kms_key().unwrap_or("None").to_string(),
            ),
            (
                "Webserver URL".to_string(),
                env.webserver_url().unwrap_or("unknown").to_string(),
            ),
            (
                "Created At".to_string(),
                created_at.as_deref().unwrap_or("unknown").to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_environment_list() {
        let envs = vec!["env1".to_string(), "env2".to_string()];
        let (items, mwaa_items) = MwaaService::format_environment_list(&envs);

        assert_eq!(items.len(), 4); // Header, Separator, 2 envs
        assert!(items[0].contains("Environment Name"));
        assert!(items[2].contains("env1"));

        assert!(matches!(mwaa_items[2], MwaaItem::Environment(_)));
    }

    #[test]
    fn test_get_environment_details_pairs() {
        let env = aws_sdk_mwaa::types::Environment::builder()
            .name("test-env")
            .arn("arn:aws:mwaa:us-east-1:123456789012:environment/test-env")
            .status(aws_sdk_mwaa::types::EnvironmentStatus::Available)
            .build();

        let details = MwaaService::get_environment_details_pairs(&env);

        assert_eq!(details.len(), 9);
        assert_eq!(details[0], ("Name".to_string(), "test-env".to_string()));
        assert_eq!(details[2], ("Status".to_string(), "AVAILABLE".to_string()));
    }
}
