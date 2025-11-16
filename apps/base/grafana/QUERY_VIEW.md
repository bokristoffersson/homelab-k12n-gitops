# Query vy_in_tomodell_customer View

## Check if the view exists and what it contains

Run these queries in Grafana Explore or directly in the database:

### 1. Check if the view exists
```sql
SELECT 
  table_schema,
  table_name,
  view_definition
FROM information_schema.views 
WHERE table_name LIKE '%tomodell_customer%';
```

### 2. Get the view definition (what columns it has)
```sql
SELECT 
  column_name,
  data_type,
  is_nullable
FROM information_schema.columns
WHERE table_name = 'vy_in_tomodell_customer'
ORDER BY ordinal_position;
```

### 3. See what data the view contains (sample)
```sql
SELECT * 
FROM vy_in_tomodell_customer 
LIMIT 10;
```

### 4. Get row count
```sql
SELECT COUNT(*) as total_rows
FROM vy_in_tomodell_customer;
```

## Run via kubectl

```bash
# Get database credentials first
DB_USER=$(kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)

# Check view definition
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U $DB_USER -d timescaledb -c "
SELECT 
  column_name,
  data_type,
  is_nullable
FROM information_schema.columns
WHERE table_name = 'vy_in_tomodell_customer'
ORDER BY ordinal_position;
"

# See sample data
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U $DB_USER -d timescaledb -c "
SELECT * FROM vy_in_tomodell_customer LIMIT 10;
"
```

## Get the actual SQL definition

```bash
DB_USER=$(kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)

kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U $DB_USER -d timescaledb -c "
SELECT pg_get_viewdef('vy_in_tomodell_customer', true);
"
```



