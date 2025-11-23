use aws_sdk_s3::operation::list_buckets::ListBucketsOutput;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::types::{Bucket, Object};
use aws_smithy_mocks::{mock, mock_client};
use aws_smithy_types::date_time::DateTime;
use awsome::aws::S3Service;

#[tokio::test]
async fn test_list_buckets() {
    // Create a rule that returns a successful response for list_buckets
    let list_buckets_rule = mock!(aws_sdk_s3::Client::list_buckets).then_output(|| {
        ListBucketsOutput::builder()
            .buckets(
                Bucket::builder()
                    .name("test-bucket-1")
                    .creation_date(DateTime::from_secs(1672531200)) // 2023-01-01
                    .build(),
            )
            .buckets(
                Bucket::builder()
                    .name("test-bucket-2")
                    .creation_date(DateTime::from_secs(1672617600)) // 2023-01-02
                    .build(),
            )
            .build()
    });

    // Create a mocked client with the rule
    let client = mock_client!(aws_sdk_s3, [&list_buckets_rule]);
    let s3_service = S3Service::new(client);

    // Call the service method
    let buckets = s3_service
        .list_buckets()
        .await
        .expect("failed to list buckets");

    // Verify results
    assert_eq!(buckets.len(), 2);
    assert_eq!(buckets[0].0, "test-bucket-1");
    assert_eq!(buckets[1].0, "test-bucket-2");

    // Verify the rule was used
    assert_eq!(list_buckets_rule.num_calls(), 1);
}

#[tokio::test]
async fn test_list_objects() {
    // Create a rule for list_objects_v2
    let list_objects_rule = mock!(aws_sdk_s3::Client::list_objects_v2)
        .match_requests(|req| {
            req.bucket() == Some("test-bucket") && req.prefix() == Some("folder/")
        })
        .then_output(|| {
            ListObjectsV2Output::builder()
                .contents(
                    Object::builder()
                        .key("folder/file1.txt")
                        .size(1024)
                        .last_modified(DateTime::from_secs(1672531200))
                        .build(),
                )
                .build()
        });

    let client = mock_client!(aws_sdk_s3, [&list_objects_rule]);
    let s3_service = S3Service::new(client);

    let objects = s3_service
        .list_objects("test-bucket", "folder/")
        .await
        .expect("failed to list objects");

    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].0, "file1.txt"); // Name should be stripped of prefix
    assert_eq!(objects[0].1, "1.00 KB"); // Size formatted

    assert_eq!(list_objects_rule.num_calls(), 1);
}
