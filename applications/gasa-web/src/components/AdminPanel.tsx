import { useState } from 'react'
import { useCreateSlots, type NewSlot } from '../hooks/useSlots'

interface Props {
  onError: (error: Error) => void
}

export default function AdminPanel({ onError }: Props) {
  const [open, setOpen] = useState(false)
  const [date, setDate] = useState('')
  const [start, setStart] = useState('19:00')
  const [end, setEnd] = useState('20:00')
  const [note, setNote] = useState('')
  const [repeatWeeks, setRepeatWeeks] = useState(1)
  const create = useCreateSlots(onError)

  const submit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!date || !start || !end) return

    const slots: NewSlot[] = []
    for (let week = 0; week < repeatWeeks; week++) {
      const startsAt = new Date(`${date}T${start}`)
      const endsAt = new Date(`${date}T${end}`)
      startsAt.setDate(startsAt.getDate() + week * 7)
      endsAt.setDate(endsAt.getDate() + week * 7)
      slots.push({
        starts_at: startsAt.toISOString(),
        ends_at: endsAt.toISOString(),
        note: note.trim() || undefined,
      })
    }
    create.mutate(slots, {
      onSuccess: () => {
        setDate('')
        setNote('')
        setRepeatWeeks(1)
        setOpen(false)
      },
    })
  }

  if (!open) {
    return (
      <button className="btn btn-primary admin-toggle" onClick={() => setOpen(true)}>
        Nytt körpass
      </button>
    )
  }

  return (
    <form className="admin-panel" onSubmit={submit}>
      <h2>Nytt körpass</h2>
      <div className="form-row">
        <label>
          Datum
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
            required
          />
        </label>
        <label>
          Start
          <input
            type="time"
            value={start}
            onChange={(e) => setStart(e.target.value)}
            required
          />
        </label>
        <label>
          Slut
          <input
            type="time"
            value={end}
            onChange={(e) => setEnd(e.target.value)}
            required
          />
        </label>
      </div>
      <label>
        Anteckning (valfritt)
        <input
          type="text"
          value={note}
          placeholder="t.ex. motorvägskörning"
          onChange={(e) => setNote(e.target.value)}
        />
      </label>
      <label>
        Upprepa varje vecka (antal veckor)
        <input
          type="number"
          min={1}
          max={12}
          value={repeatWeeks}
          onChange={(e) => setRepeatWeeks(Number(e.target.value))}
        />
      </label>
      <div className="form-actions">
        <button
          type="submit"
          className="btn btn-primary"
          disabled={create.isPending}
        >
          {create.isPending ? 'Lägger upp…' : 'Lägg upp'}
        </button>
        <button type="button" className="btn" onClick={() => setOpen(false)}>
          Avbryt
        </button>
      </div>
    </form>
  )
}
