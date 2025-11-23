use anyhow::Result;
use aws_config::BehaviorVersion;

mod cloudwatch;
mod dynamodb;
mod ec2;
mod ecs;
mod iam;
mod s3;
pub mod utils;

pub use cloudwatch::CloudwatchService;
pub use dynamodb::{DynamoDbItem, DynamoDbService};
pub use ec2::{Ec2Item, Ec2Service};
pub use ecs::{EcsItem, EcsService};
pub use iam::{IamItem, IamService};
pub use s3::{S3Item, S3NavigationAction, S3Service};

#[derive(Clone)]
pub struct AwsClient {
    ec2_service: Ec2Service,
    s3_service: S3Service,
    iam_service: IamService,
    cloudwatch_service: CloudwatchService,
    dynamodb_service: DynamoDbService,
    ecs_service: EcsService,
}

impl AwsClient {
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

        Ok(Self {
            ec2_service: Ec2Service::new(aws_sdk_ec2::Client::new(&config)),
            s3_service: S3Service::new(aws_sdk_s3::Client::new(&config)),
            iam_service: IamService::new(aws_sdk_iam::Client::new(&config)),
            cloudwatch_service: CloudwatchService::new(aws_sdk_cloudwatch::Client::new(&config)),
            dynamodb_service: DynamoDbService::new(aws_sdk_dynamodb::Client::new(&config)),
            ecs_service: EcsService::new(aws_sdk_ecs::Client::new(&config)),
        })
    }

    pub async fn list_ec2_instances(
        &self,
    ) -> Result<Vec<(String, String, String, String, String)>> {
        self.ec2_service.list_instances().await
    }

    pub async fn list_s3_buckets(&self) -> Result<Vec<(String, String)>> {
        self.s3_service.list_buckets().await
    }

    pub async fn list_iam_users(&self) -> Result<Vec<(String, String, String)>> {
        self.iam_service.list_users().await
    }

    pub async fn list_cloudwatch_alarms(&self) -> Result<Vec<String>> {
        self.cloudwatch_service.list_alarms().await
    }

    pub async fn list_dynamodb_tables(&self) -> Result<Vec<(String, String, String, String)>> {
        self.dynamodb_service.list_tables_with_details().await
    }

    pub async fn get_s3_bucket_details(&self, bucket_name: &str) -> Result<Vec<(String, String)>> {
        self.s3_service.get_bucket_details(bucket_name).await
    }

    pub async fn list_s3_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<(String, String, String)>> {
        self.s3_service.list_objects(bucket, prefix).await
    }

    pub async fn get_s3_object_details(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Vec<(String, String)>> {
        self.s3_service.get_object_details(bucket, key).await
    }

    pub async fn get_dynamodb_table_details(
        &self,
        table_name: &str,
    ) -> Result<Vec<(String, String)>> {
        self.dynamodb_service.describe_table(table_name).await
    }

    pub async fn list_ecs_clusters(&self) -> Result<Vec<String>> {
        self.ecs_service.list_clusters().await
    }

    pub async fn list_ecs_services(&self, cluster: &str) -> Result<Vec<String>> {
        self.ecs_service.list_services(cluster).await
    }

    pub async fn list_ecs_tasks(
        &self,
        cluster: &str,
        service: Option<&str>,
    ) -> Result<Vec<(String, String, String, String, String)>> {
        self.ecs_service.list_tasks(cluster, service).await
    }
}
