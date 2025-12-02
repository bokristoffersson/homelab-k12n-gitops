# Cloudflare API Token Permissions Required

This document specifies the exact permissions needed for the Cloudflare API token used by the `setup-cloudflare-access` Job.

## Required Permissions

When creating your Cloudflare API token, you need the following permissions:

### Account Permissions

1. **Cloudflare Access:Apps** - **Edit**
   - Required to create and manage Access Applications
   - Used to create/update the `heatpump-api` Access Application

2. **Cloudflare Access:Service Tokens** - **Edit**
   - Required to create and manage Service Tokens
   - Used to create Service Tokens for API authentication

3. **Cloudflare Access:Policies** - **Edit**
   - Required to create and manage Access Policies
   - Used to create policies that allow Service Token access

4. **Zone:Zone Settings** - **Read** (optional, for zone lookup)
   - Used to get Zone ID from zone name
   - Can be replaced with hardcoded Zone ID if preferred

### Zone Permissions (if using zone lookup)

5. **Zone:Zone** - **Read**
   - Required to look up Zone ID by zone name
   - Only needed if `CLOUDFLARE_ZONE_NAME` is set in the job

## How to Create the API Token

### Step 1: Go to API Tokens

1. Log in to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Click on your profile (top right) → **My Profile**
3. Go to **API Tokens** tab
4. Click **Create Token**

### Step 2: Use Custom Token Template

1. Click **Create Custom Token**
2. Configure the token:

#### Token Name
```
heatpump-api-access-setup
```

#### Permissions

**Account - Cloudflare Access:Apps**
- Permission: **Edit**

**Account - Cloudflare Access:Service Tokens**
- Permission: **Edit**

**Account - Cloudflare Access:Policies**
- Permission: **Edit**

**Zone - Zone:Zone** (if using zone lookup)
- Permission: **Read**
- Zone Resources: Select your zone (`k12n.com`)

**Zone - Zone:Zone Settings** (optional)
- Permission: **Read**
- Zone Resources: Select your zone (`k12n.com`)

#### Account Resources

Select your Cloudflare account (the account where your Zero Trust is configured)

#### Zone Resources (if using zone permissions)

Select the zone: `k12n.com`

### Step 3: Create and Save

1. Click **Continue to summary**
2. Review the permissions
3. Click **Create Token**
4. **IMPORTANT**: Copy the token immediately - you won't be able to see it again!
5. Save it securely

## Getting Your Account ID

You need your Cloudflare Account ID for the secret. You can find it:

1. **From Dashboard URL**: When viewing any resource, the URL contains your account ID:
   ```
   https://dash.cloudflare.com/<account-id>/...
   ```

2. **From Account Settings**:
   - Go to Cloudflare Dashboard
   - Click on your account name (top right)
   - The Account ID is displayed in the account selector

3. **From API** (if you have a token with account read permission):
   ```bash
   curl -X GET "https://api.cloudflare.com/client/v4/accounts" \
     -H "Authorization: Bearer YOUR_API_TOKEN" \
     -H "Content-Type: application/json"
   ```

## Creating the Sealed Secret

Once you have both the API token and Account ID:

```bash
kubectl create secret generic cloudflare-api-token \
  --from-literal=api-token='<your-api-token>' \
  --from-literal=account-id='<your-account-id>' \
  --namespace=heatpump-api \
  --dry-run=client -o yaml | \
  kubeseal -o yaml > cloudflare-api-token-secret-sealed.yaml
```

Then add the sealed secret file to your kustomization.yaml:

```yaml
resources:
  - cloudflare-api-token-secret-sealed.yaml
  - setup-cloudflare-access-job.yaml
```

## Minimum Required Permissions Summary

For the setup job to work, the API token must have:

✅ **Account - Cloudflare Access:Apps** - Edit  
✅ **Account - Cloudflare Access:Service Tokens** - Edit  
✅ **Account - Cloudflare Access:Policies** - Edit  
✅ **Zone - Zone:Zone** - Read (if using zone lookup)  

## Security Best Practices

1. **Principle of Least Privilege**: Only grant the minimum permissions needed
2. **Token Rotation**: Rotate API tokens periodically (every 90 days recommended)
3. **Scope Limitation**: Limit token to specific accounts/zones if possible
4. **Secure Storage**: Store tokens in sealed secrets, never commit unencrypted tokens
5. **Audit Logging**: Monitor API token usage in Cloudflare audit logs

## Troubleshooting

### Error: "Insufficient permissions"

If you get permission errors:
1. Verify all required permissions are granted
2. Check that the token is scoped to the correct account
3. Ensure the token hasn't expired or been revoked

### Error: "Account not found"

- Verify the Account ID is correct
- Ensure the token has access to the specified account

### Error: "Zone not found"

- Verify the zone name matches exactly (case-sensitive)
- Check that the token has read access to the zone
- Or hardcode the Zone ID in the job instead of looking it up

## Alternative: Using Zone ID Directly

If you prefer not to grant zone read permissions, you can:

1. Get your Zone ID manually:
   ```bash
   curl -X GET "https://api.cloudflare.com/client/v4/zones?name=k12n.com" \
     -H "Authorization: Bearer YOUR_API_TOKEN" \
     -H "Content-Type: application/json"
   ```

2. Modify the job to use `CLOUDFLARE_ZONE_ID` environment variable instead of zone lookup
3. Remove zone read permissions from the API token

