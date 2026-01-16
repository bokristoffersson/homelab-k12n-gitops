import { api } from './api';
import { OutboxResponse } from '../types/outbox';

export const getOutboxEntries = async (deviceId: string): Promise<OutboxResponse> => {
  const response = await api.get<OutboxResponse>(`/api/v1/heatpump/settings/${deviceId}/outbox`);
  return response.data;
};
