use anyhow::Result;
use aws_sdk_iam::Client as IamClient;

#[derive(Clone)]
pub struct IamService {
    client: IamClient,
}

impl IamService {
    pub fn new(client: IamClient) -> Self {
        Self { client }
    }

    pub async fn list_users(&self) -> Result<Vec<(String, String, String)>> {
        let resp = self.client.list_users().send().await?;

        let users: Vec<(String, String, String)> = resp
            .users()
            .iter()
            .map(|u| {
                let name = u.user_name().to_string();
                let id = u.user_id().to_string();
                let date = u.create_date().to_string();
                (name, id, date)
            })
            .collect();

        Ok(users)
    }

    pub fn format_user_list(users: &[(String, String, String)]) -> (Vec<String>, Vec<IamItem>) {
        if users.is_empty() {
            return (vec!["No IAM Users found".to_string()], vec![IamItem::Header]);
        }

        // Calculate column widths
        let max_name_len = users.iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);
            
        let max_id_len = users.iter()
            .map(|(_, id, _)| id.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let header = format!(
            "{:<width_name$}  {:<width_id$}  Creation Date", 
            "User Name", "User ID", 
            width_name = max_name_len, 
            width_id = max_id_len
        );
        let separator = format!("{}", "-".repeat(max_name_len + max_id_len + 20));

        let mut items = vec![header, separator];
        let mut iam_items = vec![IamItem::Header, IamItem::Separator];

        for (name, id, date) in users {
            items.push(format!(
                "{:<width_name$}  {:<width_id$}  {}", 
                name, id, date, 
                width_name = max_name_len, 
                width_id = max_id_len
            ));
            iam_items.push(IamItem::User(name.clone()));
        }
        (items, iam_items)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IamItem {
    Header,
    Separator,
    User(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_user_list() {
        // Test empty list
        let (items, iam_items) = IamService::format_user_list(&[]);
        assert_eq!(items[0], "No IAM Users found");
        assert!(matches!(iam_items[0], IamItem::Header));

        // Test populated list
        let users = vec![
            ("user1".to_string(), "id1".to_string(), "2023-01-01".to_string()),
            ("user2".to_string(), "id2".to_string(), "2023-01-02".to_string()),
        ];
        let (items, iam_items) = IamService::format_user_list(&users);
        
        assert_eq!(items.len(), 4); // Header, Separator, 2 users
        assert!(items[0].contains("User Name"));
        assert!(items[0].contains("User ID"));
        assert!(items[2].contains("user1"));
        assert!(items[3].contains("user2"));
        
        assert!(matches!(iam_items[2], IamItem::User(_)));
        if let IamItem::User(name) = &iam_items[2] {
            assert_eq!(name, "user1");
        }
    }
}
