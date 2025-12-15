export interface HeatpumpStatus {
  ts: string;
  device_id: string | null;
  compressor_on: boolean | null;
  hotwater_production: boolean | null;
  flowlinepump_on: boolean | null;
  brinepump_on: boolean | null;
  aux_heater_3kw_on: boolean | null;
  aux_heater_6kw_on: boolean | null;
  outdoor_temp: number | null;
  supplyline_temp: number | null;
  returnline_temp: number | null;
  hotwater_temp: number | null;
  brine_out_temp: number | null;
  brine_in_temp: number | null;
}



