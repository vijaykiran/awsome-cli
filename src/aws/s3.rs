use anyhow::Result;
use aws_sdk_s3::Client as S3Client;

#[derive(Clone)]
pub struct S3Service {
    client: S3Client,
}

impl S3Service {
    pub fn new(client: S3Client) -> Self {
        Self { client }
    }

    pub async fn list_buckets(&self) -> Result<Vec<(String, String)>> {
        let resp = self.client.list_buckets().send().await?;

        let buckets: Vec<(String, String)> = resp
            .buckets()
            .iter()
            .filter_map(|b| {
                let name = b.name()?.to_string();
                let creation_date = b
                    .creation_date()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                Some((name, creation_date))
            })
            .collect();

        Ok(buckets)
    }

    pub async fn get_bucket_details(&self, bucket_name: &str) -> Result<Vec<(String, String)>> {
        let mut details = Vec::new();

        details.push(("Bucket Name".to_string(), bucket_name.to_string()));

        // Get bucket location
        match self
            .client
            .get_bucket_location()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(location) => {
                let region = location
                    .location_constraint()
                    .map(|lc| format!("{:?}", lc))
                    .unwrap_or_else(|| "us-east-1".to_string());
                details.push(("Region".to_string(), region));
            }
            Err(e) => {
                details.push(("Region".to_string(), format!("Error: {}", e)));
            }
        }

        // Get versioning status
        match self
            .client
            .get_bucket_versioning()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(versioning) => {
                let status = versioning
                    .status()
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "Disabled".to_string());
                details.push(("Versioning".to_string(), status));
            }
            Err(e) => {
                details.push(("Versioning".to_string(), format!("Error: {}", e)));
            }
        }

        // Get encryption configuration
        match self
            .client
            .get_bucket_encryption()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(encryption) => {
                if let Some(rules) = encryption
                    .server_side_encryption_configuration()
                    .and_then(|config| config.rules().first())
                {
                    if let Some(def) = rules.apply_server_side_encryption_by_default() {
                        let algo = format!("{:?}", def.sse_algorithm());
                        details.push(("Encryption".to_string(), algo));
                    } else {
                        details.push(("Encryption".to_string(), "Unknown".to_string()));
                    }
                } else {
                    details.push(("Encryption".to_string(), "None".to_string()));
                }
            }
            Err(_) => {
                details.push(("Encryption".to_string(), "None".to_string()));
            }
        }

        // Get ACL
        match self
            .client
            .get_bucket_acl()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(acl) => {
                let grants = acl.grants().len();
                details.push(("ACL Grants".to_string(), format!("{} grant(s)", grants)));

                // Show owner
                if let Some(owner) = acl.owner() {
                    if let Some(display_name) = owner.display_name() {
                        details.push(("Owner".to_string(), display_name.to_string()));
                    }
                }
            }
            Err(e) => {
                details.push(("ACL".to_string(), format!("Error: {}", e)));
            }
        }

        // Get public access block
        match self
            .client
            .get_public_access_block()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(pab) => {
                if let Some(config) = pab.public_access_block_configuration() {
                    let block_public_acls = config.block_public_acls().unwrap_or(false);
                    let ignore_public_acls = config.ignore_public_acls().unwrap_or(false);
                    let block_public_policy = config.block_public_policy().unwrap_or(false);
                    let restrict_public_buckets = config.restrict_public_buckets().unwrap_or(false);

                    let all_blocked = block_public_acls
                        && ignore_public_acls
                        && block_public_policy
                        && restrict_public_buckets;

                    if all_blocked {
                        details.push(("Public Access".to_string(), "All Blocked".to_string()));
                    } else {
                        details
                            .push(("Public Access".to_string(), "Partially Blocked".to_string()));
                    }
                }
            }
            Err(_) => {
                details.push(("Public Access".to_string(), "Unknown".to_string()));
            }
        }

        // Get tags
        match self
            .client
            .get_bucket_tagging()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(tagging) => {
                let tag_count = tagging.tag_set().len();
                details.push(("Tags".to_string(), format!("{} tag(s)", tag_count)));

                // Show first few tags
                for (idx, tag) in tagging.tag_set().iter().take(3).enumerate() {
                    let key = tag.key();
                    let value = tag.value();
                    details.push((format!("  Tag {}", idx + 1), format!("{} = {}", key, value)));
                }
            }
            Err(_) => {
                details.push(("Tags".to_string(), "None".to_string()));
            }
        }

        Ok(details)
    }

    pub async fn get_object_details(&self, bucket: &str, key: &str) -> Result<Vec<(String, String)>> {
        let mut details = Vec::new();
        details.push(("Name".to_string(), key.to_string()));

        match self.client.head_object().bucket(bucket).key(key).send().await {
            Ok(head) => {
                if let Some(size) = head.content_length() {
                     details.push(("Size".to_string(), format_size(size)));
                }
                
                if let Some(last_modified) = head.last_modified() {
                    details.push(("Last Modified".to_string(), last_modified.to_string()));
                }
                
                if let Some(etag) = head.e_tag() {
                    details.push(("ETag".to_string(), etag.to_string()));
                }
                
                if let Some(storage_class) = head.storage_class() {
                    details.push(("Storage Class".to_string(), format!("{:?}", storage_class)));
                }
                
                if let Some(content_type) = head.content_type() {
                    details.push(("Content Type".to_string(), content_type.to_string()));
                }
            }
            Err(e) => {
                details.push(("Error".to_string(), format!("Failed to get object details: {}", e)));
            }
        }
        
        Ok(details)
    }
    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<(String, String, String)>> {
        let mut objects = Vec::new();

        let resp = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix)
            .delimiter("/")
            .send()
            .await?;

        // Add folders (CommonPrefixes)
        for cp in resp.common_prefixes() {
            if let Some(folder_prefix) = cp.prefix() {
                // Remove the parent prefix from the display name
                // Actually we want the relative name
                
                 // If we are in "folder/", and we get "folder/sub/", we want to show "sub/"
                 
                 let name = if !prefix.is_empty() && folder_prefix.starts_with(prefix) {
                     folder_prefix.strip_prefix(prefix).unwrap_or(folder_prefix)
                 } else {
                     folder_prefix
                 };

                objects.push((name.to_string(), "DIR".to_string(), "".to_string()));
            }
        }

        // Add files (Contents)
        // Add files (Contents)
        for object in resp.contents() {
            if let Some(key) = object.key() {
                // Skip the folder object itself if it exists
                if key == prefix {
                    continue;
                }

                let name = if !prefix.is_empty() && key.starts_with(prefix) {
                     key.strip_prefix(prefix).unwrap_or(key)
                } else {
                     key
                };

                let size = object.size().unwrap_or(0);
                let size_str = format_size(size);
                
                let date = object
                    .last_modified()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                objects.push((name.to_string(), size_str, date));
            }
        }

        Ok(objects)
    }
}

fn format_size(size: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

pub enum S3NavigationAction {
    EnterFolder(String),
    GoBack,
    ShowDetails(String),
    EnterBucket(String),
    None,
}

#[derive(Clone, Debug)]
pub enum S3Item {
    Header,
    Separator,
    Bucket(String),
    Folder(String),
    Object(String),
    ParentDir,
}

impl S3Service {
    pub fn format_bucket_list(buckets: &[(String, String)]) -> (Vec<String>, Vec<S3Item>) {
        if buckets.is_empty() {
            return (vec!["No S3 Buckets found".to_string()], vec![S3Item::Header]);
        }

        let max_name_len = buckets.iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}  Creation Date", "Bucket Name", width = max_name_len);
        let separator = format!("{}", "-".repeat(max_name_len + 25));

        let mut items = vec![header, separator];
        let mut s3_items = vec![S3Item::Header, S3Item::Separator];

        for (name, date) in buckets {
            items.push(format!("{:<width$}  {}", name, date, width = max_name_len));
            s3_items.push(S3Item::Bucket(name.clone()));
        }
        (items, s3_items)
    }

    pub fn format_object_list(objects: &[(String, String, String)], _bucket: &str, _prefix: &str) -> (Vec<String>, Vec<S3Item>) {
        let max_name_len = objects.iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!("{:<width$}  {:<10}  Last Modified", "Name", "Size", width = max_name_len);
        let separator = format!("{}", "-".repeat(max_name_len + 30));

        let mut items = vec![header, separator];
        let mut s3_items = vec![S3Item::Header, S3Item::Separator];

        items.push("..".to_string());
        s3_items.push(S3Item::ParentDir);

        for (name, size, date) in objects {
            items.push(format!("{:<width$}  {:<10}  {}", name, size, date, width = max_name_len));
            if size == "DIR" {
                s3_items.push(S3Item::Folder(name.clone()));
            } else {
                s3_items.push(S3Item::Object(name.clone()));
            }
        }
        (items, s3_items)
    }

    pub fn handle_selection(item: &S3Item, current_path: &Option<String>) -> S3NavigationAction {
        match item {
            S3Item::Bucket(name) => S3NavigationAction::EnterBucket(format!("{}/", name)),
            S3Item::Folder(name) => {
                 if let Some(path) = current_path {
                     S3NavigationAction::EnterFolder(format!("{}{}", path, name))
                 } else {
                     S3NavigationAction::None
                 }
            },
            S3Item::Object(key) => {
                 if let Some(path) = current_path {
                     // Construct full key
                     let parts: Vec<&str> = path.splitn(2, '/').collect();
                     let prefix = if parts.len() > 1 { parts[1] } else { "" };
                     let full_key = format!("{}{}", prefix, key);
                     S3NavigationAction::ShowDetails(full_key)
                 } else {
                     S3NavigationAction::None
                 }
            },
            S3Item::ParentDir => S3NavigationAction::GoBack,
            _ => S3NavigationAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_bucket_list() {
        // Test empty list
        let (items, s3_items) = S3Service::format_bucket_list(&[]);
        assert_eq!(items[0], "No S3 Buckets found");
        assert!(matches!(s3_items[0], S3Item::Header));

        // Test populated list
        let buckets = vec![
            ("bucket1".to_string(), "2023-01-01".to_string()),
            ("bucket2".to_string(), "2023-01-02".to_string()),
        ];
        let (items, s3_items) = S3Service::format_bucket_list(&buckets);
        
        assert_eq!(items.len(), 4); // Header, Separator, 2 buckets
        assert!(items[0].contains("Bucket Name"));
        assert!(items[2].contains("bucket1"));
        assert!(items[3].contains("bucket2"));
        
        assert!(matches!(s3_items[2], S3Item::Bucket(_)));
        if let S3Item::Bucket(name) = &s3_items[2] {
            assert_eq!(name, "bucket1");
        }
    }

    #[test]
    fn test_format_object_list() {
        let objects = vec![
            ("folder/".to_string(), "DIR".to_string(), "".to_string()),
            ("file.txt".to_string(), "1.00 KB".to_string(), "2023-01-01".to_string()),
        ];
        
        let (items, s3_items) = S3Service::format_object_list(&objects, "bucket", "");
        
        assert_eq!(items.len(), 5); // Header, Separator, ParentDir, Folder, File
        assert_eq!(items[2], "..");
        assert!(matches!(s3_items[2], S3Item::ParentDir));
        
        assert!(items[3].contains("folder/"));
        assert!(matches!(s3_items[3], S3Item::Folder(_)));
        
        assert!(items[4].contains("file.txt"));
        assert!(matches!(s3_items[4], S3Item::Object(_)));
    }
}
