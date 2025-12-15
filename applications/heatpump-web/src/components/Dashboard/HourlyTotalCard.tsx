import { HourlyTotal } from '../../types/energy';

interface HourlyTotalCardProps {
  hourlyTotal: HourlyTotal | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function HourlyTotalCard({ hourlyTotal, error, isLoading }: HourlyTotalCardProps) {
  if (error) {
    return (
      <div className="card card-error">
        <h3>This Hour</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading || !hourlyTotal) {
    return (
      <div className="card">
        <h3>This Hour</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3>This Hour</h3>
      <div className="energy-value">
        {hourlyTotal.total_kwh.toFixed(2)} <span className="unit">kWh</span>
      </div>
      <div className="subtitle">(so far)</div>
    </div>
  );
}



