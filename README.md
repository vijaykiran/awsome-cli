# AWSOME - AWS Console UI

A terminal-based AWS console UI built with Rust and ratatui, providing an easy and intuitive interface for managing AWS resources.

## Features

- 🖥️ Beautiful terminal-based user interface with color-coded states
- ⚡ Fast and lightweight Rust implementation
- 🔄 Real-time AWS resource listing for multiple services:
  - EC2 (Elastic Compute Cloud) - List instances with their states
  - S3 (Simple Storage Service) - List all buckets
  - IAM (Identity and Access Management) - List users
  - CloudWatch - List alarms
- ⭐ **Favorite services** - Mark frequently used services as favorites for quick access
- 📋 **Popup service selector** - Clean, centered popup to switch between services
- 🎯 **Smart top bar** - Shows only your favorite services for quick reference
- 🎨 Visual feedback for loading, success, and error states
- 🔐 Secure AWS credential handling using standard SDK credential chain
- ⚠️ Detailed error messages with troubleshooting guidance

## Prerequisites

- Rust 1.90.0 or later
- AWS credentials configured (see Configuration section)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd rrrr

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Configuration

Configure your AWS credentials using one of these methods:

### Option 1: Environment Variables
```bash
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
export AWS_REGION="us-east-1"
```

### Option 2: AWS Credentials File
Create or edit `~/.aws/credentials`:
```ini
[default]
aws_access_key_id = your-access-key
aws_secret_access_key = your-secret-key
```

And `~/.aws/config`:
```ini
[default]
region = us-east-1
```

## Usage

Launch the application:
```bash
cargo run
```

### Keyboard Controls

**Main View:**
- **Space**: Open service selector popup
- **r** or **R**: Refresh and load resources from AWS
- **↑/↓**: Navigate through resource list
- **Enter**: Select current item (displays selection in status bar)
- **q**: Quit application

**Service Selector Popup:**
- **↑/↓**: Navigate through services
- **Enter**: Select service and close popup
- **f**: Toggle service as favorite (★)
- **Esc** or **Space**: Close popup without changing service

### Visual Indicators

The UI provides visual feedback through color coding:
- **Yellow border**: Loading resources from AWS
- **Green border**: Successfully loaded resources
- **Red border**: Error occurred (check status bar for details)
- **Status bar colors**: Match the current state for quick visual reference
- **Top bar**: Shows only favorite services (marked with ★) for quick access
- **Cyan color**: Favorite services in the popup selector

## Development

See [CLAUDE.md](CLAUDE.md) for detailed development documentation.

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Project Structure

```
src/
├── main.rs    # Application entry point and event loop
├── app.rs     # Application state management
├── ui.rs      # UI rendering logic
└── aws.rs     # AWS SDK client wrapper
```

## How It Works

1. Launch the application - it automatically initializes the AWS client
2. Press **r** to load resources for the current service (EC2 by default)
3. Press **Space** to open the service selector popup
4. In the popup:
   - Use **↑/↓** to navigate services
   - Press **f** to mark/unmark services as favorites (★)
   - Press **Enter** to select a service
5. Your favorite services appear in the top bar for quick reference
6. Press **r** again to refresh the resource list for the new service
7. Use **↑/↓** to navigate through resources
8. Watch the status bar for real-time feedback on operations

## Roadmap

- [x] AWS client integration
- [x] Loading states and error handling
- [x] Real-time resource listing for EC2, S3, IAM, and CloudWatch
- [ ] Add resource detail views
- [ ] Implement resource filtering/search
- [ ] Add support for more AWS services (Lambda, DynamoDB, RDS, etc.)
- [ ] Add resource creation/modification capabilities
- [ ] Add configuration file support
- [ ] Implement caching to improve performance
- [ ] Support for multiple AWS profiles

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
