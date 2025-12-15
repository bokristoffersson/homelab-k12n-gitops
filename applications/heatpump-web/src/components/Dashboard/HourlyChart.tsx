import { EnergyHourly } from '../../types/energy';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface HourlyChartProps {
  history: EnergyHourly[] | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function HourlyChart({ history, error, isLoading }: HourlyChartProps) {
  if (error) {
    return (
      <div className="card chart-card card-error">
        <h3>Today - Hourly Consumption</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="card chart-card">
        <h3>Today - Hourly Consumption</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  if (!history || history.length === 0) {
    return (
      <div className="card chart-card">
        <h3>Today - Hourly Consumption</h3>
        <div className="no-data">No data available</div>
      </div>
    );
  }

  const chartData = history.map(item => ({
    time: new Date(item.hour_start).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
    energy: item.total_energy_kwh || 0,
  }));

  return (
    <div className="card chart-card">
      <h3>Today - Hourly Consumption</h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis label={{ value: 'kWh', angle: -90, position: 'insideLeft' }} />
          <Tooltip />
          <Legend />
          <Line type="monotone" dataKey="energy" stroke="#4a90e2" strokeWidth={2} name="Energy (kWh)" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}



