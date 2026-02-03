import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getAllPlugs,
  togglePlug,
  getSchedules,
  createSchedule,
  deleteSchedule,
} from '../../services/plugs';
import { PowerPlug, PowerPlugSchedule, ScheduleCreate } from '../../types/plugs';
import './Plugs.css';

function formatUptime(seconds: number | null): string {
  if (seconds === null) return 'N/A';
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`);

  return parts.join(' ');
}

function getSignalStrength(rssi: number | null): { label: string; className: string } {
  if (rssi === null) return { label: 'N/A', className: '' };
  if (rssi >= -50) return { label: `${rssi} dBm (Excellent)`, className: 'signal-good' };
  if (rssi >= -60) return { label: `${rssi} dBm (Good)`, className: 'signal-good' };
  if (rssi >= -70) return { label: `${rssi} dBm (Fair)`, className: 'signal-fair' };
  return { label: `${rssi} dBm (Poor)`, className: 'signal-poor' };
}

function PlugCard({ plug }: { plug: PowerPlug }) {
  const queryClient = useQueryClient();
  const [schedulesExpanded, setSchedulesExpanded] = useState(false);
  const [newScheduleTime, setNewScheduleTime] = useState('08:00');
  const [newScheduleAction, setNewScheduleAction] = useState<'on' | 'off'>('on');

  const {
    data: schedules,
    isLoading: schedulesLoading,
  } = useQuery({
    queryKey: ['plugs', plug.plug_id, 'schedules'],
    queryFn: () => getSchedules(plug.plug_id),
    enabled: schedulesExpanded,
  });

  const toggleMutation = useMutation({
    mutationFn: (newStatus: boolean) => togglePlug(plug.plug_id, newStatus),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugs'] });
    },
  });

  const createScheduleMutation = useMutation({
    mutationFn: (schedule: ScheduleCreate) => createSchedule(plug.plug_id, schedule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugs', plug.plug_id, 'schedules'] });
      setNewScheduleTime('08:00');
      setNewScheduleAction('on');
    },
  });

  const deleteScheduleMutation = useMutation({
    mutationFn: (scheduleId: number) => deleteSchedule(plug.plug_id, scheduleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugs', plug.plug_id, 'schedules'] });
    },
  });

  const handleToggle = () => {
    toggleMutation.mutate(!plug.status);
  };

  const handleAddSchedule = (e: React.FormEvent) => {
    e.preventDefault();
    createScheduleMutation.mutate({
      action: newScheduleAction,
      time_of_day: newScheduleTime,
      enabled: true,
    });
  };

  const handleDeleteSchedule = (scheduleId: number) => {
    if (window.confirm('Delete this schedule?')) {
      deleteScheduleMutation.mutate(scheduleId);
    }
  };

  const signal = getSignalStrength(plug.wifi_rssi);

  return (
    <div className="plug-card">
      <div className="plug-card-header">
        <div className="plug-info">
          <h3>{plug.name}</h3>
          <span className="plug-id">{plug.plug_id}</span>
        </div>
        <label className={`toggle-switch ${toggleMutation.isPending ? 'pending' : ''}`}>
          <input
            type="checkbox"
            checked={plug.status}
            onChange={handleToggle}
            disabled={toggleMutation.isPending}
          />
          <span className="toggle-slider" />
          <span className={`toggle-label ${plug.status ? 'on' : 'off'}`}>
            {plug.status ? 'ON' : 'OFF'}
          </span>
        </label>
      </div>

      <div className="plug-details">
        <div className="plug-detail">
          <span className="plug-detail-label">WiFi Signal</span>
          <span className={`plug-detail-value ${signal.className}`}>
            {signal.label}
          </span>
        </div>
        <div className="plug-detail">
          <span className="plug-detail-label">Uptime</span>
          <span className="plug-detail-value">{formatUptime(plug.uptime_seconds)}</span>
        </div>
        <div className="plug-detail">
          <span className="plug-detail-label">Last Updated</span>
          <span className="plug-detail-value">
            {new Date(plug.updated_at).toLocaleString()}
          </span>
        </div>
      </div>

      <div className="schedules-section">
        <div
          className="schedules-header"
          onClick={() => setSchedulesExpanded(!schedulesExpanded)}
        >
          <h4>Schedules</h4>
          <span className={`schedules-toggle ${schedulesExpanded ? 'expanded' : ''}`}>
            &#9660;
          </span>
        </div>

        {schedulesExpanded && (
          <div className="schedules-content">
            {schedulesLoading ? (
              <div className="loading">Loading schedules...</div>
            ) : (
              <>
                {schedules && schedules.length > 0 ? (
                  <div className="schedule-list">
                    {schedules
                      .sort((a, b) => a.time_of_day.localeCompare(b.time_of_day))
                      .map((schedule: PowerPlugSchedule) => (
                        <div
                          key={schedule.id}
                          className={`schedule-item ${!schedule.enabled ? 'disabled' : ''}`}
                        >
                          <div className="schedule-info">
                            <span className="schedule-time">{schedule.time_of_day}</span>
                            <span className={`schedule-action ${schedule.action}`}>
                              {schedule.action}
                            </span>
                          </div>
                          <button
                            className="schedule-delete"
                            onClick={() => handleDeleteSchedule(schedule.id)}
                            disabled={deleteScheduleMutation.isPending}
                            aria-label="Delete schedule"
                          >
                            &#10005;
                          </button>
                        </div>
                      ))}
                  </div>
                ) : (
                  <div className="no-schedules">No schedules configured</div>
                )}

                <form className="add-schedule-form" onSubmit={handleAddSchedule}>
                  <input
                    type="time"
                    className="schedule-input time-input"
                    value={newScheduleTime}
                    onChange={(e) => setNewScheduleTime(e.target.value)}
                    required
                  />
                  <select
                    className="schedule-input action-select"
                    value={newScheduleAction}
                    onChange={(e) => setNewScheduleAction(e.target.value as 'on' | 'off')}
                  >
                    <option value="on">ON</option>
                    <option value="off">OFF</option>
                  </select>
                  <button
                    type="submit"
                    className="add-schedule-button"
                    disabled={createScheduleMutation.isPending}
                  >
                    Add
                  </button>
                </form>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default function Plugs() {
  const {
    data: plugs,
    error,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ['plugs'],
    queryFn: getAllPlugs,
    refetchInterval: 30000,
    retry: 1,
  });

  if (isLoading) {
    return (
      <div className="plugs-page">
        <div className="plugs-header">
          <h2>Power Plugs</h2>
        </div>
        <div className="loading">Loading plugs...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="plugs-page">
        <div className="plugs-header">
          <h2>Power Plugs</h2>
        </div>
        <div className="error-banner">
          <strong>Error loading plugs:</strong>
          <div>
            {error instanceof Error
              ? (error as any).response?.data?.error || error.message
              : 'Failed to load plugs'}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="plugs-page">
      <div className="plugs-header">
        <h2>Power Plugs</h2>
        <button className="refresh-button" onClick={() => refetch()}>
          Refresh
        </button>
      </div>

      {!plugs || plugs.length === 0 ? (
        <div className="no-data">No power plugs found</div>
      ) : (
        <div className="plugs-grid">
          {plugs.map((plug) => (
            <PlugCard key={plug.plug_id} plug={plug} />
          ))}
        </div>
      )}
    </div>
  );
}
