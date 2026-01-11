import { useQuery } from '@tanstack/react-query';
import { getAllSettings } from '../../services/settings';
import { HeatpumpMode } from '../../types/settings';
import './Settings.css';

export default function Settings() {
  const {
    data: settings,
    error,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ['heatpump', 'settings'],
    queryFn: getAllSettings,
    refetchInterval: 30000, // Refresh every 30 seconds
    retry: 1,
  });

  if (isLoading) {
    return (
      <div className="settings-page">
        <div className="settings-header">
          <h2>Heatpump Settings</h2>
        </div>
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="settings-page">
        <div className="settings-header">
          <h2>Heatpump Settings</h2>
        </div>
        <div className="error-banner">
          <strong>Error loading settings:</strong>
          <div>
            {error instanceof Error
              ? (error as any).response?.data?.error || error.message
              : 'Failed to load settings'}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h2>Heatpump Settings</h2>
        <button className="refresh-button" onClick={() => refetch()}>
          Refresh
        </button>
      </div>

      {!settings || settings.length === 0 ? (
        <div className="no-data">No heatpump devices found</div>
      ) : (
        <div className="settings-grid">
          {settings.map((setting) => (
            <div key={setting.device_id} className="settings-card">
              <div className="settings-card-header">
                <h3>{setting.device_id}</h3>
                <span className="last-updated">
                  Updated: {new Date(setting.updated_at).toLocaleString()}
                </span>
              </div>

              <div className="settings-section">
                <h4>Temperature Control</h4>
                <div className="setting-item">
                  <span className="setting-label">Indoor Target Temperature</span>
                  <span className="setting-value">
                    {setting.indoor_target_temp !== null
                      ? `${setting.indoor_target_temp.toFixed(1)}°C`
                      : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Mode</span>
                  <span className="setting-value setting-badge">
                    {setting.mode !== null ? HeatpumpMode[setting.mode] || `Unknown (${setting.mode})` : 'N/A'}
                  </span>
                </div>
              </div>

              <div className="settings-section">
                <h4>Heating Curve</h4>
                <div className="setting-item">
                  <span className="setting-label">Curve</span>
                  <span className="setting-value">
                    {setting.curve !== null ? setting.curve : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Curve Min</span>
                  <span className="setting-value">
                    {setting.curve_min !== null ? `${setting.curve_min}°C` : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Curve Max</span>
                  <span className="setting-value">
                    {setting.curve_max !== null ? `${setting.curve_max}°C` : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Curve at +5°C</span>
                  <span className="setting-value">
                    {setting.curve_plus_5 !== null ? `${setting.curve_plus_5}°C` : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Curve at 0°C</span>
                  <span className="setting-value">
                    {setting.curve_zero !== null ? `${setting.curve_zero}°C` : 'N/A'}
                  </span>
                </div>
                <div className="setting-item">
                  <span className="setting-label">Curve at -5°C</span>
                  <span className="setting-value">
                    {setting.curve_minus_5 !== null ? `${setting.curve_minus_5}°C` : 'N/A'}
                  </span>
                </div>
              </div>

              <div className="settings-section">
                <h4>Other Settings</h4>
                <div className="setting-item">
                  <span className="setting-label">Heat Stop</span>
                  <span className="setting-value">
                    {setting.heatstop !== null ? setting.heatstop : 'N/A'}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="settings-footer">
        <div className="info-box">
          <strong>ℹ️ Read-Only View</strong>
          <p>Settings are automatically synchronized from your heatpump telemetry. Editing capabilities coming in Phase 2.</p>
        </div>
      </div>
    </div>
  );
}
