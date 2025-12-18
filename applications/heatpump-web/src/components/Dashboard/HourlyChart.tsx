import { EnergyHourly } from '../../types/energy';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface HourlyChartProps {
  history: EnergyHourly[] | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

const CustomTooltip = ({ active, payload }: any) => {
  if (active && payload && payload.length) {
    return (
      <div style={{
        backgroundColor: 'rgba(255, 255, 255, 0.95)',
        border: '1px solid #ccc',
        borderRadius: '4px',
        padding: '10px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
      }}>
        <p style={{ margin: '0 0 5px 0', fontWeight: 'bold' }}>{payload[0].payload.timeRange}</p>
        <p style={{ margin: 0, color: '#4a90e2' }}>
          Energy: {payload[0].value.toFixed(2)} kWh
        </p>
      </div>
    );
  }
  return null;
};

export default function HourlyChart({ history, error, isLoading }: HourlyChartProps) {
  if (error) {
    return (
      <div className="card chart-card card-error">
        <h3>Hourly Consumption - Last 24 Hours</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="card chart-card">
        <h3>Hourly Consumption - Last 24 Hours</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  if (!history || history.length === 0) {
    return (
      <div className="card chart-card">
        <h3>Hourly Consumption - Last 24 Hours</h3>
        <div className="no-data">No data available</div>
      </div>
    );
  }

  const chartData = history.map(item => {
    const startTime = new Date(item.hour_start).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const endTime = new Date(item.hour_end).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    return {
      time: `${startTime}-${endTime}`,
      timeRange: `${startTime} - ${endTime}`,
      energy: item.total_energy_kwh || 0,
    };
  });

  return (
    <div className="card chart-card">
      <h3>Hourly Consumption - Last 24 Hours</h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" angle={-45} textAnchor="end" height={80} />
          <YAxis label={{ value: 'kWh', angle: -90, position: 'insideLeft' }} />
          <Tooltip content={<CustomTooltip />} />
          <Legend />
          <Line type="monotone" dataKey="energy" stroke="#4a90e2" strokeWidth={2} name="Energy (kWh)" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}



