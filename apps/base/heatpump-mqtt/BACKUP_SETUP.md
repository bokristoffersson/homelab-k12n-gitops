# Database Backup Setup Guide

This guide explains how to set up automated backups for the TimescaleDB database to AWS S3.

## Quick Start

1. **Create AWS S3 bucket and IAM user** (see AWS Setup below)
2. **Edit and seal the AWS secret:**
   ```bash
   # Create the secret file with your credentials
   cat > backup-aws-secret.yaml << EOF
   apiVersion: v1
   kind: Secret
   metadata:
     name: timescaledb-backup-aws
     namespace: heatpump-mqtt
   type: Opaque
   stringData:
     AWS_ACCESS_KEY_ID: your-access-key-id
     AWS_SECRET_ACCESS_KEY: your-secret-access-key
     AWS_REGION: eu-north-1
     S3_BUCKET: k12n-homelab-db-backups
     S3_PREFIX: heatpump-mqtt-backups/
   EOF
   
   # Seal the secret
   kubectl create secret generic timescaledb-backup-aws \
     --from-file=backup-aws-secret.yaml \
     --dry-run=client \
     -o yaml | kubeseal -o yaml > apps/base/heatpump-mqtt/backup-aws-secret-sealed.yaml
   
   # Clean up the plaintext file
   rm backup-aws-secret.yaml
   ```
3. **Apply the resources:**
   ```bash
   kubectl apply -k apps/base/heatpump-mqtt/
   ```
4. **Verify backups are working:**
   ```bash
   kubectl get cronjob -n heatpump-mqtt
   kubectl get jobs -n heatpump-mqtt | grep backup
   ```

## Overview

The backup solution consists of:
- **Automated daily backups** via CronJob (runs at 2 AM UTC)
- **S3 storage** with encryption for backup retention
- **S3 lifecycle policies** for automatic cleanup
- **Compression** (gzip) to reduce storage costs
- **Manual restore job** for database restores
- **Best practices** for security, encryption, and cost optimization

## AWS Setup

### 1. Create S3 Bucket

```bash
# Create an S3 bucket for backups
aws s3 mb s3://your-homelab-db-backups --region eu-north-1

# Enable versioning (recommended)
aws s3api put-bucket-versioning \
  --bucket your-homelab-db-backups \
  --versioning-configuration Status=Enabled

# Enable server-side encryption
aws s3api put-bucket-encryption \
  --bucket your-homelab-db-backups \
  --server-side-encryption-configuration '{
    "Rules": [{
      "ApplyServerSideEncryptionByDefault": {
        "SSEAlgorithm": "AES256"
      }
    }]
  }'

# Enable lifecycle policy for cost optimization
cat > lifecycle.json << EOF
{
  "Rules": [{
    "ID": "DeleteOldBackups",
    "Status": "Enabled",
    "Filter": {},
    "Expiration": {
      "Days": 90
    }
  }]
}
EOF

aws s3api put-bucket-lifecycle-configuration \
  --bucket your-homelab-db-backups \
  --lifecycle-configuration file://lifecycle.json
```

### 2. Create IAM User and Policy

```bash
# Create IAM user for backups
aws iam create-user --user-name timescaledb-backup-user

# Create policy
cat > backup-policy.json << EOF
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": [
      "s3:PutObject",
      "s3:GetObject",
      "s3:DeleteObject",
      "s3:ListBucket"
    ],
    "Resource": [
      "arn:aws:s3:::your-homelab-db-backups",
      "arn:aws:s3:::your-homelab-db-backups/*"
    ]
  }]
}
EOF

aws iam create-policy \
  --policy-name TimescaleDBBackupPolicy \
  --policy-document file://backup-policy.json

# Attach policy to user
aws iam attach-user-policy \
  --user-name timescaledb-backup-user \
  --policy-arn arn:aws:iam::YOUR_ACCOUNT_ID:policy/TimescaleDBBackupPolicy

# Create access keys
aws iam create-access-key --user-name timescaledb-backup-user
```

**Save the Access Key ID and Secret Access Key** - you'll need them for the next step.

## Kubernetes Setup

### 1. Seal the AWS Secret

Edit `backup-aws-secret.yaml` with your AWS credentials:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: timescaledb-backup-aws
  namespace: heatpump-mqtt
type: Opaque
stringData:
  AWS_ACCESS_KEY_ID: AKIAIOSFODNN7EXAMPLE
  AWS_SECRET_ACCESS_KEY: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
  AWS_REGION: us-east-1
  S3_BUCKET: your-homelab-db-backups
  S3_PREFIX: heatpump-mqtt-backups/
```

Then seal it using the sealed-secrets controller:

```bash
# Install kubeseal if not already installed
brew install kubeseal  # macOS
# or
curl -L -o kubeseal https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.26.0/kubeseal-linux-amd64

# Seal the secret
kubectl create secret generic timescaledb-backup-aws \
  --from-file=backup-aws-secret.yaml \
  --dry-run=client \
  -o yaml | kubeseal -o yaml > backup-aws-secret-sealed.yaml

# Move to appropriate location
mv backup-aws-secret-sealed.yaml apps/base/heatpump-mqtt/
```

### 2. Apply the Resources

The kustomization.yaml already includes the backup resources. Apply them:

```bash
# From the repository root
kubectl apply -k apps/base/heatpump-mqtt/
```

Or if using Flux:

```bash
# Commit and push changes
git add apps/base/heatpump-mqtt/backup-*.yaml
git add apps/base/heatpump-mqtt/kustomization.yaml
git commit -m "Add database backup configuration"
git push
```

### 3. Verify Backups

Check the CronJob status:

```bash
kubectl get cronjob timescaledb-backup -n heatpump-mqtt
```

List recent backups in S3:

```bash
aws s3 ls s3://your-homelab-db-backups/heatpump-mqtt-backups/
```

Check backup job logs:

```bash
# List recent backup jobs
kubectl get jobs -n heatpump-mqtt | grep backup

# Check logs of last backup
kubectl logs -n heatpump-mqtt job/timescaledb-backup-<timestamp> --tail=50
```

## Restore from Backup

> **Note:** The restore job requires PostgreSQL superuser privileges to drop and recreate the database. Ensure that the `timescaledb-secret` contains credentials for the `postgres` superuser (POSTGRES_USER=postgres and POSTGRES_PASSWORD).

### Manual Restore

To restore from a backup:

1. **List available backups:**
   ```bash
   aws s3 ls s3://your-homelab-db-backups/heatpump-mqtt-backups/ | cat
   ```

2. **Get the backup filename you want to restore from**

3. **Create a job manifest with the backup file:**
   ```bash
   # Edit the file and set BACKUP_FILE in the env section
   kubectl create -f apps/base/heatpump-mqtt/backup-restore-job.yaml
   
   # Or create the job directly with the backup file:
   BACKUP_FILE="heatpump-backup-20241215-020000.sql.gz" \
     kubectl create job timescaledb-restore-$(date +%s) \
     --from=job/timescaledb-restore \
     -n heatpump-mqtt
   ```

4. **Set the BACKUP_FILE environment variable:**
   ```bash
   kubectl set env job/timescaledb-restore-<timestamp> BACKUP_FILE=heatpump-backup-20241215-020000.sql.gz -n heatpump-mqtt
   ```

5. **Monitor the restore:**
   ```bash
   kubectl logs -f job/timescaledb-restore-<timestamp> -n heatpump-mqtt
   ```

### Quick Restore Commands

```bash
# Restore specific backup
kubectl create job --from=cronjob/timescaledb-backup restore-test -n heatpump-mqtt

# Or create a one-time restore with specific backup file
BACKUP_FILE="heatpump-backup-20241215-020000.sql.gz" \
envsubst < backup-restore-job.yaml | kubectl apply -f -
```

## Best Practices

### Security

1. **Use IAM roles with least privilege** - The IAM user should only have access to the backup bucket
2. **Enable S3 bucket encryption** - All backups are encrypted at rest
3. **Use sealed secrets** - Keep AWS credentials encrypted in Git
4. **Rotate access keys** - Periodically rotate AWS access keys
5. **Enable S3 bucket versioning** - Helps recover from accidental deletes

### Backup Configuration

1. **Schedule:** Currently set to 2 AM UTC daily. Adjust as needed:
   ```yaml
   schedule: "0 2 * * *"  # Cron format (minute hour day month day-of-week)
   ```

2. **Retention:** Currently 30 days. Adjust in the CronJob script or use S3 lifecycle policies

3. **Compression:** Backups are gzip compressed to save storage costs

4. **Monitoring:** Set up alerts for failed backup jobs:
   ```bash
   # Check for failed backups
   kubectl get jobs -n heatpump-mqtt | grep backup | grep -v Completed
   ```

### Cost Optimization

1. **S3 Storage Classes:** Use lifecycle policies to move old backups to cheaper storage:
   ```json
   {
     "Rules": [{
       "ID": "TransitionToIA",
       "Status": "Enabled",
       "Filter": {},
       "Transitions": [{
         "Days": 30,
         "StorageClass": "STANDARD_IA"
       }]
     }]
   }
   ```

2. **Lifecycle Cleanup:** Set up automatic deletion of backups older than your retention period

3. **Compression:** Already enabled via gzip

## Troubleshooting

### Backup Fails with "Access Denied"

- Check IAM permissions
- Verify AWS credentials in the secret
- Ensure bucket name and region are correct

### Backup Fails with "Connection Refused"

- Verify timescaledb service is running
- Check network policies
- Ensure database credentials are correct

### Large Backups Timeout

- Increase resource limits in CronJob
- Consider splitting backups by table
- Adjust database dump options (`pg_dump` flags)

## Testing

### Test Backup Locally

```bash
# Run a manual backup
kubectl create job --from=cronjob/timescaledb-backup test-backup -n heatpump-mqtt

# Check logs
kubectl logs job/test-backup -n heatpump-mqtt

# Verify in S3
aws s3 ls s3://your-homelab-db-backups/heatpump-mqtt-backups/
```

### Test Restore on Staging

1. Create a test database
2. Run backup
3. Modify test database
4. Restore from backup
5. Verify data is restored correctly

## Maintenance

### Update Backup Schedule

Edit `apps/base/heatpump-mqtt/backup-cronjob.yaml`:

```yaml
spec:
  schedule: "0 3 * * *"  # Change time or frequency
```

### Update Retention Period

Modify the cleanup section in the CronJob script to change the retention period (currently 30 days).

### Monitor Disk Space

Keep an eye on S3 bucket size:

```bash
aws s3 ls s3://your-homelab-db-backups/heatpump-mqtt-backups/ --recursive --summarize
```

