export interface OutboxEntry {
  id: number;
  aggregate_type: string;
  aggregate_id: string;
  event_type: string;
  payload: Record<string, unknown>;
  status: 'pending' | 'published' | 'confirmed' | 'failed';
  created_at: string;
  published_at: string | null;
  confirmed_at: string | null;
  error_message: string | null;
  retry_count: number;
  max_retries: number;
}

export interface OutboxResponse {
  data: OutboxEntry[];
}
