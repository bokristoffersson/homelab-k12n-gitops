# Resetting Admin Password

The admin password is stored as a bcrypt hash in a sealed secret. **You cannot retrieve the original plaintext password** - that's the security feature of hashing. However, you can reset it to a new password.

## Option 1: Reset Password (Recommended)

### Step 1: Generate a New Password Hash

You need to create a bcrypt hash of your new password. Here are several methods:

#### Method A: Using the Application Code (Recommended)

Since you have the application code, you can use it to generate the hash:

```bash
cd applications/redpanda-sink

# Create a temporary binary to hash passwords
cat > src/bin/hash_password.rs <<'EOF'
use redpanda_sink::auth::password::hash_password;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: hash-password <password>");
        std::process::exit(1);
    }
    
    match hash_password(&args[1]) {
        Ok(hash) => println!("{}", hash),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
EOF

# Add binary to Cargo.toml (if not already there)
grep -q "[[bin]]" Cargo.toml || cat >> Cargo.toml <<'EOF'

[[bin]]
name = "hash-password"
path = "src/bin/hash_password.rs"
EOF

# Build and run
cargo build --bin hash-password --release
./target/release/hash-password "your-new-password"
```

#### Method B: Using a Temporary Cargo Project

```bash
cd /tmp
cargo new --bin hash-password
cd hash-password

# Add bcrypt dependency
echo 'bcrypt = "0.15"' >> Cargo.toml

# Create the hashing script
cat > src/main.rs <<'EOF'
fn main() {
    let password = std::env::args().nth(1).expect("Usage: hash-password <password>");
    println!("{}", bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap());
}
EOF

# Build and run
cargo build --release
./target/release/hash-password "your-new-password"
```

#### Method C: Using Docker (if you have the image)

```bash
# Run the hash-password binary from the Docker image
docker run --rm ghcr.io/bokristoffersson/redpanda-sink:latest \
  hash-password "your-new-password"
```

### Step 2: Generate JWT Secret (if needed)

If you also need to regenerate the JWT secret:

```bash
# Generate a new JWT secret
JWT_SECRET=$(openssl rand -hex 32)
echo "JWT_SECRET: $JWT_SECRET"
```

### Step 3: Create Temporary Secret File

Create a temporary secret file `auth-secret.yaml`:

```bash
cat > /tmp/auth-secret.yaml <<'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: auth-secret
  namespace: redpanda-sink
type: Opaque
stringData:
  JWT_SECRET: "${JWT_SECRET:-$(openssl rand -hex 32)}"
  ADMIN_PASSWORD_HASH: "PASTE_YOUR_HASH_HERE"
EOF
```

Replace `PASTE_YOUR_HASH_HERE` with the hash from Step 1.

### Step 4: Seal the Secret

Use `kubeseal` to encrypt the secret:

```bash
# Make sure kubeseal is installed and configured
kubeseal -f /tmp/auth-secret.yaml -o /tmp/auth-secret-sealed.yaml

# Review the sealed secret
cat /tmp/auth-secret-sealed.yaml
```

### Step 5: Update the Sealed Secret

Replace the content in `gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml`:

```bash
# Backup the old secret
cp gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml \
   gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml.backup

# Copy the new sealed secret
cp /tmp/auth-secret-sealed.yaml \
   gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml
```

### Step 6: Apply the New Secret

```bash
# Apply the new sealed secret
kubectl apply -f gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml

# Restart the deployment to pick up the new secret
kubectl rollout restart deployment/redpanda-sink -n redpanda-sink

# Wait for rollout
kubectl rollout status deployment/redpanda-sink -n redpanda-sink
```

### Step 7: Test the New Password

```bash
# Port-forward to access the API
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080

# In another terminal, test login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-new-password"}'
```

## Option 2: Check Current Hash (For Reference Only)

You can view the current hash (though it won't help you recover the password):

```bash
# Get the current password hash from the secret
kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d
echo ""

# This will show you the bcrypt hash, but you cannot reverse it to get the password
```

## Option 3: Quick Reset Script

Save this as `reset-password.sh`:

```bash
#!/bin/bash

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <new-password>"
    exit 1
fi

NEW_PASSWORD="$1"
NAMESPACE="redpanda-sink"
SECRET_NAME="auth-secret"

echo "=== Generating Password Hash ==="
cd /tmp
cargo new --bin hash-password 2>/dev/null || true
cd hash-password

if ! grep -q "bcrypt" Cargo.toml; then
    echo 'bcrypt = "0.15"' >> Cargo.toml
fi

cat > src/main.rs <<'EOF'
fn main() {
    let password = std::env::args().nth(1).expect("Usage: hash-password <password>");
    println!("{}", bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap());
}
EOF

cargo build --release --quiet
PASSWORD_HASH=$(./target/release/hash-password "$NEW_PASSWORD")

echo "Password hash generated: ${PASSWORD_HASH:0:20}..."

echo -e "\n=== Generating JWT Secret ==="
JWT_SECRET=$(openssl rand -hex 32)
echo "JWT secret generated"

echo -e "\n=== Creating Temporary Secret ==="
printf 'apiVersion: v1
kind: Secret
metadata:
  name: %s
  namespace: %s
type: Opaque
stringData:
  JWT_SECRET: "%s"
  ADMIN_PASSWORD_HASH: "%s"
' "$SECRET_NAME" "$NAMESPACE" "$JWT_SECRET" "$PASSWORD_HASH" > /tmp/auth-secret.yaml

echo -e "\n=== Sealing Secret ==="
kubeseal -f /tmp/auth-secret.yaml -o /tmp/auth-secret-sealed.yaml

echo -e "\n=== Review Sealed Secret ==="
cat /tmp/auth-secret-sealed.yaml

echo -e "\n=== Next Steps ==="
echo "1. Review the sealed secret above"
echo "2. Copy it to: gitops/apps/base/redpanda-sink/auth-secret-sealed.yaml"
echo "3. Commit and push, or apply directly:"
echo "   kubectl apply -f /tmp/auth-secret-sealed.yaml"
echo "4. Restart the deployment:"
echo "   kubectl rollout restart deployment/redpanda-sink -n $NAMESPACE"
echo ""
echo "Your new password is: $NEW_PASSWORD"
echo "⚠️  Save this password securely - you cannot retrieve it later!"
```

Make it executable and run:
```bash
chmod +x reset-password.sh
./reset-password.sh "my-new-secure-password"
```

## Security Notes

1. **Never commit plaintext passwords** - Always use sealed secrets
2. **Save your password securely** - You cannot retrieve it from the hash
3. **Use a password manager** - Store the password in a secure password manager
4. **Rotate passwords regularly** - Consider setting up password rotation

## Troubleshooting

### kubeseal Not Available

Install kubeseal:
```bash
# macOS
brew install kubeseal

# Linux
wget https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/kubeseal-0.24.0-linux-amd64.tar.gz
tar -xvzf kubeseal-0.24.0-linux-amd64.tar.gz kubeseal
sudo install -m 755 kubeseal /usr/local/bin/kubeseal
```

### Secret Not Updating

1. **Check if sealed secret controller is running:**
   ```bash
   kubectl get pods -n kube-system | grep sealed-secrets
   ```

2. **Force reconciliation:**
   ```bash
   kubectl delete secret auth-secret -n redpanda-sink
   # The sealed-secrets controller will recreate it from the SealedSecret
   ```

3. **Restart the deployment:**
   ```bash
   kubectl rollout restart deployment/redpanda-sink -n redpanda-sink
   ```

### Password Not Working After Reset

1. **Verify the hash was set correctly:**
   ```bash
   kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.ADMIN_PASSWORD_HASH}' | base64 -d
   ```

2. **Check application logs:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "auth\|login"
   ```

3. **Verify the username is correct:**
   ```bash
   # Check the configmap for the username
   kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep username
   ```
