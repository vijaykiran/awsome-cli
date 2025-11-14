use anyhow::Result;
use crate::aws::AwsClient;

#[derive(Clone, Copy, PartialEq)]
pub enum ServiceType {
    EC2,
    S3,
    IAM,
    CloudWatch,
}

impl ServiceType {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceType::EC2 => "EC2 Instances",
            ServiceType::S3 => "S3 Buckets",
            ServiceType::IAM => "IAM Users",
            ServiceType::CloudWatch => "CloudWatch Alarms",
        }
    }

    pub fn short_name(&self) -> &str {
        match self {
            ServiceType::EC2 => "EC2",
            ServiceType::S3 => "S3",
            ServiceType::IAM => "IAM",
            ServiceType::CloudWatch => "CloudWatch",
        }
    }
}

#[derive(Clone)]
pub struct ServiceInfo {
    pub service_type: ServiceType,
    pub favorite: bool,
}

impl ServiceInfo {
    pub fn new(service_type: ServiceType, favorite: bool) -> Self {
        Self {
            service_type,
            favorite,
        }
    }

    pub fn as_str(&self) -> &str {
        self.service_type.as_str()
    }

    pub fn short_name(&self) -> &str {
        self.service_type.short_name()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error,
}

pub struct App {
    pub services: Vec<ServiceInfo>,
    pub active_service: usize,
    pub selected_index: usize,
    pub items: Vec<String>,
    pub status_message: String,
    pub loading_state: LoadingState,
    pub aws_client: Option<AwsClient>,
    pub error_message: Option<String>,
    pub show_service_popup: bool,
    pub popup_selected_index: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            services: vec![
                ServiceInfo::new(ServiceType::EC2, true),   // EC2 is favorite by default
                ServiceInfo::new(ServiceType::S3, true),    // S3 is favorite by default
                ServiceInfo::new(ServiceType::IAM, false),
                ServiceInfo::new(ServiceType::CloudWatch, false),
            ],
            active_service: 0,
            selected_index: 0,
            items: vec![
                "Initializing AWS client...".to_string(),
            ],
            status_message: "Press Space for services, r to refresh, q to quit".to_string(),
            loading_state: LoadingState::Idle,
            aws_client: None,
            error_message: None,
            show_service_popup: false,
            popup_selected_index: 0,
        }
    }

    pub async fn initialize_aws_client(&mut self) -> Result<()> {
        self.loading_state = LoadingState::Loading;
        self.status_message = "Connecting to AWS...".to_string();

        match AwsClient::new().await {
            Ok(client) => {
                self.aws_client = Some(client);
                self.loading_state = LoadingState::Loaded;
                self.status_message = "AWS client initialized. Press r to load resources.".to_string();
                self.items = vec!["Press 'r' to refresh and load resources".to_string()];
                Ok(())
            }
            Err(e) => {
                self.loading_state = LoadingState::Error;
                self.error_message = Some(format!("Failed to initialize AWS client: {}", e));
                self.status_message = "Error: Failed to connect to AWS. Check credentials.".to_string();
                self.items = vec![
                    "Failed to initialize AWS client".to_string(),
                    "Please check your AWS credentials and configuration".to_string(),
                    format!("Error: {}", e),
                ];
                Err(e)
            }
        }
    }


    pub fn next_item(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn previous_item(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.items.len() - 1;
        }
    }

    pub async fn select_item(&mut self) -> Result<()> {
        if self.selected_index < self.items.len() {
            self.status_message = format!("Selected: {}", self.items[self.selected_index]);
        }
        Ok(())
    }

    pub fn get_active_service(&self) -> &ServiceInfo {
        &self.services[self.active_service]
    }

    pub fn toggle_service_popup(&mut self) {
        self.show_service_popup = !self.show_service_popup;
        if self.show_service_popup {
            self.popup_selected_index = self.active_service;
        }
    }

    pub fn popup_next(&mut self) {
        if !self.services.is_empty() {
            self.popup_selected_index = (self.popup_selected_index + 1) % self.services.len();
        }
    }

    pub fn popup_previous(&mut self) {
        if !self.services.is_empty() {
            if self.popup_selected_index > 0 {
                self.popup_selected_index -= 1;
            } else {
                self.popup_selected_index = self.services.len() - 1;
            }
        }
    }

    pub fn select_popup_service(&mut self) {
        self.active_service = self.popup_selected_index;
        self.show_service_popup = false;
        self.selected_index = 0;
        self.loading_state = LoadingState::Idle;
        self.items = vec![format!("Press 'r' to load {} resources", self.services[self.active_service].as_str())];
        self.status_message = format!("Switched to {}. Press r to refresh.", self.services[self.active_service].as_str());
    }

    pub fn toggle_favorite(&mut self) {
        if self.show_service_popup && self.popup_selected_index < self.services.len() {
            self.services[self.popup_selected_index].favorite = !self.services[self.popup_selected_index].favorite;
            self.status_message = format!(
                "{} {}",
                if self.services[self.popup_selected_index].favorite { "Added to" } else { "Removed from" },
                "favorites"
            );
        }
    }

    pub fn get_favorite_services(&self) -> Vec<(usize, &ServiceInfo)> {
        self.services
            .iter()
            .enumerate()
            .filter(|(_, s)| s.favorite)
            .collect()
    }

    pub async fn refresh_resources(&mut self) -> Result<()> {
        let client = match &self.aws_client {
            Some(c) => c,
            None => {
                self.status_message = "AWS client not initialized".to_string();
                return Ok(());
            }
        };

        self.loading_state = LoadingState::Loading;
        self.items = vec!["Loading...".to_string()];
        self.status_message = format!("Loading {} resources...", self.get_active_service().as_str());

        let result = match self.get_active_service().service_type {
            ServiceType::EC2 => client.list_ec2_instances().await,
            ServiceType::S3 => client.list_s3_buckets().await,
            ServiceType::IAM => client.list_iam_users().await,
            ServiceType::CloudWatch => client.list_cloudwatch_alarms().await,
        };

        match result {
            Ok(resources) => {
                self.loading_state = LoadingState::Loaded;
                if resources.is_empty() {
                    self.items = vec![format!("No {} found", self.get_active_service().as_str())];
                    self.status_message = format!("No resources found for {}", self.get_active_service().as_str());
                } else {
                    self.items = resources;
                    self.status_message = format!("Loaded {} resources ({})", self.items.len(), self.get_active_service().as_str());
                }
                self.selected_index = 0;
                self.error_message = None;
                Ok(())
            }
            Err(e) => {
                self.loading_state = LoadingState::Error;
                self.error_message = Some(format!("{}", e));
                self.items = vec![
                    format!("Error loading {}", self.get_active_service().as_str()),
                    format!("Details: {}", e),
                    "".to_string(),
                    "Possible causes:".to_string(),
                    "- Invalid AWS credentials".to_string(),
                    "- Insufficient IAM permissions".to_string(),
                    "- Network connectivity issues".to_string(),
                ];
                self.status_message = format!("Error: Failed to load resources");
                Ok(())
            }
        }
    }
}
