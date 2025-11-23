use anyhow::Result;
use crate::aws::{AwsClient, S3Service, S3NavigationAction, S3Item};

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
    pub profile_name: String,
    pub show_detail_popup: bool,
    pub detail_content: Vec<(String, String)>, // Key-value pairs for details
    pub detail_loading: bool,
    pub detail_scroll: usize,
    pub animation_frame: usize,
    pub show_quit_confirm: bool,
    pub current_path: Option<String>, // For S3 navigation (bucket/prefix)
    pub s3_items: Vec<S3Item>,
}

impl App {
    pub fn new() -> Self {
        // Get AWS profile name from environment or default to "default"
        let profile_name = std::env::var("AWS_PROFILE")
            .unwrap_or_else(|_| "default".to_string());

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
            profile_name,
            show_detail_popup: false,
            detail_content: Vec::new(),
            detail_loading: false,
            detail_scroll: 0,
            animation_frame: 0,
            show_quit_confirm: false,
            current_path: None,
            s3_items: Vec::new(),
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
            let selected = self.items[self.selected_index].clone();
            
            // Handle S3 navigation
            if matches!(self.get_active_service().service_type, ServiceType::S3) {
                let action = if self.selected_index < self.s3_items.len() {
                    S3Service::handle_selection(&self.s3_items[self.selected_index], &self.current_path)
                } else {
                    S3NavigationAction::None
                };
                
                match action {
                    S3NavigationAction::EnterBucket(path) => {
                        self.current_path = Some(path);
                        self.refresh_resources().await?;
                        return Ok(());
                    }
                    S3NavigationAction::EnterFolder(path) => {
                        self.current_path = Some(path);
                        self.refresh_resources().await?;
                        return Ok(());
                    }
                    S3NavigationAction::GoBack => {
                        if let Some(path) = &self.current_path {
                            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                            if parts.len() <= 1 {
                                self.current_path = None;
                            } else {
                                let new_path = parts[..parts.len()-1].join("/") + "/";
                                self.current_path = Some(new_path);
                            }
                            self.refresh_resources().await?;
                            return Ok(());
                        }
                    }
                    S3NavigationAction::ShowDetails(_key) => {
                        self.show_resource_details().await?;
                        return Ok(());
                    }
                    S3NavigationAction::None => {
                        if self.current_path.is_none() {
                             self.status_message = "Please select a bucket row".to_string();
                        }
                        return Ok(());
                    }
                }
            }
            
            self.status_message = format!("Selected: {}", selected);
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
        self.current_path = None; // Reset path when switching services
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

    pub fn close_detail_popup(&mut self) {
        self.show_detail_popup = false;
        self.detail_content.clear();
        self.detail_scroll = 0;
    }

    pub fn detail_scroll_up(&mut self) {
        if self.detail_scroll > 0 {
            self.detail_scroll -= 1;
        }
    }

    pub fn detail_scroll_down(&mut self) {
        if self.detail_scroll < self.detail_content.len().saturating_sub(1) {
            self.detail_scroll += 1;
        }
    }

    pub async fn show_resource_details(&mut self) -> Result<()> {
        if self.items.is_empty() || self.selected_index >= self.items.len() {
            return Ok(());
        }

        let client = match &self.aws_client {
            Some(c) => c,
            None => {
                self.status_message = "AWS client not initialized".to_string();
                return Ok(());
            }
        };

        let resource_line = &self.items[self.selected_index];

        // Extract resource name based on service type
        let resource_name = match self.get_active_service().service_type {
            ServiceType::S3 => {
                if let Some(path) = &self.current_path {
                    // We are inside a bucket, show object details
                    // The selected line is a table row: "Name  Size  Date"
                    let name = resource_line.split_whitespace().next().unwrap_or(resource_line);
                    // Construct full key
                    let parts: Vec<&str> = path.splitn(2, '/').collect();
                    let prefix = if parts.len() > 1 { parts[1] } else { "" };
                    format!("{}{}", prefix, name)
                } else {
                    // For S3 buckets, extract bucket name from table format
                    // Skip header and separator rows
                    if self.selected_index <= 1 {
                        self.status_message = "Please select a bucket row".to_string();
                        return Ok(());
                    }
                    // Extract bucket name (everything before the two spaces and date)
                    resource_line.split_whitespace().next().unwrap_or(resource_line).to_string()
                }
            }
            _ => resource_line.clone(),
        };

        // Show popup with loading state
        self.show_detail_popup = true;
        self.detail_loading = true;
        self.detail_content = vec![("Loading...".to_string(), "".to_string())];

        // Fetch details based on service type
        let result = match self.get_active_service().service_type {
            ServiceType::S3 => client.get_s3_bucket_details(&resource_name).await,
            ServiceType::EC2 => {
                // For now, just show a placeholder
                Ok(vec![("Instance ID".to_string(), resource_name.clone())])
            }
            ServiceType::IAM => {
                Ok(vec![("User Name".to_string(), resource_name.clone())])
            }
            ServiceType::CloudWatch => {
                Ok(vec![("Alarm Name".to_string(), resource_name.clone())])
            }
        };

        match result {
            Ok(details) => {
                self.detail_content = details;
                self.detail_loading = false;
                self.status_message = format!("Viewing details for {}", resource_name);
            }
            Err(e) => {
                self.detail_content = vec![
                    ("Error".to_string(), "Failed to load details".to_string()),
                    ("Details".to_string(), format!("{}", e)),
                ];
                self.detail_loading = false;
                self.status_message = format!("Error loading details: {}", e);
            }
        }

        Ok(())
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

        match self.get_active_service().service_type {
            ServiceType::EC2 => {
                match client.list_ec2_instances().await {
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
                    Err(e) => self.handle_resource_error(e),
                }
            }
            ServiceType::S3 => {
                if let Some(path) = &self.current_path {
                    // List objects in bucket/prefix
                    let parts: Vec<&str> = path.splitn(2, '/').collect();
                    let bucket = parts[0];
                    let prefix = if parts.len() > 1 { parts[1] } else { "" };
                    
                    match client.list_s3_objects(bucket, prefix).await {
                        Ok(objects) => {
                            self.loading_state = LoadingState::Loaded;
                            let (items, s3_items) = S3Service::format_object_list(&objects, bucket, prefix);
                            self.items = items;
                            self.s3_items = s3_items;
                            self.status_message = format!("Browsing s3://{}/{}", bucket, prefix);
                        }
                        Err(e) => self.handle_resource_error(e)?,
                    }
                    Ok(())
                } else {
                    match client.list_s3_buckets().await {
                        Ok(buckets) => {
                            self.loading_state = LoadingState::Loaded;
                            let (items, s3_items) = S3Service::format_bucket_list(&buckets);
                            self.items = items;
                            self.s3_items = s3_items;
                            if buckets.is_empty() {
                                self.status_message = format!("No resources found for {}", self.get_active_service().as_str());
                            } else {
                                self.status_message = format!("Loaded {} buckets", buckets.len());
                            }
                            self.selected_index = 0;
                            self.error_message = None;
                            Ok(())
                        }
                        Err(e) => self.handle_resource_error(e),
                    }
                }
            }
            ServiceType::IAM => {
                match client.list_iam_users().await {
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
                    Err(e) => self.handle_resource_error(e),
                }
            }
            ServiceType::CloudWatch => {
                match client.list_cloudwatch_alarms().await {
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
                    Err(e) => self.handle_resource_error(e),
                }
            }
        }
    }

    fn handle_resource_error(&mut self, e: anyhow::Error) -> Result<()> {
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

    pub fn tick_animation(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % 10;
    }

    pub fn get_loading_spinner(&self) -> &str {
        const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER_FRAMES[self.animation_frame]
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.loading_state, LoadingState::Loading) || self.detail_loading
    }

    pub fn show_quit_confirmation(&mut self) {
        self.show_quit_confirm = true;
    }

    pub fn hide_quit_confirmation(&mut self) {
        self.show_quit_confirm = false;
    }
}
