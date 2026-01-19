import { HeatpumpStatus } from '../../types/heatpump';
import { TemperatureLatest } from '../../types/temperature';

interface TemperaturesProps {
  heatpump: HeatpumpStatus | undefined;
  indoorTemp: TemperatureLatest | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function Temperatures({ heatpump, indoorTemp, error, isLoading }: TemperaturesProps) {
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
        <TempItem label="Indoor" value={indoorTemp?.temperature_c} />
        <TempItem label="Outdoor" value={heatpump.outdoor_temp} />
        <TempItem label="Supply" value={heatpump.supplyline_temp} />
        <TempItem label="Return" value={heatpump.returnline_temp} />
        <TempItem label="Hot Water" value={heatpump.hotwater_temp} />
        <TempItem label="Brine Out" value={heatpump.brine_out_temp} />
        <TempItem label="Brine In" value={heatpump.brine_in_temp} />
        <TempItem label="Integral" value={heatpump.integral} />
      </div>
      {indoorTemp?.humidity && (
        <div style={{ marginTop: '0.5rem', fontSize: '0.9rem', color: '#888' }}>
          Indoor Humidity: {indoorTemp.humidity.toFixed(1)}%
        </div>
      )}
    </div>
  );
}



