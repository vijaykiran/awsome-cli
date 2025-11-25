use anyhow::Result;
use aws_sdk_lambda::Client as LambdaClient;

#[derive(Clone)]
pub struct LambdaService {
    client: LambdaClient,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LambdaItem {
    Header,
    Separator,
    Function(String),
}

impl LambdaService {
    pub fn new(client: LambdaClient) -> Self {
        Self { client }
    }

    pub async fn list_functions(&self) -> Result<Vec<(String, String, String)>> {
        let resp = self.client.list_functions().send().await?;
        let functions = resp
            .functions
            .unwrap_or_default()
            .into_iter()
            .filter_map(|f| {
                let name = f.function_name?;
                let runtime = f
                    .runtime
                    .map(|r| r.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let last_modified = f.last_modified.unwrap_or_else(|| "unknown".to_string());
                Some((name, runtime, last_modified))
            })
            .collect();
        Ok(functions)
    }

    pub async fn get_function(
        &self,
        name: &str,
    ) -> Result<aws_sdk_lambda::types::FunctionConfiguration> {
        let resp = self
            .client
            .get_function()
            .function_name(name)
            .send()
            .await?;
        resp.configuration
            .ok_or_else(|| anyhow::anyhow!("Function configuration not found"))
    }

    pub fn format_function_list(
        functions: &[(String, String, String)],
    ) -> (Vec<String>, Vec<LambdaItem>) {
        if functions.is_empty() {
            return (
                vec!["No Lambda Functions found".to_string()],
                vec![LambdaItem::Header],
            );
        }

        let max_name_len = functions
            .iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(20)
            .max(20);

        let max_runtime_len = functions
            .iter()
            .map(|(_, runtime, _)| runtime.len())
            .max()
            .unwrap_or(10)
            .max(10);

        let header = format!(
            "{:<name_width$}  {:<runtime_width$}  {:<mod_width$}",
            "Function Name",
            "Runtime",
            "Last Modified",
            name_width = max_name_len,
            runtime_width = max_runtime_len,
            mod_width = 25
        );
        let separator = "-".repeat(header.len()).to_string();

        let mut items = vec![header, separator];
        let mut lambda_items = vec![LambdaItem::Header, LambdaItem::Separator];

        for (name, runtime, last_modified) in functions {
            items.push(format!(
                "{:<name_width$}  {:<runtime_width$}  {:<mod_width$}",
                name,
                runtime,
                last_modified,
                name_width = max_name_len,
                runtime_width = max_runtime_len,
                mod_width = 25
            ));
            lambda_items.push(LambdaItem::Function(name.clone()));
        }
        (items, lambda_items)
    }

    pub fn get_function_details_pairs(
        config: &aws_sdk_lambda::types::FunctionConfiguration,
    ) -> Vec<(String, String)> {
        let last_modified = config.last_modified().unwrap_or("unknown").to_string();
        vec![
            (
                "Name".to_string(),
                config.function_name().unwrap_or("unknown").to_string(),
            ),
            (
                "ARN".to_string(),
                config.function_arn().unwrap_or("unknown").to_string(),
            ),
            (
                "Runtime".to_string(),
                config
                    .runtime()
                    .map(|r| r.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ),
            (
                "Handler".to_string(),
                config.handler().unwrap_or("unknown").to_string(),
            ),
            (
                "Description".to_string(),
                config.description().unwrap_or("").to_string(),
            ),
            (
                "Memory Size".to_string(),
                config
                    .memory_size()
                    .map(|m| format!("{} MB", m))
                    .unwrap_or("unknown".to_string()),
            ),
            (
                "Timeout".to_string(),
                config
                    .timeout()
                    .map(|t| format!("{} s", t))
                    .unwrap_or("unknown".to_string()),
            ),
            ("Last Modified".to_string(), last_modified),
            (
                "Role".to_string(),
                config.role().unwrap_or("unknown").to_string(),
            ),
            (
                "State".to_string(),
                config
                    .state()
                    .map(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_function_list() {
        let functions = vec![
            (
                "func1".to_string(),
                "python3.9".to_string(),
                "2023-01-01".to_string(),
            ),
            (
                "func2".to_string(),
                "nodejs18.x".to_string(),
                "2023-01-02".to_string(),
            ),
        ];
        let (items, lambda_items) = LambdaService::format_function_list(&functions);

        assert_eq!(items.len(), 4); // Header, Separator, 2 functions
        assert!(items[0].contains("Function Name"));
        assert!(items[0].contains("Runtime"));
        assert!(items[0].contains("Last Modified"));
        assert!(items[2].contains("func1"));
        assert!(items[2].contains("python3.9"));

        assert!(matches!(lambda_items[2], LambdaItem::Function(_)));
    }

    #[test]
    fn test_get_function_details_pairs() {
        let config = aws_sdk_lambda::types::FunctionConfiguration::builder()
            .function_name("test-func")
            .function_arn("arn:aws:lambda:us-east-1:123456789012:function:test-func")
            .runtime(aws_sdk_lambda::types::Runtime::Python39)
            .handler("index.handler")
            .memory_size(128)
            .timeout(30)
            .build();

        let details = LambdaService::get_function_details_pairs(&config);

        assert_eq!(details.len(), 10);
        assert_eq!(details[0], ("Name".to_string(), "test-func".to_string()));
        assert_eq!(details[2], ("Runtime".to_string(), "python3.9".to_string()));
        assert_eq!(
            details[5],
            ("Memory Size".to_string(), "128 MB".to_string())
        );
    }
}
