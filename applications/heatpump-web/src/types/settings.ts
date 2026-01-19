export interface HeatpumpSetting {
  device_id: string;
  indoor_target_temp: number | null;
  mode: number | null;
  curve: number | null;
  curve_min: number | null;
  curve_max: number | null;
  curve_plus_5: number | null;
  curve_zero: number | null;
  curve_minus_5: number | null;
  heatstop: number | null;
  integral_setting: number | null;
  updated_at: string;
}

export interface SettingsResponse {
  settings: HeatpumpSetting[];
}

export interface SettingResponse extends HeatpumpSetting {}

export interface SettingPatch {
  indoor_target_temp?: number;
  mode?: number;
  curve?: number;
  curve_min?: number;
  curve_max?: number;
  curve_plus_5?: number;
  curve_zero?: number;
  curve_minus_5?: number;
  heatstop?: number;
  integral_setting?: number;
}

// Mode mapping for display
export const HeatpumpMode: Record<number, string> = {
  0: 'Off',
  1: 'Heating',
  2: 'Cooling',
  3: 'Auto',
};
