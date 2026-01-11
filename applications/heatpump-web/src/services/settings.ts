import { api } from './api';
import { HeatpumpSetting, SettingsResponse, SettingResponse, SettingPatch } from '../types/settings';

// Get all device settings
export const getAllSettings = async (): Promise<HeatpumpSetting[]> => {
  const response = await api.get<SettingsResponse>('/api/v1/heatpump/settings');
  return response.data.settings;
};

// Get settings for a specific device
export const getSettingByDevice = async (deviceId: string): Promise<HeatpumpSetting> => {
  const response = await api.get<SettingResponse>(`/api/v1/heatpump/settings/${deviceId}`);
  return response.data;
};

// Update settings for a device (PATCH)
export const updateSetting = async (deviceId: string, patch: SettingPatch): Promise<HeatpumpSetting> => {
  const response = await api.patch<SettingResponse>(`/api/v1/heatpump/settings/${deviceId}`, patch);
  return response.data;
};
