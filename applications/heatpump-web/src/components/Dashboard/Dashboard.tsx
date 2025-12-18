import { useQuery } from '@tanstack/react-query';
import { api } from '../../services/api';
import { EnergyLatest, HourlyTotal, EnergyHourly } from '../../types/energy';
import { HeatpumpStatus } from '../../types/heatpump';
import { useTheme } from '../../hooks/useTheme';
import CurrentPowerCard from './CurrentPowerCard';
import HourlyTotalCard from './HourlyTotalCard';
import HeatpumpStatusComponent from './HeatpumpStatus';
import Temperatures from './Temperatures';
import HourlyChart from './HourlyChart';
import './Dashboard.css';

export default function Dashboard() {
  const { theme, toggleTheme } = useTheme();
  
  const { data: energy, error: energyError, isLoading: energyLoading } = useQuery<EnergyLatest>({
    queryKey: ['energy', 'latest'],
    queryFn: () => api.get('/api/v1/energy/latest').then((r) => r.data),
    refetchInterval: 5000,
    retry: 1,
  });

  const { data: hourlyTotal, error: hourlyError, isLoading: hourlyLoading } = useQuery<HourlyTotal>({
    queryKey: ['energy', 'hourly-total'],
    queryFn: () => api.get('/api/v1/energy/hourly-total').then((r) => r.data),
    refetchInterval: 60000,
    retry: 1,
  });

  const { data: heatpump, error: heatpumpError, isLoading: heatpumpLoading } = useQuery<HeatpumpStatus>({
    queryKey: ['heatpump', 'latest'],
    queryFn: () => api.get('/api/v1/heatpump/latest').then((r) => r.data),
    refetchInterval: 5000,
    retry: 1,
  });

  const { data: history, error: historyError, isLoading: historyLoading } = useQuery<EnergyHourly[]>({
    queryKey: ['energy', 'history', '24h'],
    queryFn: () => {
      const now = new Date();
      const past24h = new Date(now.getTime() - 24 * 60 * 60 * 1000); // 24 hours ago
      return api
        .get('/api/v1/energy/history', {
          params: {
            from: past24h.toISOString(),
            to: now.toISOString(),
          },
        })
        .then((r) => r.data);
    },
    refetchInterval: 300000,
    retry: 1,
  });

  // Show error banner if any query failed
  const hasErrors = energyError || hourlyError || heatpumpError || historyError;

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1>Dashboard</h1>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <span>Last Update: {new Date().toLocaleTimeString()}</span>
          <button className="theme-toggle" onClick={toggleTheme} aria-label="Toggle theme">
            {theme === 'light' ? '🌙' : '☀️'}
          </button>
        </div>
      </div>
      {hasErrors && (
        <div className="error-banner">
          <strong>⚠️ API Errors:</strong>
          {energyError && (
            <div>
              Energy Latest: {energyError instanceof Error 
                ? (energyError as any).response?.data?.error || energyError.message 
                : 'Failed to load'}
              {(energyError as any).response?.status && ` (Status: ${(energyError as any).response.status})`}
            </div>
          )}
          {hourlyError && (
            <div>
              Hourly Total: {hourlyError instanceof Error 
                ? (hourlyError as any).response?.data?.error || hourlyError.message 
                : 'Failed to load'}
              {(hourlyError as any).response?.status && ` (Status: ${(hourlyError as any).response.status})`}
            </div>
          )}
          {heatpumpError && (
            <div>
              Heatpump Status: {heatpumpError instanceof Error 
                ? (heatpumpError as any).response?.data?.error || heatpumpError.message 
                : 'Failed to load'}
              {(heatpumpError as any).response?.status && ` (Status: ${(heatpumpError as any).response.status})`}
            </div>
          )}
          {historyError && (
            <div>
              Energy History: {historyError instanceof Error 
                ? (historyError as any).response?.data?.error || historyError.message 
                : 'Failed to load'}
              {(historyError as any).response?.status && ` (Status: ${(historyError as any).response.status})`}
            </div>
          )}
          <div className="error-help">
            Check backend logs: <code>kubectl logs -n redpanda-sink -l app=redpanda-sink --tail=50</code>
            <br />
            Test API: <code>curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/v1/energy/latest</code>
          </div>
        </div>
      )}
      <div className="dashboard-grid">
        <CurrentPowerCard energy={energy} error={energyError} isLoading={energyLoading} />
        <HourlyTotalCard hourlyTotal={hourlyTotal} error={hourlyError} isLoading={hourlyLoading} />
        <HeatpumpStatusComponent heatpump={heatpump} error={heatpumpError} isLoading={heatpumpLoading} />
        <Temperatures heatpump={heatpump} error={heatpumpError} isLoading={heatpumpLoading} />
        <HourlyChart history={history} error={historyError} isLoading={historyLoading} />
      </div>
    </div>
  );
}



