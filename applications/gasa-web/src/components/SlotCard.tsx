import type { Me, Slot } from '../types'
import { formatTimeRange } from '../format'
import { useBookSlot, useCancelBooking, useDeleteSlot } from '../hooks/useSlots'

interface Props {
  slot: Slot
  me: Me
  onError: (error: Error) => void
}

export default function SlotCard({ slot, me, onError }: Props) {
  const book = useBookSlot(onError)
  const cancel = useCancelBooking(onError)
  const remove = useDeleteSlot(onError)

  const busy = book.isPending || cancel.isPending || remove.isPending
  const isMine = slot.booked_by?.username === me.username
  const canCancel = slot.booked_by !== null && (isMine || me.is_admin)

  return (
    <div className={`slot-card ${slot.booked_by ? 'booked' : 'free'}`}>
      <div className="slot-info">
        <span className="slot-time">
          {formatTimeRange(slot.starts_at, slot.ends_at)}
        </span>
        {slot.note && <span className="slot-note">{slot.note}</span>}
      </div>
      <div className="slot-actions">
        {slot.booked_by ? (
          <span className="chip chip-booked">
            {isMine ? 'Din bokning' : slot.booked_by.display_name}
          </span>
        ) : (
          <span className="chip chip-free">Ledig</span>
        )}
        {!slot.booked_by && (
          <button
            className="btn btn-primary"
            disabled={busy}
            onClick={() => book.mutate(slot.id)}
          >
            {book.isPending ? 'Bokar…' : 'Boka'}
          </button>
        )}
        {canCancel && (
          <button
            className="btn"
            disabled={busy}
            onClick={() => cancel.mutate(slot.id)}
          >
            {cancel.isPending ? 'Avbokar…' : 'Avboka'}
          </button>
        )}
        {me.is_admin && (
          <button
            className="btn btn-danger"
            disabled={busy}
            onClick={() => {
              if (window.confirm('Ta bort körpasset?')) {
                remove.mutate(slot.id)
              }
            }}
          >
            Ta bort
          </button>
        )}
      </div>
    </div>
  )
}
