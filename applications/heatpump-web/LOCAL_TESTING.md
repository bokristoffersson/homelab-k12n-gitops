# Local Testing Guide

This guide will help you test and verify the frontend application locally.

## Prerequisites

- Node.js (v18 or higher recommended)
- npm or yarn
- Access to the Kubernetes cluster (if backend is deployed there)
- kubectl configured (if testing against deployed backend)

## Step 1: Install Dependencies

```bash
cd applications/heatpump-web
npm install
```

## Step 2: Configure Environment Variables

The frontend needs to know where the backend API is located.

### Option A: Testing with Local Backend (if running locally)

If you have the backend running locally on port 8080, create a `.env` file:

```bash
# Create .env file
cat > .env << EOF
VITE_API_URL=http://localhost:8080
EOF
```

### Option B: Testing with Kubernetes Backend (Port Forward)

If the backend is deployed in Kubernetes, you'll need to port-forward the service:

```bash
# Port-forward the backend API service
kubectl port-forward -n redpanda-sink service/redpanda-sink 8080:8080
```

Then create a `.env` file pointing to localhost:

```bash
# Create .env file
cat > .env << EOF
VITE_API_URL=http://localhost:8080
EOF
```

**Note:** Keep the port-forward running in a separate terminal while testing.

## Step 3: Start the Development Server

```bash
npm run dev
```

The application will start on `http://localhost:3000` (as configured in `vite.config.ts`).

## Step 4: Test the Application

### 4.1 Test Login Flow

1. Open `http://localhost:3000` in your browser
2. You should be redirected to `/login` (or `/dashboard` if already authenticated)
3. Try logging in with your credentials
4. Verify you're redirected to `/dashboard` after successful login

### 4.2 Test Dashboard Components

Once logged in, verify all components are working:

- **Current Power Card**: Should display power in kW with a live indicator
- **Hourly Total Card**: Should show energy consumption for the current hour
- **Heatpump Status**: Should display status indicators (ON/OFF) for various components
- **Temperatures**: Should show temperature readings from the heatpump
- **Hourly Chart**: Should display a chart of hourly energy consumption

### 4.3 Test Data Refresh

- Components should auto-refresh:
  - Current Power: every 5 seconds
  - Hourly Total: every 60 seconds
  - Heatpump Status: every 5 seconds
  - Chart: every 5 minutes

### 4.4 Test Protected Routes

1. Log out (or clear localStorage)
2. Try accessing `/dashboard` directly
3. You should be redirected to `/login`

## Step 5: Verify Browser Console

Open browser DevTools (F12) and check:

- **Console tab**: Should have no errors
- **Network tab**: 
  - API requests should succeed (status 200)
  - Failed requests should show appropriate error messages
  - JWT token should be included in Authorization header

## Troubleshooting

### Backend Connection Issues

If you see connection errors:

1. **Check if backend is running:**
   ```bash
   # For Kubernetes
   kubectl get pods -n redpanda-sink
   
   # Check service
   kubectl get svc -n redpanda-sink
   ```

2. **Verify port-forward is active:**
   ```bash
   # Should show port-forward process
   lsof -i :8080
   ```

3. **Test API directly:**
   ```bash
   curl http://localhost:8080/api/v1/energy/latest
   # Should return JSON (or 401 if auth required)
   ```

### Authentication Issues

1. **Check localStorage:**
   - Open DevTools → Application → Local Storage
   - Look for `heatpump_auth_token` and `heatpump_username`

2. **Clear and retry:**
   ```javascript
   // In browser console
   localStorage.clear()
   // Then refresh and login again
   ```

### CORS Issues

If you see CORS errors, the backend needs to allow requests from `http://localhost:3000`. Check backend CORS configuration.

### Build Errors

If you see TypeScript or build errors:

```bash
# Check for TypeScript errors
npm run build

# Check linting
npm run lint
```

## Quick Test Checklist

- [ ] Dependencies installed (`node_modules` exists)
- [ ] `.env` file created with correct `VITE_API_URL`
- [ ] Backend accessible (port-forward active or local backend running)
- [ ] Dev server starts without errors
- [ ] Login page loads correctly
- [ ] Can login with valid credentials
- [ ] Dashboard displays all components
- [ ] Data loads and displays correctly
- [ ] Components auto-refresh
- [ ] Protected routes redirect to login when not authenticated
- [ ] No console errors
- [ ] Network requests succeed

## Next Steps

Once local testing is successful:
- Proceed to Chunk 9: Frontend Deployment and CI/CD
- Build production bundle: `npm run build`
- Test production build: `npm run preview`
