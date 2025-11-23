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
            let name = arn.split('/').next_back().unwrap_or(arn).to_string();
            cluster_names.push(name);
        }

        Ok(cluster_names)
    }

    pub async fn list_services(&self, cluster: &str) -> Result<Vec<String>> {
        let resp = self.client.list_services().cluster(cluster).send().await?;
        let services = resp.service_arns();

        let mut service_names = Vec::new();
        for arn in services {
            let name = arn.split('/').next_back().unwrap_or(arn).to_string();
            service_names.push(name);
        }

        Ok(service_names)
    }

    pub async fn list_tasks(
        &self,
        cluster: &str,
        service_name: Option<&str>,
    ) -> Result<Vec<(String, String, String, String, String)>> {
        let mut req = self.client.list_tasks().cluster(cluster);
        if let Some(service) = service_name {
            req = req.service_name(service);
        }
        let resp = req.send().await?;
        let task_arns = resp.task_arns();

        if task_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe tasks to get details (limit to 100 for now as per API limit)
        let tasks_to_describe: Vec<String> = task_arns.iter().take(100).cloned().collect();

        let resp = self
            .client
            .describe_tasks()
            .cluster(cluster)
            .set_tasks(Some(tasks_to_describe))
            .send()
            .await?;

        let mut tasks = Vec::new();
        for task in resp.tasks() {
            let id = task
                .task_arn()
                .unwrap_or("")
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string();
            let def = task
                .task_definition_arn()
                .unwrap_or("")
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string();
            let last_status = task.last_status().unwrap_or("unknown").to_string();
            let desired_status = task.desired_status().unwrap_or("unknown").to_string();
            let started_at = task
                .started_at()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "pending".to_string());

            tasks.push((id, def, last_status, desired_status, started_at));
        }

        Ok(tasks)
    }

    pub fn format_task_list(
        tasks: &[(String, String, String, String, String)],
        cluster: &str,
        service: Option<&str>,
    ) -> (Vec<String>, Vec<EcsItem>) {
        let context = if let Some(svc) = service {
            format!("Service {}", svc)
        } else {
            format!("Cluster {}", cluster)
        };

        if tasks.is_empty() {
            let mut items = vec![format!("No Tasks found in {}", context)];
            items.push("..".to_string());
            let ecs_items = vec![EcsItem::Header, EcsItem::ParentDir];
            return (items, ecs_items);
        }

        let max_id_len = tasks
            .iter()
            .map(|(id, _, _, _, _)| id.len())
            .max()
            .unwrap_or(32)
            .max(32);
        let max_def_len = tasks
            .iter()
            .map(|(_, def, _, _, _)| def.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!(
            "{:<width_id$}  {:<width_def$}  {:<12}  {:<12}  {}",
            "Task ID",
            "Definition",
            "Last Status",
            "Desired",
            "Started At",
            width_id = max_id_len,
            width_def = max_def_len
        );
        let separator = "-".repeat(header.len()).to_string();

        let mut items = vec![header, separator];
        let mut ecs_items = vec![EcsItem::Header, EcsItem::Separator];

        items.push("..".to_string());
        ecs_items.push(EcsItem::ParentDir);

        for (id, def, last, desired, started) in tasks {
            items.push(format!(
                "{:<width_id$}  {:<width_def$}  {:<12}  {:<12}  {}",
                id,
                def,
                last,
                desired,
                started,
                width_id = max_id_len,
                width_def = max_def_len
            ));
            ecs_items.push(EcsItem::Task(id.clone()));
        }
        (items, ecs_items)
    }

    pub fn format_cluster_list(clusters: &[String]) -> (Vec<String>, Vec<EcsItem>) {
        if clusters.is_empty() {
            return (
                vec!["No ECS Clusters found".to_string()],
                vec![EcsItem::Header],
            );
        }

        let max_name_len = clusters
            .iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}", "Cluster Name", width = max_name_len);
        let separator = "-".repeat(max_name_len + 5).to_string();

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

        let max_name_len = services
            .iter()
            .map(|name| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}", "Service Name", width = max_name_len);
        let separator = "-".repeat(max_name_len + 5).to_string();

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

    #[test]
    fn test_format_task_list() {
        let tasks = vec![
            (
                "task1".to_string(),
                "def1".to_string(),
                "RUNNING".to_string(),
                "RUNNING".to_string(),
                "2023-01-01".to_string(),
            ),
            (
                "task2".to_string(),
                "def2".to_string(),
                "STOPPED".to_string(),
                "STOPPED".to_string(),
                "2023-01-02".to_string(),
            ),
        ];
        let (items, ecs_items) = EcsService::format_task_list(&tasks, "cluster1", Some("service1"));

        assert_eq!(items.len(), 5); // Header, Separator, ParentDir, 2 tasks
        assert!(items[0].contains("Task ID"));
        assert!(items[3].contains("task1"));
        assert!(items[3].contains("RUNNING"));

        assert!(matches!(ecs_items[3], EcsItem::Task(_)));
    }
}
