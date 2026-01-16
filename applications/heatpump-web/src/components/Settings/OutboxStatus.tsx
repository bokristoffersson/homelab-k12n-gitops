import { useQuery } from '@tanstack/react-query';
import { getOutboxEntries } from '../../services/outbox';
import { OutboxEntry } from '../../types/outbox';
import './OutboxStatus.css';

interface OutboxStatusProps {
  deviceId: string;
}

export const OutboxStatus: React.FC<OutboxStatusProps> = ({ deviceId }) => {
  const { data, isLoading, error } = useQuery({
    queryKey: ['outbox', deviceId],
    queryFn: () => getOutboxEntries(deviceId),
    refetchInterval: 2000, // Poll every 2 seconds
    enabled: !!deviceId,
  });

  if (isLoading) {
    return <div className="outbox-status loading">Loading status...</div>;
  }

  if (error) {
    return <div className="outbox-status error">Failed to load status</div>;
  }

  const entries = data?.data || [];
  const latestEntry = entries.length > 0 ? entries[0] : null;

  if (!latestEntry) {
    return null;
  }

  return (
    <div className="outbox-status">
      <div className="status-header">
        <span className="status-label">Update Status:</span>
        <StatusBadge entry={latestEntry} />
      </div>
      {latestEntry.error_message && (
        <div className="status-error">
          <strong>Error:</strong> {latestEntry.error_message}
        </div>
      )}
      <div className="status-timeline">
        <TimelineItem
          label="Created"
          timestamp={latestEntry.created_at}
          active={true}
        />
        <TimelineItem
          label="Published"
          timestamp={latestEntry.published_at}
          active={latestEntry.status !== 'pending'}
        />
        <TimelineItem
          label="Confirmed"
          timestamp={latestEntry.confirmed_at}
          active={latestEntry.status === 'confirmed'}
        />
      </div>
    </div>
  );
};

interface StatusBadgeProps {
  entry: OutboxEntry;
}

const StatusBadge: React.FC<StatusBadgeProps> = ({ entry }) => {
  const getStatusInfo = (status: OutboxEntry['status']) => {
    switch (status) {
      case 'pending':
        return { className: 'status-pending', icon: '⏳', text: 'Pending' };
      case 'published':
        return { className: 'status-published', icon: '📤', text: 'Published' };
      case 'confirmed':
        return { className: 'status-confirmed', icon: '✓', text: 'Confirmed' };
      case 'failed':
        return { className: 'status-failed', icon: '✗', text: 'Failed' };
    }
  };

  const info = getStatusInfo(entry.status);

  return (
    <span className={`status-badge ${info.className}`}>
      <span className="status-icon">{info.icon}</span>
      <span className="status-text">{info.text}</span>
      {entry.retry_count > 0 && (
        <span className="retry-count">
          (Retry {entry.retry_count}/{entry.max_retries})
        </span>
      )}
    </span>
  );
};

interface TimelineItemProps {
  label: string;
  timestamp: string | null;
  active: boolean;
}

const TimelineItem: React.FC<TimelineItemProps> = ({ label, timestamp, active }) => {
  const formatTimestamp = (ts: string | null) => {
    if (!ts) return '-';
    const date = new Date(ts);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  return (
    <div className={`timeline-item ${active ? 'active' : 'inactive'}`}>
      <div className="timeline-dot" />
      <div className="timeline-content">
        <div className="timeline-label">{label}</div>
        <div className="timeline-timestamp">{formatTimestamp(timestamp)}</div>
      </div>
    </div>
  );
};
