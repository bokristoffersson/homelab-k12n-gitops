import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from './contexts/ThemeContext';
import Layout from './components/Layout/Layout';
import Dashboard from './components/Dashboard/Dashboard';
import Settings from './components/Settings/Settings';
import Plugs from './components/Plugs/Plugs';
import AuthLogin from './components/Auth/AuthLogin';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  // Authentication is handled by traefikoidc middleware
  // API routes are protected - 401 triggers redirect to /auth/login
  // traefikoidc handles OIDC flow and sets session cookie

  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <Routes>
            <Route path="/auth/login" element={<AuthLogin />} />
            <Route path="/" element={<Layout />}>
              <Route index element={<Navigate to="/dashboard" replace />} />
              <Route path="dashboard" element={<Dashboard />} />
              <Route path="settings" element={<Settings />} />
              <Route path="plugs" element={<Plugs />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </ThemeProvider>
  );
}

export default App;



