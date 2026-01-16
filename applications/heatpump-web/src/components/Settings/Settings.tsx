import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getAllSettings, updateSetting } from '../../services/settings';
import { HeatpumpMode, HeatpumpSetting, SettingPatch } from '../../types/settings';
import { OutboxStatus } from './OutboxStatus';
import './Settings.css';

export default function Settings() {
  const queryClient = useQueryClient();

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

  const updateMutation = useMutation({
    mutationFn: ({ deviceId, patch }: { deviceId: string; patch: SettingPatch }) =>
      updateSetting(deviceId, patch),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['heatpump', 'settings'] });
      queryClient.invalidateQueries({ queryKey: ['outbox'] });
    },
  });

  const adjustField = (deviceId: string, fieldName: keyof SettingPatch, delta: number) => {
    const setting = settings?.find((s) => s.device_id === deviceId);
    if (!setting) return;

    const currentValue = setting[fieldName as keyof HeatpumpSetting];
    if (currentValue === null || currentValue === undefined) return;

    const newValue = (currentValue as number) + delta;
    const patch: SettingPatch = {
      [fieldName]: newValue,
    };

    updateMutation.mutate({ deviceId, patch });
  };

  const AdjustableField = ({
    label,
    value,
    deviceId,
    fieldName,
    unit,
  }: {
    label: string;
    value: number | null;
    deviceId: string;
    fieldName: keyof SettingPatch;
    unit?: string;
  }) => {
    if (value === null) {
      return (
        <div className="setting-item">
          <span className="setting-label">{label}</span>
          <span className="setting-value">N/A</span>
        </div>
      );
    }

    return (
      <div className="setting-item adjustable">
        <span className="setting-label">{label}</span>
        <div className="setting-value-controls">
          <button
            className="adjust-button"
            onClick={() => adjustField(deviceId, fieldName, -1)}
            disabled={updateMutation.isPending}
            aria-label={`Decrease ${label}`}
          >
            -
          </button>
          <span className="setting-value">
            {value}{unit && unit}
          </span>
          <button
            className="adjust-button"
            onClick={() => adjustField(deviceId, fieldName, 1)}
            disabled={updateMutation.isPending}
            aria-label={`Increase ${label}`}
          >
            +
          </button>
        </div>
      </div>
    );
  };

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
                <AdjustableField
                  label="Indoor Target Temperature"
                  value={setting.indoor_target_temp}
                  deviceId={setting.device_id}
                  fieldName="indoor_target_temp"
                  unit="°C"
                />
                <div className="setting-item">
                  <span className="setting-label">Mode</span>
                  <span className="setting-value setting-badge">
                    {setting.mode !== null ? HeatpumpMode[setting.mode] || `Unknown (${setting.mode})` : 'N/A'}
                  </span>
                </div>
              </div>

              <div className="settings-section">
                <h4>Heating Curve</h4>
                <AdjustableField
                  label="Curve"
                  value={setting.curve}
                  deviceId={setting.device_id}
                  fieldName="curve"
                />
                <AdjustableField
                  label="Curve Min"
                  value={setting.curve_min}
                  deviceId={setting.device_id}
                  fieldName="curve_min"
                  unit="°C"
                />
                <AdjustableField
                  label="Curve Max"
                  value={setting.curve_max}
                  deviceId={setting.device_id}
                  fieldName="curve_max"
                  unit="°C"
                />
                <AdjustableField
                  label="Curve at +5°C"
                  value={setting.curve_plus_5}
                  deviceId={setting.device_id}
                  fieldName="curve_plus_5"
                  unit="°C"
                />
                <AdjustableField
                  label="Curve at 0°C"
                  value={setting.curve_zero}
                  deviceId={setting.device_id}
                  fieldName="curve_zero"
                  unit="°C"
                />
                <AdjustableField
                  label="Curve at -5°C"
                  value={setting.curve_minus_5}
                  deviceId={setting.device_id}
                  fieldName="curve_minus_5"
                  unit="°C"
                />
              </div>

              <div className="settings-section">
                <h4>Other Settings</h4>
                <AdjustableField
                  label="Heat Stop"
                  value={setting.heatstop}
                  deviceId={setting.device_id}
                  fieldName="heatstop"
                />
              </div>

              <OutboxStatus deviceId={setting.device_id} />
            </div>
          ))}
        </div>
      )}

      <div className="settings-footer">
        <div className="info-box">
          <strong>ℹ️ Increment/Decrement Control</strong>
          <p>
            Use the +/- buttons to adjust settings one step at a time. Each change is queued in the
            outbox and sent to your heatpump via MQTT.
          </p>
        </div>
      </div>
    </div>
  );
}
