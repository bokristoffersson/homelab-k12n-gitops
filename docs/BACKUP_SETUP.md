# Database Backup Setup Guide

This guide covers setting up automated database backups to AWS S3 for both TimescaleDB and PostgreSQL databases.

## Overview

- **TimescaleDB**: Daily backups at 2:00 AM
- **Heatpump Settings (PostgreSQL)**: Daily backups at 2:15 AM
- **Storage**: AWS S3 with lifecycle policies
- **Format**: Compressed SQL dumps (gzip)
- **Retention**: Configurable via S3 lifecycle policies

## Prerequisites

- AWS account with S3 access
- AWS CLI installed locally (for initial setup)
- `kubectl` access to your cluster
- `kubeseal` installed for creating sealed secrets

## Step 1: Create AWS S3 Bucket

### Using AWS Console

1. Navigate to AWS S3 Console
2. Click "Create bucket"
3. Configure:
   - **Bucket name**: `k12n-homelab-db-backups` (or your preferred name)
   - **Region**: Choose your preferred region (e.g., `eu-north-1`)
   - **Block Public Access**: Keep all boxes checked (block all public access)
   - **Versioning**: Optional, recommended for safety
   - **Encryption**: Enable server-side encryption (SSE-S3)
4. Click "Create bucket"

### Using AWS CLI

```bash
# Set your preferred region and bucket name
export AWS_REGION="eu-north-1"
export BUCKET_NAME="k12n-homelab-db-backups"

# Create the bucket
aws s3 mb s3://${BUCKET_NAME} --region ${AWS_REGION}

# Enable versioning (optional but recommended)
aws s3api put-bucket-versioning \
  --bucket ${BUCKET_NAME} \
  --versioning-configuration Status=Enabled

# Enable encryption
aws s3api put-bucket-encryption \
  --bucket ${BUCKET_NAME} \
  --server-side-encryption-configuration '{
    "Rules": [{
      "ApplyServerSideEncryptionByDefault": {
        "SSEAlgorithm": "AES256"
      },
      "BucketKeyEnabled": true
    }]
  }'
```

## Step 2: Create IAM User and Policy

### Create IAM Policy

Create a policy file `backup-policy.json`:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowBackupOperations",
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:ListBucket",
        "s3:DeleteObject"
      ],
      "Resource": [
        "arn:aws:s3:::k12n-homelab-db-backups",
        "arn:aws:s3:::k12n-homelab-db-backups/*"
      ]
    }
  ]
}
```

### Create IAM User

Using AWS Console:
1. Navigate to IAM → Users → Create user
2. User name: `homelab-db-backup`
3. Attach the policy created above
4. Create access key for "Application running outside AWS"
5. Save the Access Key ID and Secret Access Key

Using AWS CLI:

```bash
# Create IAM user
aws iam create-user --user-name homelab-db-backup

# Create policy
aws iam create-policy \
  --policy-name homelab-db-backup-policy \
  --policy-document file://backup-policy.json

# Attach policy to user (replace ACCOUNT_ID with your AWS account ID)
aws iam attach-user-policy \
  --user-name homelab-db-backup \
  --policy-arn arn:aws:iam::ACCOUNT_ID:policy/homelab-db-backup-policy

# Create access key
aws iam create-access-key --user-name homelab-db-backup
```

Save the Access Key ID and Secret Access Key from the output.

## Step 3: Configure S3 Lifecycle Policy

Configure automatic deletion of old backups to manage costs.

### Example: Keep 30 Days of Backups

Using AWS Console:
1. Navigate to S3 → Your bucket → Management tab
2. Create lifecycle rule:
   - **Name**: `delete-old-backups`
   - **Scope**: Apply to all objects
   - **Lifecycle rule actions**:
     - ✅ Expire current versions of objects
   - **Days after object creation**: 30
3. Save

Using AWS CLI:

Create `lifecycle-policy.json`:

```json
{
  "Rules": [
    {
      "Id": "DeleteOldBackups",
      "Status": "Enabled",
      "Expiration": {
        "Days": 30
      },
      "Filter": {}
    }
  ]
}
```

Apply it:

```bash
aws s3api put-bucket-lifecycle-configuration \
  --bucket k12n-homelab-db-backups \
  --lifecycle-configuration file://lifecycle-policy.json
```

### Alternative: Transition to Cheaper Storage

For longer retention with cost optimization:

```json
{
  "Rules": [
    {
      "Id": "TransitionAndDelete",
      "Status": "Enabled",
      "Transitions": [
        {
          "Days": 30,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 90,
          "StorageClass": "GLACIER_IR"
        }
      ],
      "Expiration": {
        "Days": 365
      },
      "Filter": {}
    }
  ]
}
```

This policy:
- Keeps daily backups in S3 Standard for 30 days
- Moves to S3 Infrequent Access (cheaper) after 30 days
- Moves to Glacier Instant Retrieval after 90 days
- Deletes after 1 year

## Step 4: Create Kubernetes Sealed Secrets

### For TimescaleDB

```bash
# Replace with your actual AWS credentials
kubectl create secret generic timescaledb-backup-aws \
  --namespace=timescaledb \
  --from-literal=AWS_ACCESS_KEY_ID='YOUR_ACCESS_KEY_ID' \
  --from-literal=AWS_SECRET_ACCESS_KEY='YOUR_SECRET_ACCESS_KEY' \
  --from-literal=AWS_REGION='eu-north-1' \
  --from-literal=S3_BUCKET='k12n-homelab-db-backups' \
  --from-literal=S3_PREFIX='timescaledb-backups' \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace timescaledb \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/timescaledb/backup-aws-credentials-sealed.yaml
```

### For Heatpump-Settings PostgreSQL

```bash
# Replace with your actual AWS credentials
kubectl create secret generic postgres-backup-aws \
  --namespace=heatpump-settings \
  --from-literal=AWS_ACCESS_KEY_ID='YOUR_ACCESS_KEY_ID' \
  --from-literal=AWS_SECRET_ACCESS_KEY='YOUR_SECRET_ACCESS_KEY' \
  --from-literal=AWS_REGION='eu-north-1' \
  --from-literal=S3_BUCKET='k12n-homelab-db-backups' \
  --from-literal=S3_PREFIX='heatpump-settings-backups' \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace heatpump-settings \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/heatpump-settings/backup-aws-credentials-sealed.yaml
```

## Step 5: Deploy to Cluster

The backup CronJobs are already configured in the GitOps repository. Once you've created the sealed secrets:

```bash
# Commit the sealed secrets
git add gitops/apps/base/*/backup-aws-credentials-sealed.yaml
git commit -m "feat: add S3 backup credentials"
git push

# Force Flux to reconcile
flux reconcile kustomization timescaledb --with-source
flux reconcile kustomization heatpump-settings --with-source
```

## Step 6: Test Backups

### Manual Test Run

```bash
# Trigger TimescaleDB backup manually
kubectl create job timescaledb-backup-test \
  --from=cronjob/timescaledb-backup \
  -n timescaledb

# Trigger heatpump-settings backup manually
kubectl create job postgres-backup-test \
  --from=cronjob/postgres-backup \
  -n heatpump-settings

# Watch the backup progress
kubectl logs -n timescaledb -l job-name=timescaledb-backup-test --follow
kubectl logs -n heatpump-settings -l job-name=postgres-backup-test --follow
```

### Verify in S3

```bash
# List TimescaleDB backups
aws s3 ls s3://k12n-homelab-db-backups/timescaledb-backups/

# List heatpump-settings backups
aws s3 ls s3://k12n-homelab-db-backups/heatpump-settings-backups/
```

### Clean Up Test Jobs

```bash
kubectl delete job timescaledb-backup-test -n timescaledb
kubectl delete job postgres-backup-test -n heatpump-settings
```

## Backup Schedule

| Database | CronJob Name | Schedule | Namespace |
|----------|-------------|----------|-----------|
| TimescaleDB (telemetry) | `timescaledb-backup` | `0 2 * * *` (2:00 AM daily) | `timescaledb` |
| PostgreSQL (heatpump_settings) | `postgres-backup` | `15 2 * * *` (2:15 AM daily) | `heatpump-settings` |

## Monitoring Backups

### Check CronJob Status

```bash
# View all backup CronJobs
kubectl get cronjob -A | grep backup

# Check last run
kubectl get jobs -n timescaledb -l app=timescaledb-backup
kubectl get jobs -n heatpump-settings -l app=postgres-backup

# View recent logs
kubectl logs -n timescaledb -l app=timescaledb-backup --tail=50
```

### Setup Alerts (Optional)

Consider setting up alerts for:
- Failed backup jobs (exit code != 0)
- Missing backups (no new files in S3 after 25 hours)
- S3 upload failures

You can use:
- Kubernetes events monitoring
- AWS CloudWatch alarms on S3 PUT operations
- Prometheus/Alertmanager rules

## Restore Process

### Download Backup from S3

```bash
# List available backups
aws s3 ls s3://k12n-homelab-db-backups/timescaledb-backups/

# Download a specific backup
aws s3 cp \
  s3://k12n-homelab-db-backups/timescaledb-backups/telemetry-20251226-110323.sql.gz \
  ./telemetry-backup.sql.gz
```

### Restore to Database

For TimescaleDB:

```bash
# Extract backup
gunzip telemetry-backup.sql.gz

# Port-forward to database
kubectl port-forward -n timescaledb svc/timescaledb 5432:5432

# Restore (in another terminal)
# WARNING: This will overwrite existing data
psql -h localhost -U postgres -d telemetry < telemetry-backup.sql
```

For heatpump-settings:

```bash
# Extract backup
gunzip heatpump_settings-backup.sql.gz

# Port-forward to database
kubectl port-forward -n heatpump-settings svc/postgres 5432:5432

# Restore (in another terminal)
psql -h localhost -U postgres -d heatpump_settings < heatpump_settings-backup.sql
```

## Troubleshooting

### Backup Job Fails

Check the logs:
```bash
kubectl logs -n timescaledb -l app=timescaledb-backup --tail=100
```

Common issues:
- AWS credentials invalid: Check sealed secret and IAM user
- S3 bucket doesn't exist: Verify bucket name and region
- Network issues: Check cluster internet connectivity
- Database connection failed: Verify database service name and port

### AWS CLI Installation in Pod Fails

The backup job installs AWS CLI at runtime. If this fails:
- Check container image supports `apt-get` (postgres:16)
- Verify internet connectivity from pods
- Consider pre-building a custom image with AWS CLI

### Backup Size Too Large

If backups are getting large:
- Consider using `pg_dump --no-owner --no-acl` flags
- Use `--compress=9` for better compression
- Implement incremental backups (requires WAL archiving)
- Review data retention in database (e.g., TimescaleDB retention policies)

## Security Best Practices

1. **Rotate IAM credentials regularly** (every 90 days)
2. **Use IAM user specifically for backups** (principle of least privilege)
3. **Enable S3 bucket versioning** for protection against accidental deletes
4. **Enable S3 server-side encryption**
5. **Monitor S3 access logs** for suspicious activity
6. **Never commit unsealed secrets** to Git
7. **Test restore process periodically** (quarterly recommended)

## Cost Optimization

Estimated costs (eu-north-1, as of 2024):

- **S3 Standard**: ~$0.023/GB/month
- **S3 IA**: ~$0.0125/GB/month (after 30 days)
- **Glacier IR**: ~$0.004/GB/month (after 90 days)

For ~10GB of daily backups with 30-day retention:
- Month 1-30: ~300GB × $0.023 = ~$6.90/month
- With transitions: ~$3-4/month

Tips:
- Use lifecycle policies to transition old backups
- Compress backups (already done with gzip)
- Review and prune old test data
- Consider longer intervals for large, rarely-changing databases

## References

- [AWS S3 Lifecycle Policies](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lifecycle-mgmt.html)
- [PostgreSQL pg_dump Documentation](https://www.postgresql.org/docs/current/app-pgdump.html)
- [TimescaleDB Backup Best Practices](https://docs.timescale.com/self-hosted/latest/backup-and-restore/)
- [Sealed Secrets Documentation](https://sealed-secrets.netlify.app/)
