use crate::aws::utils::format_size;
use anyhow::Result;
use aws_sdk_dynamodb::Client;

#[derive(Clone)]
pub struct DynamoDbService {
    client: Client,
}

#[derive(Clone, Debug)]
pub enum DynamoDbItem {
    
    Header,
    Separator,
    Table(String),
}

impl DynamoDbService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_tables_with_details(&self) -> Result<Vec<(String, String, String, String)>> {
        let resp = self.client.list_tables().send().await?;
        let table_names = resp.table_names.unwrap_or_default();

        let mut tables = Vec::new();

        for table_name in table_names {
            match self
                .client
                .describe_table()
                .table_name(&table_name)
                .send()
                .await
            {
                Ok(desc) => {
                    if let Some(table) = desc.table {
                        let status = table
                            .table_status
                            .map(|s| s.as_str().to_string())
                            .unwrap_or_else(|| "UNKNOWN".to_string());

                        let item_count = table.item_count.unwrap_or(0);
                        let size_bytes = table.table_size_bytes.unwrap_or(0);
                        let size_str = format_size(size_bytes);

                        tables.push((table_name, status, item_count.to_string(), size_str));
                    }
                }
                Err(_) => {
                    // If we can't describe a table, still include it with unknown details
                    tables.push((
                        table_name,
                        "UNKNOWN".to_string(),
                        "?".to_string(),
                        "?".to_string(),
                    ));
                }
            }
        }

        Ok(tables)
    }

    pub async fn describe_table(&self, table_name: &str) -> Result<Vec<(String, String)>> {
        let resp = self
            .client
            .describe_table()
            .table_name(table_name)
            .send()
            .await?;

        let mut details = Vec::new();

        if let Some(table) = resp.table {
            details.push((
                "Table Name".to_string(),
                table.table_name.unwrap_or_default(),
            ));
            details.push((
                "Status".to_string(),
                table
                    .table_status
                    .map(|s| s.as_str().to_string())
                    .unwrap_or_default(),
            ));
            details.push((
                "Item Count".to_string(),
                table.item_count.unwrap_or(0).to_string(),
            ));
            details.push((
                "Size".to_string(),
                format_size(table.table_size_bytes.unwrap_or(0)),
            ));
            details.push((
                "Creation Date".to_string(),
                table
                    .creation_date_time
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
            ));

            if let Some(key_schema) = table.key_schema {
                let keys: Vec<String> = key_schema
                    .iter()
                    .map(|k| format!("{} ({})", k.attribute_name, k.key_type.as_str()))
                    .collect();
                details.push(("Key Schema".to_string(), keys.join(", ")));
            }

            // Format Global Secondary Indexes as a table
            if let Some(gsi) = table.global_secondary_indexes
                && !gsi.is_empty()
            {
                details.push(("".to_string(), "".to_string())); // Blank line
                details.push(("Global Secondary Indexes".to_string(), "".to_string()));

                // Header
                details.push((
                    "".to_string(),
                    format!("{:<30} {:<15} {:<10}", "Index Name", "Status", "Items"),
                ));
                details.push(("".to_string(), "-".repeat(60)));

                // Index rows
                for index in gsi.iter() {
                    let name = index.index_name.as_deref().unwrap_or("?");
                    let status = index
                        .index_status
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    let items = index.item_count.unwrap_or(0);
                    details.push((
                        "".to_string(),
                        format!("{:<30} {:<15} {:<10}", name, status, items),
                    ));
                }
            }

            // Format Local Secondary Indexes as a table
            if let Some(lsi) = table.local_secondary_indexes
                && !lsi.is_empty()
            {
                details.push(("".to_string(), "".to_string())); // Blank line
                details.push(("Local Secondary Indexes".to_string(), "".to_string()));

                // Header
                details.push((
                    "".to_string(),
                    format!("{:<30} {:<10}", "Index Name", "Items"),
                ));
                details.push(("".to_string(), "-".repeat(45)));

                // Index rows
                for index in lsi.iter() {
                    let name = index.index_name.as_deref().unwrap_or("?");
                    let items = index.item_count.unwrap_or(0);
                    details.push(("".to_string(), format!("{:<30} {:<10}", name, items)));
                }
            }
        }

        Ok(details)
    }

    pub fn format_table_list(
        tables: &[(String, String, String, String)],
    ) -> (Vec<String>, Vec<DynamoDbItem>) {
        if tables.is_empty() {
            return (
                vec!["No DynamoDB Tables found".to_string()],
                vec![DynamoDbItem::Header],
            );
        }

        let max_name_len = tables
            .iter()
            .map(|(name, _, _, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!(
            "{:<width$}  {:<10}  {:<12}  Size",
            "Table Name",
            "Status",
            "Items",
            width = max_name_len
        );
        let separator = "-".repeat(max_name_len + 40).to_string();

        let mut items = vec![header, separator];
        let mut dynamodb_items = vec![DynamoDbItem::Header, DynamoDbItem::Separator];

        for (name, status, item_count, size) in tables {
            items.push(format!(
                "{:<width$}  {:<10}  {:<12}  {}",
                name,
                status,
                item_count,
                size,
                width = max_name_len
            ));
            dynamodb_items.push(DynamoDbItem::Table(name.clone()));
        }

        (items, dynamodb_items)
    }
}
