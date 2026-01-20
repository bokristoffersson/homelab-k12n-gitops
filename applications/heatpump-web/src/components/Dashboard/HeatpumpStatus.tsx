import { HeatpumpStatus as HeatpumpStatusType } from '../../types/heatpump';

interface HeatpumpStatusProps {
  heatpump: HeatpumpStatusType | undefined;
  error?: Error | null;
  isLoading?: boolean;
}

export default function HeatpumpStatus({ heatpump, error, isLoading }: HeatpumpStatusProps) {
  const StatusBadge = ({ label, value }: { label: string; value: boolean | null | undefined }) => (
    <div className="status-item">
      <span className="status-label">
        <span className={`status-dot ${value ? 'on' : 'off'}`} aria-hidden="true" />
        {label}:
      </span>
      <span className={`status-badge ${value ? 'on' : 'off'}`}>
        {value ? 'ON' : 'OFF'}
      </span>
    </div>
  );

  if (error) {
    return (
      <div className="card card-error">
        <h3>Heatpump Status</h3>
        <div className="error-message">
          Error: {error instanceof Error ? error.message : 'Failed to load'}
        </div>
      </div>
    );
  }

  if (isLoading || !heatpump) {
    return (
      <div className="card">
        <h3>Heatpump Status</h3>
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3>Heatpump Status</h3>
      <div className="status-grid">
        <StatusBadge label="Compressor" value={heatpump.compressor_on} />
        <StatusBadge label="Hot Water" value={heatpump.hotwater_production} />
        <StatusBadge label="Flow Pump" value={heatpump.flowlinepump_on} />
        <StatusBadge label="Brine Pump" value={heatpump.brinepump_on} />
        <StatusBadge label="Aux 3kW" value={heatpump.aux_heater_3kw_on} />
        <StatusBadge label="Aux 6kW" value={heatpump.aux_heater_6kw_on} />
      </div>
    </div>
  );
}



