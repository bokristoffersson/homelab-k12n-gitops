````javascript
---
name: Chunk 8 Frontend Components
overview: Build React components for dashboard, authentication UI, and data visualization
todos:
    - id: create-login-component
    content: Build Login.tsx with form, validation, and auth integration
    status: pending
    - id: create-protected-route
    content: Build ProtectedRoute.tsx route guard component
    status: pending
    - id: create-current-power-card
    content: Build CurrentPowerCard component
    status: pending
    - id: create-hourly-total-card
    content: Build HourlyTotalCard component
    status: pending
    - id: create-heatpump-status
    content: Build HeatpumpStatus component
    status: pending
    - id: create-temperatures
    content: Build Temperatures component
    status: pending
    - id: create-hourly-chart
    content: Build HourlyChart component with recharts
    status: pending
    - id: create-dashboard
    content: Build main Dashboard component combining all cards
    status: pending
    - id: setup-routing
    content: Set up React Router in App.tsx with routes
    status: pending
    - id: add-styling
    content: Add CSS for layout and component styling
    status: pending
---

# Chunk 8: Frontend Components

## Overview

Build all React components for the dashboard UI including authentication, data cards, charts, and routing. This creates the complete user interface.

## Files to Create

### 1. Authentication Components

**File**: `applications/heatpump-web/src/components/Auth/Login.tsx`

- Login form with username/password inputs
- Form validation
- Call auth service login function
- Error handling and display
- Redirect on successful login

**File**: `applications/heatpump-web/src/components/Auth/ProtectedRoute.tsx`

- Route guard component
- Check authentication status
- Redirect to login if not authenticated
- Render children if authenticated

### 2. Dashboard Components

**File**: `applications/heatpump-web/src/components/Dashboard/Dashboard.tsx`

- Main dashboard container
- Use React Query to fetch data
- Layout with grid/flexbox
- Combine all dashboard cards
- Loading and error states

**File**: `applications/heatpump-web/src/components/Dashboard/CurrentPowerCard.tsx`

- Display current power consumption
- Fetch from /api/v1/energy/latest
- Format as kW with appropriate styling
- Loading state

**File**: `applications/heatpump-web/src/components/Dashboard/HourlyTotalCard.tsx`

- Display hourly total energy
- Fetch from /api/v1/energy/hourly-total
- Format as kWh
- Show current hour label

**File**: `applications/heatpump-web/src/components/Dashboard/HeatpumpStatus.tsx`

- Display heatpump status indicators
- Fetch from /api/v1/heatpump/latest
- Show device status, mode, etc.
- Visual indicators (colors/icons)

**File**: `applications/heatpump-web/src/components/Dashboard/Temperatures.tsx`

- Display temperature readings
- From heatpump latest data
- Show flow temp, return temp, etc.
- Temperature gauge or numeric display

**File**: `applications/heatpump-web/src/components/Dashboard/HourlyChart.tsx`

- Chart component using recharts
- Fetch from /api/v1/energy/history
- Display hourly energy consumption over time
- Line or bar chart

### 3. Routing

**File**: `applications/heatpump-web/src/App.tsx`

- Set up React Router
- Define routes: /login, /dashboard
- Protected route wrapper
- Navigation/layout wrapper

### 4. Styling

**File**: `applications/heatpump-web/src/index.css`

- Base styles
- CSS variables for theming
- Responsive design basics

**File**: `applications/heatpump-web/src/components/Dashboard/Dashboard.css` (optional)

- Component-specific styles

## Implementation Steps

1. Create Login component with form and validation
2. Create ProtectedRoute wrapper component
3. Create all dashboard card components
4. Create HourlyChart with recharts
5. Create main Dashboard component that uses all cards
6. Set up routing in App.tsx
7. Add basic styling for UI
8. Test all components render correctly
9. Test data fetching and display
10. Test authentication flow

## Component Hierarchy

```
App
├── Router
│   ├── Login (public)
│   └── ProtectedRoute
│       └── Dashboard
│           ├── CurrentPowerCard
│           ├── HourlyTotalCard
│           ├── HeatpumpStatus
│           ├── Temperatures
│           └── HourlyChart
```

## Verification

```bash
cd applications/heatpump-web
npm run dev
# Test login flow
# Test dashboard renders all components
# Test data displays correctly
# Test navigation
```

## Dependencies

- Chunk 7: API client and auth service must be complete
- Chunk 6: Backend API must be deployed and accessible

## Next Chunk

Chunk 9: Frontend Deployment and CI/CD


````