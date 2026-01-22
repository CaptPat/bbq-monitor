# AWS Cloud Sync Setup Guide

This document describes how to set up and use the AWS IoT and DynamoDB cloud
synchronization features.

## Overview

The BBQ Monitor can sync temperature readings to AWS cloud services for:

- Multi-device coordination across distributed instances
- Historical data backup and analytics
- Real-time updates across multiple clients
- Remote monitoring from any location

## Architecture

### Components

1. **AWS IoT Core**: MQTT messaging for real-time temperature updates
2. **DynamoDB**: Persistent storage for temperature readings
3. **Local Database**: SQLite for local caching and offline operation

### Data Flow

```text
Local BLE Device → Local Database → AWS IoT (publish) → Other Clients
                                  ↓
                               DynamoDB (store)
                                  ↓
                            Periodic Sync ← Local Database
```

## Requirements

### Rust Version

AWS SDK requires Rust 1.88 or later. Check your version:

```bash
rustc --version
```

To update Rust:

```bash
rustup update stable
```

### AWS Account Setup

1. **Create IoT Thing**:
   - Go to AWS IoT Core console
   - Create a new Thing (e.g., `bbq-monitor-001`)
   - Download certificates and keys
   - Note the IoT endpoint

2. **Create DynamoDB Table**:

   ```bash
   aws dynamodb create-table \
     --table-name bbq-monitor-readings \
     --attribute-definitions \
       AttributeName=device_address,AttributeType=S \
       AttributeName=timestamp_key,AttributeType=S \
     --key-schema \
       AttributeName=device_address,KeyType=HASH \
       AttributeName=timestamp_key,KeyType=RANGE \
     --billing-mode PAY_PER_REQUEST
   ```

3. **Configure IAM Permissions**:
   - Allow IoT publish/subscribe to `bbq-monitor/*`
   - Allow DynamoDB PutItem, Query, Scan on your table

## Configuration

### 1. Enable AWS Feature

Edit `Cargo.toml` and uncomment AWS dependencies (requires Rust 1.88+):

```toml
# AWS SDK
aws-config = "1.1"
aws-sdk-iot = "1.81"
aws-sdk-iotdataplane = "1.71"
aws-sdk-dynamodb = "1.50"
```

### 2. Build with AWS Support

```bash
cargo build --features aws
```

### 3. Configure AWS Settings

Edit `config.toml`:

```toml
[aws]
# Enable cloud sync
enabled = true

# AWS region (e.g., us-east-1, us-west-2, eu-west-1)
region = "us-east-1"

# IoT Thing name (must match AWS IoT Core)
thing_name = "bbq-monitor-001"

# DynamoDB table name
table_name = "bbq-monitor-readings"

# Sync interval in seconds (300 = 5 minutes)
sync_interval_secs = 300
```

### 4. Set Up AWS Credentials

The application uses standard AWS credential chain. Options:

**Option A: Environment Variables**

```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1
```

**Option B: AWS CLI Configuration**

```bash
aws configure
```

**Option C: IAM Role (EC2/ECS)**

If running on AWS infrastructure, attach an IAM role with appropriate
permissions.

## Usage

### Running with Cloud Sync

```bash
# Build with AWS feature
cargo build --features aws --release

# Run
./target/release/bbq-monitor
```

### Verifying Cloud Sync

Check the logs for:

```text
✅ AWS cloud sync initialized
Synced 5 readings to cloud
Synced 2 readings from cloud
```

### Monitoring

1. **AWS IoT Console**: View MQTT messages in the test client
   - Subscribe to: `bbq-monitor/#`

2. **DynamoDB Console**: Query your readings table
   - View recent entries by device_address

3. **Web Dashboard**: Timestamp aging indicators show data freshness
   - Normal: < 30 seconds (gray)
   - Aged: 30-60 seconds (orange, italic)
   - Stale: > 60 seconds (red, italic)

## Timestamp Aging

The web dashboard shows visual indicators for data freshness:

- **Recent** (< 30s): Normal gray text
- **Aged** (30-60s): Orange italic text
- **Stale** (> 60s): Red italic text

This helps identify:

- Active local BLE readings (always fresh)
- Cloud-synced readings from other clients (may be aged)
- Connection issues (stale readings)

## Troubleshooting

### Build Fails: "rustc 1.87.0 is not supported"

Update to Rust 1.88 or later:

```bash
rustup update stable
rustc --version  # Should show 1.88.0 or higher
```

### Cloud Sync Not Working

1. Check AWS credentials:

   ```bash
   aws sts get-caller-identity
   ```

2. Verify IoT endpoint:

   ```bash
   aws iot describe-endpoint --endpoint-type iot:Data-ATS
   ```

3. Check IAM permissions:
   - IoT: `iot:Publish`, `iot:Subscribe`, `iot:Receive`, `iot:Connect`
   - DynamoDB: `dynamodb:PutItem`, `dynamodb:Query`

4. Check logs for errors:

   ```bash
   tail -f bbq_monitor.log | grep -i error
   ```

### High AWS Costs

1. Reduce sync interval in `config.toml`:

   ```toml
   sync_interval_secs = 600  # 10 minutes instead of 5
   ```

2. Enable DynamoDB TTL to auto-expire old records:

   ```bash
   aws dynamodb update-time-to-live \
     --table-name bbq-monitor-readings \
     --time-to-live-specification \
       "Enabled=true, AttributeName=ttl"
   ```

3. Use On-Demand billing for DynamoDB (already recommended)

## Architecture Decisions

### Why IoT Core + DynamoDB?

- **IoT Core**: Real-time pub/sub for instant updates across clients
- **DynamoDB**: Durable storage, serverless scaling, query capabilities
- **Hybrid**: Local SQLite ensures offline operation and reduces cloud costs

### Sync Strategy

- **Local → Cloud**: Every sync interval (default 5 minutes)
- **Cloud → Local**: Queries last hour of cloud data each interval
- **Conflict Resolution**: Timestamps determine newest reading
- **Deduplication**: Checks for existing readings before inserting

### Cost Optimization

- Local database caches all readings
- Cloud sync only uploads new readings since last sync
- DynamoDB queries limited to recent time range
- Pay-per-request billing scales with actual usage

## Future Enhancements

Potential improvements:

- [ ] WebSocket subscriptions to IoT topics for instant updates
- [ ] Batch uploads to reduce API calls
- [ ] S3 archival for long-term historical data
- [ ] CloudWatch metrics and alarms
- [ ] Multi-region replication
- [ ] Cognito authentication for web dashboard
- [ ] Lambda triggers for alert notifications

## Security Considerations

1. **Never commit credentials** to version control
2. Use **IAM roles** when running on AWS infrastructure
3. Rotate **access keys** regularly
4. Use **certificate-based authentication** for IoT devices
5. Enable **CloudTrail** for audit logging
6. Apply **least-privilege** IAM policies

## Example Scenarios

### Home Setup (Single Location)

- Run one instance locally
- Cloud sync provides backup and remote access
- View dashboard from any device on network

### Multi-Smoker Setup (Backyard Event)

- Run multiple instances (one per smoker)
- Each instance has local BLE connection
- Cloud sync coordinates all readings
- Single dashboard aggregates all devices

### Distributed Monitoring (Multiple Locations)

- Instance per location (home, restaurant, catering)
- Each syncs to shared cloud
- Central monitoring dashboard
- Historical analytics across all locations

## Support

For issues or questions:

1. Check logs: `bbq_monitor.log`
2. Enable debug logging: Set `level = "debug"` in `config.toml`
3. Review AWS CloudWatch logs (if using Lambda/ECS)
4. File issues on GitHub repository
