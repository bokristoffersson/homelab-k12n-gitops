import { HeatpumpStatus } from '../../types/heatpump';

interface TemperaturesProps {
  heatpump: HeatpumpStatus | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function Temperatures({ heatpump, error, isLoading }: TemperaturesProps) {
  const TempItem = ({ label, value }: { label: string; value: number | null | undefined }) => (
    <div className="temp-item">
      <span className="temp-label">{label}:</span>
      <span className="temp-value">
        {value !== null && value !== undefined ? `${value}°C` : 'N/A'}
      </span>
    </div>
  );

  if (error) {
    return (
      <div className="card card-error">
        <h3>Temperatures</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading || !heatpump) {
    return (
      <div className="card">
        <h3>Temperatures</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3>Temperatures</h3>
      <div className="temp-grid">
        <TempItem label="Outdoor" value={heatpump.outdoor_temp} />
        <TempItem label="Supply" value={heatpump.supplyline_temp} />
        <TempItem label="Return" value={heatpump.returnline_temp} />
        <TempItem label="Hot Water" value={heatpump.hotwater_temp} />
        <TempItem label="Brine Out" value={heatpump.brine_out_temp} />
        <TempItem label="Brine In" value={heatpump.brine_in_temp} />
      </div>
    </div>
  );
}



