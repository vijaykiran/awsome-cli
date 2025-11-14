# Quick Start Guide - AWSOME

Get started with AWSOME in 5 minutes!

## Step 1: Configure AWS Credentials

Choose one of these methods:

### Option A: Environment Variables (Quick & Easy)
```bash
export AWS_ACCESS_KEY_ID="your-access-key-id"
export AWS_SECRET_ACCESS_KEY="your-secret-access-key"
export AWS_REGION="us-east-1"
```

### Option B: AWS Credentials File (Recommended)
```bash
# Create credentials file if it doesn't exist
mkdir -p ~/.aws

# Add your credentials
cat > ~/.aws/credentials <<EOF
[default]
aws_access_key_id = your-access-key-id
aws_secret_access_key = your-secret-access-key
EOF

# Add your config
cat > ~/.aws/config <<EOF
[default]
region = us-east-1
EOF
```

## Step 2: Run the Application

```bash
# Development mode (with debug info)
cargo run

# Or build and run release version (faster)
cargo build --release
./target/release/awsome
```

## Step 3: Use the Application

1. **Wait for initialization** - The app will connect to AWS automatically
2. **Press 'r'** - Load EC2 instances (EC2 is selected by default)
3. **Navigate** - Use ↑/↓ arrow keys to browse resources
4. **Switch services** - Press Space to open the service selector
   - Use ↑/↓ to navigate
   - Press 'f' to mark services as favorites (★)
   - Press Enter to select
5. **Refresh** - Press 'r' after switching services to load new data
6. **Quit** - Press 'q' to exit

## Common Issues

### "Failed to initialize AWS client"
- Check your AWS credentials are correctly configured
- Verify your internet connection
- Ensure you have the correct AWS region set

### "Error loading resources"
- Verify your IAM user has the necessary permissions:
  - EC2: `ec2:DescribeInstances`
  - S3: `s3:ListAllMyBuckets`
  - IAM: `iam:ListUsers`
  - CloudWatch: `cloudwatch:DescribeAlarms`

### No resources showing up
- Make sure you pressed 'r' to refresh after selecting a service
- Check if you actually have resources in that AWS service
- Verify you're in the correct AWS region

## Tips

- The border color tells you the current state:
  - **Yellow** = Loading
  - **Green** = Successfully loaded
  - **Red** = Error occurred

- The status bar at the bottom provides detailed information about what's happening

- You can refresh the current service anytime by pressing 'r' again

## Next Steps

- Check out the [README.md](README.md) for more detailed information
- See [CLAUDE.md](CLAUDE.md) for development documentation
- Report issues or request features on GitHub

Enjoy using AWSOME! 🚀
