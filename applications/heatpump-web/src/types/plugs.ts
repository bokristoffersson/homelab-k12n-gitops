export interface PowerPlug {
  plug_id: string;
  name: string;
  status: boolean;
  wifi_rssi: number | null;
  uptime_seconds: number | null;
  updated_at: string;
}

export interface PlugsListResponse {
  plugs: PowerPlug[];
}

export interface PlugToggle {
  status: boolean;
}

export interface PowerPlugSchedule {
  id: number;
  plug_id: string;
  action: 'on' | 'off';
  time_of_day: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface SchedulesListResponse {
  schedules: PowerPlugSchedule[];
}

export interface ScheduleCreate {
  action: 'on' | 'off';
  time_of_day: string;
  enabled?: boolean;
}
