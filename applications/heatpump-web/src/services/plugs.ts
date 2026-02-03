import { api } from './api';
import {
  PowerPlug,
  PlugsListResponse,
  PlugToggle,
  PowerPlugSchedule,
  SchedulesListResponse,
  ScheduleCreate,
} from '../types/plugs';

// Get all power plugs
export const getAllPlugs = async (): Promise<PowerPlug[]> => {
  const response = await api.get<PlugsListResponse>('/api/v1/plugs');
  return response.data.plugs;
};

// Toggle plug status (on/off)
export const togglePlug = async (plugId: string, status: boolean): Promise<PowerPlug> => {
  const response = await api.patch<PowerPlug>(`/api/v1/plugs/${plugId}`, {
    status,
  } as PlugToggle);
  return response.data;
};

// Get schedules for a plug
export const getSchedules = async (plugId: string): Promise<PowerPlugSchedule[]> => {
  const response = await api.get<SchedulesListResponse>(`/api/v1/plugs/${plugId}/schedules`);
  return response.data.schedules;
};

// Create a new schedule for a plug
export const createSchedule = async (
  plugId: string,
  schedule: ScheduleCreate
): Promise<PowerPlugSchedule> => {
  const response = await api.post<PowerPlugSchedule>(
    `/api/v1/plugs/${plugId}/schedules`,
    schedule
  );
  return response.data;
};

// Delete a schedule
export const deleteSchedule = async (plugId: string, scheduleId: number): Promise<void> => {
  await api.delete(`/api/v1/plugs/${plugId}/schedules/${scheduleId}`);
};
