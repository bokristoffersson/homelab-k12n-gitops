export interface EnergyLatest {
  ts: string;
  consumption_total_w: number | null;
  consumption_total_actual_w: number | null;
  consumption_l1_w: number | null;
  consumption_l2_w: number | null;
  consumption_l3_w: number | null;
}

export interface HourlyTotal {
  total_kwh: number;
  hour_start: string;
  current_time: string;
}

export interface EnergyHourly {
  hour_start: string;
  hour_end: string;
  total_energy_kwh: number | null;
  total_energy_l1_kwh: number | null;
  total_energy_l2_kwh: number | null;
  total_energy_l3_kwh: number | null;
  total_energy_actual_kwh: number | null;
  measurement_count: number;
}



