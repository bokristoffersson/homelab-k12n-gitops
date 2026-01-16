import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getAllSettings, updateSetting } from '../../services/settings';
import { HeatpumpMode, SettingPatch } from '../../types/settings';
import { OutboxStatus } from './OutboxStatus';
import './Settings.css';

export default function Settings() {
  const queryClient = useQueryClient();
  const [editingDevice, setEditingDevice] = useState<string | null>(null);
  const [formData, setFormData] = useState<SettingPatch>({});
  const [submitStatus, setSubmitStatus] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

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
      setSubmitStatus({ type: 'success', message: 'Settings update submitted successfully' });
      setTimeout(() => setSubmitStatus(null), 5000);
    },
    onError: (error: any) => {
      const message = error?.response?.data?.error || error?.message || 'Failed to update settings';
      setSubmitStatus({ type: 'error', message });
      setTimeout(() => setSubmitStatus(null), 5000);
    },
  });

  const startEditing = (deviceId: string) => {
    const setting = settings?.find((s) => s.device_id === deviceId);
    if (setting) {
      setFormData({
        indoor_target_temp: setting.indoor_target_temp ?? undefined,
        mode: setting.mode ?? undefined,
        curve: setting.curve ?? undefined,
        curve_min: setting.curve_min ?? undefined,
        curve_max: setting.curve_max ?? undefined,
        curve_plus_5: setting.curve_plus_5 ?? undefined,
        curve_zero: setting.curve_zero ?? undefined,
        curve_minus_5: setting.curve_minus_5 ?? undefined,
        heatstop: setting.heatstop ?? undefined,
      });
      setEditingDevice(deviceId);
      setSubmitStatus(null);
    }
  };

  const cancelEditing = () => {
    setEditingDevice(null);
    setFormData({});
    setSubmitStatus(null);
  };

  const handleSubmit = (deviceId: string) => {
    const patch: SettingPatch = {};
    Object.entries(formData).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        patch[key as keyof SettingPatch] = value as any;
      }
    });

    if (Object.keys(patch).length === 0) {
      setSubmitStatus({ type: 'error', message: 'No changes to submit' });
      setTimeout(() => setSubmitStatus(null), 3000);
      return;
    }

    updateMutation.mutate({ deviceId, patch });
    setEditingDevice(null);
    setFormData({});
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

  const isEditing = (deviceId: string) => editingDevice === deviceId;

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h2>Heatpump Settings</h2>
        <button className="refresh-button" onClick={() => refetch()}>
          Refresh
        </button>
      </div>

      {submitStatus && (
        <div className={`status-banner ${submitStatus.type}`}>
          {submitStatus.message}
        </div>
      )}

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

              {isEditing(setting.device_id) ? (
                <>
                  <div className="settings-section">
                    <h4>Temperature Control</h4>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="indoor_target_temp">
                        Indoor Target Temperature (°C)
                      </label>
                      <input
                        id="indoor_target_temp"
                        type="number"
                        step="0.1"
                        className="setting-input"
                        value={formData.indoor_target_temp ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            indoor_target_temp: e.target.value ? parseFloat(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="mode">
                        Mode
                      </label>
                      <select
                        id="mode"
                        className="setting-input"
                        value={formData.mode ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            mode: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      >
                        <option value="">Select mode...</option>
                        {Object.entries(HeatpumpMode).map(([key, value]) => (
                          <option key={key} value={key}>
                            {value}
                          </option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div className="settings-section">
                    <h4>Heating Curve</h4>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve">
                        Curve
                      </label>
                      <input
                        id="curve"
                        type="number"
                        className="setting-input"
                        value={formData.curve ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve_min">
                        Curve Min (°C)
                      </label>
                      <input
                        id="curve_min"
                        type="number"
                        className="setting-input"
                        value={formData.curve_min ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve_min: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve_max">
                        Curve Max (°C)
                      </label>
                      <input
                        id="curve_max"
                        type="number"
                        className="setting-input"
                        value={formData.curve_max ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve_max: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve_plus_5">
                        Curve at +5°C
                      </label>
                      <input
                        id="curve_plus_5"
                        type="number"
                        className="setting-input"
                        value={formData.curve_plus_5 ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve_plus_5: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve_zero">
                        Curve at 0°C
                      </label>
                      <input
                        id="curve_zero"
                        type="number"
                        className="setting-input"
                        value={formData.curve_zero ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve_zero: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="curve_minus_5">
                        Curve at -5°C
                      </label>
                      <input
                        id="curve_minus_5"
                        type="number"
                        className="setting-input"
                        value={formData.curve_minus_5 ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            curve_minus_5: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                  </div>

                  <div className="settings-section">
                    <h4>Other Settings</h4>
                    <div className="setting-item">
                      <label className="setting-label" htmlFor="heatstop">
                        Heat Stop
                      </label>
                      <input
                        id="heatstop"
                        type="number"
                        className="setting-input"
                        value={formData.heatstop ?? ''}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            heatstop: e.target.value ? parseInt(e.target.value) : undefined,
                          })
                        }
                      />
                    </div>
                  </div>

                  <div className="settings-actions">
                    <button
                      className="action-button primary"
                      onClick={() => handleSubmit(setting.device_id)}
                      disabled={updateMutation.isPending}
                    >
                      {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
                    </button>
                    <button
                      className="action-button secondary"
                      onClick={cancelEditing}
                      disabled={updateMutation.isPending}
                    >
                      Cancel
                    </button>
                  </div>
                </>
              ) : (
                <>
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

                  <div className="settings-actions">
                    <button
                      className="action-button primary"
                      onClick={() => startEditing(setting.device_id)}
                    >
                      Edit Settings
                    </button>
                  </div>
                </>
              )}

              <OutboxStatus deviceId={setting.device_id} />
            </div>
          ))}
        </div>
      )}

      <div className="settings-footer">
        <div className="info-box">
          <strong>ℹ️ Transactional Outbox Pattern</strong>
          <p>
            Settings changes are queued in the outbox table and published to MQTT. Status tracking shows the full
            lifecycle: pending → published → confirmed.
          </p>
        </div>
      </div>
    </div>
  );
}
