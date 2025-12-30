import { TemperatureReading } from '../../types/temperature';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTheme } from '../../hooks/useTheme';

interface TemperatureChartProps {
  history: TemperatureReading[] | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

const CustomTooltip = ({ active, payload }: any) => {
  const { theme } = useTheme();
  const bgColor = theme === 'dark' ? 'rgba(45, 45, 45, 0.95)' : 'rgba(255, 255, 255, 0.95)';
  const borderColor = theme === 'dark' ? '#404040' : '#ccc';
  const textColor = theme === 'dark' ? '#e0e0e0' : '#333';
  const tempColor = theme === 'dark' ? '#ff7c43' : '#ff6b35';
  const humidityColor = theme === 'dark' ? '#5ba3f5' : '#4a90e2';

  if (active && payload && payload.length) {
    return (
      <div style={{
        backgroundColor: bgColor,
        border: `1px solid ${borderColor}`,
        borderRadius: '4px',
        padding: '10px',
        boxShadow: theme === 'dark' ? '0 2px 4px rgba(0,0,0,0.3)' : '0 2px 4px rgba(0,0,0,0.1)'
      }}>
        <p style={{ margin: '0 0 5px 0', fontWeight: 'bold', color: textColor }}>
          {payload[0].payload.formattedTime}
        </p>
        {payload.map((entry: any, index: number) => (
          <p key={index} style={{ margin: 0, color: entry.name === 'Temperature' ? tempColor : humidityColor }}>
            {entry.name}: {entry.value !== null && entry.value !== undefined ? entry.value.toFixed(1) : 'N/A'}
            {entry.name === 'Temperature' ? '°C' : '%'}
          </p>
        ))}
      </div>
    );
  }
  return null;
};

export default function TemperatureChart({ history, error, isLoading }: TemperatureChartProps) {
  const { theme } = useTheme();
  const gridColor = theme === 'dark' ? '#404040' : '#e0e0e0';
  const textColor = theme === 'dark' ? '#b0b0b0' : '#666';
  const tempColor = theme === 'dark' ? '#ff7c43' : '#ff6b35';
  const humidityColor = theme === 'dark' ? '#5ba3f5' : '#4a90e2';

  if (error) {
    return (
      <div className="card chart-card card-error">
        <h3>Indoor Temperature - Last 24 Hours</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="card chart-card">
        <h3>Indoor Temperature - Last 24 Hours</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  if (!history || history.length === 0) {
    return (
      <div className="card chart-card">
        <h3>Indoor Temperature - Last 24 Hours</h3>
        <div className="no-data">No data available yet. Waiting for Shelly H&T sensor data...</div>
      </div>
    );
  }

  const chartData = history.map(item => {
    const time = new Date(item.time);
    return {
      time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false }),
      formattedTime: time.toLocaleString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
      }),
      temperature: item.temperature_c,
      humidity: item.humidity,
    };
  });

  return (
    <div className="card chart-card">
      <h3>Indoor Temperature - Last 24 Hours</h3>
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
            yAxisId="temp"
            label={{ value: '°C', angle: -90, position: 'insideLeft', fill: textColor }}
            tick={{ fill: textColor }}
          />
          <YAxis
            yAxisId="humidity"
            orientation="right"
            label={{ value: '%', angle: 90, position: 'insideRight', fill: textColor }}
            tick={{ fill: textColor }}
          />
          <Tooltip content={<CustomTooltip />} />
          <Legend wrapperStyle={{ color: textColor }} />
          <Line
            yAxisId="temp"
            type="monotone"
            dataKey="temperature"
            stroke={tempColor}
            strokeWidth={2}
            name="Temperature"
            dot={false}
          />
          <Line
            yAxisId="humidity"
            type="monotone"
            dataKey="humidity"
            stroke={humidityColor}
            strokeWidth={2}
            name="Humidity"
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
