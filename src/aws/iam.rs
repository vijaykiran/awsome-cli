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

    pub async fn list_users(&self) -> Result<Vec<String>> {
        let resp = self.client.list_users().send().await?;

        let users: Vec<String> = resp
            .users()
            .iter()
            .map(|u| u.user_name().to_string())
            .collect();

        Ok(users)
    }
}
