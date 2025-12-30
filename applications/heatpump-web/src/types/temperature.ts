export interface TemperatureReading {
  time: string;
  device_id?: string;
  mac_address?: string;
  location?: string;
  temperature_c?: number;
  temperature_f?: number;
  humidity?: number;
  wifi_rssi?: number;
  battery_voltage?: number;
  battery_percent?: number;
}

export interface TemperatureLatest {
  time: string;
  location?: string;
  temperature_c?: number;
  humidity?: number;
  battery_percent?: number;
}
