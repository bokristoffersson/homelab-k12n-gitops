import { EnergyLatest } from '../../types/energy';

interface CurrentPowerCardProps {
  energy: EnergyLatest | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function CurrentPowerCard({ energy, error, isLoading }: CurrentPowerCardProps) {
  const powerW = energy?.consumption_total_actual_w 
    ? energy.consumption_total_actual_w 
    : energy?.consumption_total_w || 0;
  const powerKw = powerW / 1000;

  if (error) {
    return (
      <div className="card card-error">
        <h3>Current Power</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading || !energy) {
    return (
      <div className="card">
        <h3>Current Power</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3>Current Power</h3>
      <div className="power-value">
        {powerKw.toFixed(2)} <span className="unit">kW</span>
      </div>
      <div className="live-indicator">
        <span className="dot"></span> Live
      </div>
    </div>
  );
}



