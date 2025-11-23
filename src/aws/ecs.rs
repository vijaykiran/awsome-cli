use anyhow::Result;
use aws_sdk_ecs::Client as EcsClient;

#[derive(Clone)]
pub struct EcsService {
    client: EcsClient,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EcsItem {
    Header,
    Separator,
    Cluster(String),
    Service(String),
    Task(String),
    ParentDir,
}

impl EcsService {
    pub fn new(client: EcsClient) -> Self {
        Self { client }
    }

    pub async fn list_clusters(&self) -> Result<Vec<String>> {
        let resp = self.client.list_clusters().send().await?;
        let clusters = resp.cluster_arns();
        
        let mut cluster_names = Vec::new();
        for arn in clusters {
            // Extract cluster name from ARN
            // ARN format: arn:aws:ecs:region:account-id:cluster/cluster-name
            let name = arn.split('/').last().unwrap_or(arn).to_string();
            cluster_names.push(name);
        }
        
        Ok(cluster_names)
    }

    pub async fn list_services(&self, cluster: &str) -> Result<Vec<String>> {
        let resp = self.client.list_services().cluster(cluster).send().await?;
        let services = resp.service_arns();
        
        let mut service_names = Vec::new();
        for arn in services {
            let name = arn.split('/').last().unwrap_or(arn).to_string();
            service_names.push(name);
        }
        
        Ok(service_names)
    }

    pub async fn list_tasks(&self, cluster: &str) -> Result<Vec<String>> {
        let resp = self.client.list_tasks().cluster(cluster).send().await?;
        let tasks = resp.task_arns();
        
        let mut task_names = Vec::new();
        for arn in tasks {
            let name = arn.split('/').last().unwrap_or(arn).to_string();
            task_names.push(name);
        }
        
        Ok(task_names)
    }

    pub fn format_cluster_list(clusters: &[String]) -> (Vec<String>, Vec<EcsItem>) {
        if clusters.is_empty() {
            return (vec!["No ECS Clusters found".to_string()], vec![EcsItem::Header]);
        }

        let max_name_len = clusters.iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}", "Cluster Name", width = max_name_len);
        let separator = format!("{}", "-".repeat(max_name_len + 5));

        let mut items = vec![header, separator];
        let mut ecs_items = vec![EcsItem::Header, EcsItem::Separator];

        for name in clusters {
            items.push(format!("{:<width$}", name, width = max_name_len));
            ecs_items.push(EcsItem::Cluster(name.clone()));
        }
        (items, ecs_items)
    }

    pub fn format_service_list(services: &[String], cluster: &str) -> (Vec<String>, Vec<EcsItem>) {
        if services.is_empty() {
            let mut items = vec![format!("No Services found in cluster {}", cluster)];
            items.push("..".to_string());
            let ecs_items = vec![EcsItem::Header, EcsItem::ParentDir];
            return (items, ecs_items);
        }

        let max_name_len = services.iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}", "Service Name", width = max_name_len);
        let separator = format!("{}", "-".repeat(max_name_len + 5));

        let mut items = vec![header, separator];
        let mut ecs_items = vec![EcsItem::Header, EcsItem::Separator];

        items.push("..".to_string());
        ecs_items.push(EcsItem::ParentDir);

        for name in services {
            items.push(format!("{:<width$}", name, width = max_name_len));
            ecs_items.push(EcsItem::Service(name.clone()));
        }
        (items, ecs_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cluster_list() {
        let clusters = vec!["cluster1".to_string(), "cluster2".to_string()];
        let (items, ecs_items) = EcsService::format_cluster_list(&clusters);
        
        assert_eq!(items.len(), 4); // Header, Separator, 2 clusters
        assert!(items[0].contains("Cluster Name"));
        assert!(items[2].contains("cluster1"));
        
        assert!(matches!(ecs_items[2], EcsItem::Cluster(_)));
    }
}
