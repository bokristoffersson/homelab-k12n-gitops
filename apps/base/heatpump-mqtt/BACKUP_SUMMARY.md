# Database Backup Summary

## What Was Created

### Files Added
- `backup-aws-secret.yaml` - AWS credentials secret template (needs to be sealed)
- `backup-cronjob.yaml` - Automated daily backup job
- `backup-restore-job.yaml` - Manual restore job
- `BACKUP_SETUP.md` - Comprehensive setup guide
- `BACKUP_SUMMARY.md` - This file

### Files Modified
- `kustomization.yaml` - Added backup resources to the kustomization

## Key Features

✅ **Automated Daily Backups**
- Runs daily at 2 AM UTC
- Uses `pg_dump` with compression (gzip)
- Uploads to AWS S3
- Automatic cleanup of old backups

✅ **S3 Best Practices**
- Server-side encryption
- Versioning enabled
- Lifecycle policies for cost optimization
- Secure IAM credentials

✅ **Restore Capability**
- Manual restore job for database recovery
- Supports restoring to any backup point
- Graceful connection handling

✅ **Security**
- Secrets stored using sealed-secrets
- Encrypted credentials
- Least-privilege IAM access
- S3 bucket encryption

## Next Steps

1. **Review `BACKUP_SETUP.md`** for detailed setup instructions
2. **Create AWS S3 bucket and IAM user** (detailed in BACKUP_SETUP.md)
3. **Seal the AWS secret** using kubeseal
4. **Apply the configuration** with `kubectl apply -k apps/base/heatpump-mqtt/`
5. **Verify backups** are running successfully

## Quick Command Reference

```bash
# Check backup status
kubectl get cronjob -n heatpump-mqtt
kubectl get jobs -n heatpump-mqtt

# View backup logs
kubectl logs job/timescaledb-backup-<timestamp> -n heatpump-mqtt

# List backups in S3
aws s3 ls s3://your-bucket/heatpump-mqtt-backups/

# Create a manual restore job
kubectl create job timescaledb-restore-$(date +%s) \
  --from=job/timescaledb-restore \
  -n heatpump-mqtt
```

## Important Notes

- Backups use the database name `timescaledb` (from configmap)
- Restore requires PostgreSQL superuser privileges
- Backup schedule can be adjusted in `backup-cronjob.yaml`
- S3 lifecycle policies handle cleanup (90-day retention default)
- All sensitive credentials are encrypted using sealed-secrets

