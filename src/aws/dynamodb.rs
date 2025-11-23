use anyhow::Result;
use aws_sdk_dynamodb::Client;

#[derive(Clone)]
pub struct DynamoDbService {
    client: Client,
}

impl DynamoDbService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let resp = self.client.list_tables().send().await?;
        let names = resp.table_names.unwrap_or_default();
        Ok(names)
    }

    pub async fn describe_table(&self, table_name: &str) -> Result<Vec<(String, String)>> {
        let resp = self.client.describe_table().table_name(table_name).send().await?;
        
        let mut details = Vec::new();
        
        if let Some(table) = resp.table {
            details.push(("Table Name".to_string(), table.table_name.unwrap_or_default()));
            details.push(("Status".to_string(), table.table_status.map(|s| s.as_str().to_string()).unwrap_or_default()));
            details.push(("Item Count".to_string(), table.item_count.unwrap_or(0).to_string()));
            details.push(("Size (Bytes)".to_string(), table.table_size_bytes.unwrap_or(0).to_string()));
            details.push(("Creation Date".to_string(), table.creation_date_time.map(|d| d.to_string()).unwrap_or_default()));
            
            if let Some(key_schema) = table.key_schema {
                let keys: Vec<String> = key_schema.iter()
                    .map(|k| format!("{} ({})", k.attribute_name, k.key_type.as_str()))
                    .collect();
                details.push(("Key Schema".to_string(), keys.join(", ")));
            }
            
            if let Some(gsi) = table.global_secondary_indexes {
                let indexes: Vec<String> = gsi.iter()
                    .map(|i| i.index_name.as_deref().unwrap_or("?").to_string())
                    .collect();
                if !indexes.is_empty() {
                     details.push(("Global Secondary Indexes".to_string(), indexes.join(", ")));
                }
            }
            
             if let Some(lsi) = table.local_secondary_indexes {
                let indexes: Vec<String> = lsi.iter()
                    .map(|i| i.index_name.as_deref().unwrap_or("?").to_string())
                    .collect();
                if !indexes.is_empty() {
                     details.push(("Local Secondary Indexes".to_string(), indexes.join(", ")));
                }
            }
        }

        Ok(details)
    }
}
