use anyhow::Result;
use aws_sdk_ec2::Client as Ec2Client;

#[derive(Clone)]
pub struct Ec2Service {
    client: Ec2Client,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ec2Item {
    Header,
    Separator,
    Instance(String),
}

impl Ec2Service {
    pub fn new(client: Ec2Client) -> Self {
        Self { client }
    }

    pub async fn list_instances(&self) -> Result<Vec<(String, String, String, String, String)>> {
        let resp = self.client.describe_instances().send().await?;

        let mut instances = Vec::new();
        for reservation in resp.reservations() {
            for instance in reservation.instances() {
                let id = instance.instance_id().unwrap_or("unknown").to_string();
                
                let name = instance.tags()
                    .iter()
                    .find(|t| t.key() == Some("Name"))
                    .and_then(|t| t.value())
                    .unwrap_or("-")
                    .to_string();

                let state = instance.state()
                    .and_then(|s| s.name())
                    .map(|n| format!("{:?}", n))
                    .unwrap_or_else(|| "unknown".to_string());
                
                let instance_type = instance.instance_type()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "unknown".to_string());
                
                let public_ip = instance.public_ip_address()
                    .unwrap_or("-")
                    .to_string();

                instances.push((id, name, state, instance_type, public_ip));
            }
        }

        Ok(instances)
    }

    pub fn format_instance_list(instances: &[(String, String, String, String, String)]) -> (Vec<String>, Vec<Ec2Item>) {
        if instances.is_empty() {
            return (vec!["No EC2 Instances found".to_string()], vec![Ec2Item::Header]);
        }

        // Calculate column widths
        let max_id_len = instances.iter()
            .map(|(id, _, _, _, _)| id.len())
            .max()
            .unwrap_or(10)
            .max(10);
            
        let max_name_len = instances.iter()
            .map(|(_, name, _, _, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let max_state_len = instances.iter()
            .map(|(_, _, state, _, _)| state.len())
            .max()
            .unwrap_or(10)
            .max(10);
            
        let max_type_len = instances.iter()
            .map(|(_, _, _, type_, _)| type_.len())
            .max()
            .unwrap_or(10)
            .max(10);

        let header = format!(
            "{:<width_id$}  {:<width_name$}  {:<width_state$}  {:<width_type$}  Public IP", 
            "Instance ID", "Name", "State", "Type",
            width_id = max_id_len, 
            width_name = max_name_len,
            width_state = max_state_len,
            width_type = max_type_len
        );
        
        let separator_len = max_id_len + max_name_len + max_state_len + max_type_len + 25;
        let separator = format!("{}", "-".repeat(separator_len));

        let mut items = vec![header, separator];
        let mut ec2_items = vec![Ec2Item::Header, Ec2Item::Separator];

        for (id, name, state, type_, ip) in instances {
            items.push(format!(
                "{:<width_id$}  {:<width_name$}  {:<width_state$}  {:<width_type$}  {}", 
                id, name, state, type_, ip,
                width_id = max_id_len, 
                width_name = max_name_len,
                width_state = max_state_len,
                width_type = max_type_len
            ));
            ec2_items.push(Ec2Item::Instance(id.clone()));
        }
        (items, ec2_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_instance_list() {
        // Test empty list
        let (items, ec2_items) = Ec2Service::format_instance_list(&[]);
        assert_eq!(items[0], "No EC2 Instances found");
        assert!(matches!(ec2_items[0], Ec2Item::Header));

        // Test populated list
        let instances = vec![
            ("i-1234567890abcdef0".to_string(), "web-server".to_string(), "running".to_string(), "t2.micro".to_string(), "1.2.3.4".to_string()),
            ("i-0987654321fedcba0".to_string(), "db-server".to_string(), "stopped".to_string(), "m5.large".to_string(), "-".to_string()),
        ];
        let (items, ec2_items) = Ec2Service::format_instance_list(&instances);
        
        assert_eq!(items.len(), 4); // Header, Separator, 2 instances
        assert!(items[0].contains("Instance ID"));
        assert!(items[0].contains("Name"));
        assert!(items[2].contains("i-1234567890abcdef0"));
        assert!(items[3].contains("db-server"));
        
        assert!(matches!(ec2_items[2], Ec2Item::Instance(_)));
        if let Ec2Item::Instance(id) = &ec2_items[2] {
            assert_eq!(id, "i-1234567890abcdef0");
        }
    }
}
