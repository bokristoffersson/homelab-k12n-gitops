# Backups

What is backed up, how to check and restore it, and how to add a backup for a new
database.

## What is backed up

| Data | Namespace | CronJob | Schedule (UTC) | Format | AWS secret |
|---|---|---|---|---|---|
| TimescaleDB (`telemetry`) | `timescaledb` | `timescaledb-backup` | 02:00 | `pg_dump`, gzip | `timescaledb-backup-aws` |
| Backstage PostgreSQL | `backstage` | `backstage-postgres-backup` | 02:00 | `pg_dumpall`, gzip | `backstage-backup-aws` |
| homelab-settings PostgreSQL | `homelab-settings` | `postgres-backup` | 02:15 | `pg_dump`, gzip | `postgres-backup-aws` |
| gasa PostgreSQL | `gasa` | `postgres-backup` | 02:45 | `pg_dump`, gzip | `postgres-backup-aws` |
| granbok (database + documents) | `granbok` | `granbok-backup` | 03:30 | `pg_dump --format=custom` + document sync | `granbok-backup-aws` |
| etcd | host (`m720q`) | k3s snapshot | 03:00 | etcd snapshot | `k3s-etcd-snapshot-s3-config` |

Not backed up, by design:

- **Authelia** has no database. Its users are a SealedSecret in Git, and session state is
  disposable.
- **Prometheus, Loki and Redpanda** hold data that is either short-lived or already
  persisted in TimescaleDB.

### How the database jobs work

Each job dumps the database, compresses it, uploads it to S3 and then lists the object
to confirm it arrived. The job fails if the dump is empty. The file name carries a
timestamp, for example `telemetry-20261117-020003.sql.gz`.

Bucket, prefix and region come from the job's AWS secret (`S3_BUCKET`, `S3_PREFIX`,
`AWS_REGION`). The four application databases share the bucket `k12n-homelab-db-backups`
in `eu-north-1`:

| Database | Prefix |
|---|---|
| TimescaleDB | `timescaledb-backups/` |
| homelab-settings | `homelab-settings-postgres/` |
| gasa | `gasa/` |
| Backstage | `backstage-postgres-backups/` |

### granbok

granbok is bookkeeping data under a seven-year legal retention requirement, so it has
its own bucket, `k12n-granbok-backups`, with versioning and object lock (Governance mode,
8 years). The backup credentials can write, read and list, but cannot delete.

```
granbok/db/<UTC timestamp>/granbok.dump      # pg_dump, custom format
granbok/db/<UTC timestamp>/completed-at      # written last; marks a complete backup
granbok/documents/<company>/<sha256>         # content-addressed, never rewritten
```

Never restore into or edit the live granbok database or object store by hand. Follow
the granbok repository's own instructions (`deploy/HOMELAB.md` lists its homelab
requirements).

### etcd

k3s takes a snapshot daily, keeps it on `m720q` and uploads it to S3 under
`etcd-snapshots/` (14 retained). Restoring also needs the server token from
`/var/lib/rancher/k3s/server/token`; keep a copy outside the cluster.

## Checking that backups run

```bash
kubectl get cronjobs -A | grep backup          # last schedule and last success
kubectl get jobs -n timescaledb                # recent runs
kubectl logs -n timescaledb job/<job-name>     # output of one run
```

Alerts exist for granbok only: `GranbokBackupFailed`, `GranbokBackupStale` (no success
in 36 hours) and `GranbokBackupNeverSucceeded`. The other jobs have no alert; a failed
run shows up as a failed Job.

### Run a backup now

```bash
kubectl create job backup-manual --from=cronjob/timescaledb-backup -n timescaledb
kubectl logs -n timescaledb job/backup-manual --follow
kubectl delete job backup-manual -n timescaledb
```

Use the namespace and CronJob name from the table for the other databases.

## Restoring

These steps cover TimescaleDB, homelab-settings, gasa and Backstage. A restore
overwrites live data, so stop the services that write to the database first.

1. Find and download the dump:

   ```bash
   aws s3 ls s3://k12n-homelab-db-backups/<prefix>/
   aws s3 cp s3://k12n-homelab-db-backups/<prefix>/<file>.sql.gz .
   gunzip <file>.sql.gz
   ```

2. Forward the database port:

   | Database | Command |
   |---|---|
   | TimescaleDB | `kubectl port-forward -n timescaledb svc/timescaledb 5432:5432` |
   | homelab-settings | `kubectl port-forward -n homelab-settings svc/postgres 5432:5432` |
   | gasa | `kubectl port-forward -n gasa svc/postgres 5432:5432` |
   | Backstage | `kubectl port-forward -n backstage svc/backstage-postgres 5432:5432` |

3. Load the dump from another terminal. The user and password are in the database's
   credentials secret in the same namespace.

   ```bash
   psql -h localhost -U <user> -d <database> < <file>.sql
   ```

   The Backstage dump is a `pg_dumpall` of every database and role; load it with
   `-d postgres`.

4. Restart the services that use the database.

## Adding a backup for a new database

1. **Pick a prefix** in `k12n-homelab-db-backups`, or create a bucket with versioning
   and default encryption enabled.

2. **Create an IAM user** whose policy is limited to that bucket:

   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": ["s3:PutObject", "s3:GetObject", "s3:ListBucket"],
         "Resource": [
           "arn:aws:s3:::k12n-homelab-db-backups",
           "arn:aws:s3:::k12n-homelab-db-backups/*"
         ]
       }
     ]
   }
   ```

   The job never deletes. Let an S3 lifecycle rule expire old dumps instead of granting
   `s3:DeleteObject`.

3. **Seal the credentials** into the app's directory:

   ```bash
   kubectl create secret generic postgres-backup-aws \
     --namespace=<namespace> \
     --from-literal=AWS_ACCESS_KEY_ID='...' \
     --from-literal=AWS_SECRET_ACCESS_KEY='...' \
     --from-literal=AWS_REGION='eu-north-1' \
     --from-literal=S3_BUCKET='k12n-homelab-db-backups' \
     --from-literal=S3_PREFIX='<app>-backups' \
     --dry-run=client -o yaml | \
   kubeseal \
     --namespace <namespace> \
     --controller-name sealed-secrets \
     --controller-namespace kube-system \
     --format=yaml > gitops/apps/<app>/backup-aws-credentials-sealed.yaml
   ```

4. **Add the CronJob.** Copy `gitops/apps/gasa/backup-cronjob.yaml`: it dumps in a
   `postgres` init container and uploads from an `amazon/aws-cli` container through a
   shared `emptyDir`, so neither image needs extra packages. Pick a schedule that does
   not collide with the ones above.

5. **List both files** in the app's `kustomization.yaml`, open a PR, and after the merge
   run the job once by hand to confirm the upload.

## Troubleshooting

| Symptom | Check |
|---|---|
| Job fails at the dump | Database credentials secret, and that the database Service is reachable from the job |
| Job fails at the upload | Keys in the AWS secret, the IAM policy, and the bucket region |
| Job succeeds but the file is tiny | The dump step guards against empty output; compare with the previous day's size in S3 |
| No job was created | `kubectl describe cronjob <name> -n <namespace>` for a suspended CronJob or a missed schedule |
