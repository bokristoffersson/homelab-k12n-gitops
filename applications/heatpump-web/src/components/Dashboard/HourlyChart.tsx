import { EnergyHourly } from '../../types/energy';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTheme } from '../../contexts/ThemeContext';

interface HourlyChartProps {
  history: EnergyHourly[] | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

const CustomTooltip = ({ active, payload }: any) => {
  const { theme } = useTheme();
  const bgColor = theme === 'dark' ? 'rgba(45, 45, 45, 0.95)' : 'rgba(255, 255, 255, 0.95)';
  const borderColor = theme === 'dark' ? '#404040' : '#ccc';
  const textColor = theme === 'dark' ? '#e0e0e0' : '#333';
  const accentColor = theme === 'dark' ? '#5ba3f5' : '#4a90e2';
  
  if (active && payload && payload.length) {
    return (
      <div style={{
        backgroundColor: bgColor,
        border: `1px solid ${borderColor}`,
        borderRadius: '4px',
        padding: '10px',
        boxShadow: theme === 'dark' ? '0 2px 4px rgba(0,0,0,0.3)' : '0 2px 4px rgba(0,0,0,0.1)'
      }}>
        <p style={{ margin: '0 0 5px 0', fontWeight: 'bold', color: textColor }}>{payload[0].payload.timeRange}</p>
        <p style={{ margin: 0, color: accentColor }}>
          Energy: {payload[0].value.toFixed(2)} kWh
        </p>
      </div>
    );
  }
  return null;
};

export default function HourlyChart({ history, error, isLoading }: HourlyChartProps) {
  const { theme } = useTheme();
  const gridColor = theme === 'dark' ? '#404040' : '#e0e0e0';
  const textColor = theme === 'dark' ? '#b0b0b0' : '#666';
  const lineColor = theme === 'dark' ? '#5ba3f5' : '#4a90e2';
  
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
          <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
          <XAxis 
            dataKey="time" 
            angle={-45} 
            textAnchor="end" 
            height={80}
            tick={{ fill: textColor }}
          />
          <YAxis 
            label={{ value: 'kWh', angle: -90, position: 'insideLeft', fill: textColor }}
            tick={{ fill: textColor }}
          />
          <Tooltip content={<CustomTooltip />} />
          <Legend wrapperStyle={{ color: textColor }} />
          <Line type="monotone" dataKey="energy" stroke={lineColor} strokeWidth={2} name="Energy (kWh)" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}



