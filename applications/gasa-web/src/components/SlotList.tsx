import type { Me, Slot } from '../types'
import { dayKey, formatDay } from '../format'
import SlotCard from './SlotCard'

interface Props {
  slots: Slot[]
  me: Me
  onError: (error: Error) => void
}

export default function SlotList({ slots, me, onError }: Props) {
  const now = Date.now()
  const upcoming = slots.filter((s) => new Date(s.ends_at).getTime() >= now)

  if (upcoming.length === 0) {
    return <p className="empty">Inga körpass upplagda ännu.</p>
  }

  const groups: { key: string; day: string; slots: Slot[] }[] = []
  for (const slot of upcoming) {
    const key = dayKey(slot.starts_at)
    const group = groups.find((g) => g.key === key)
    if (group) {
      group.slots.push(slot)
    } else {
      groups.push({ key, day: formatDay(slot.starts_at), slots: [slot] })
    }
  }

  return (
    <div className="slot-list">
      {groups.map((group) => (
        <section key={group.key}>
          <h2 className="day-heading">{group.day}</h2>
          {group.slots.map((slot) => (
            <SlotCard key={slot.id} slot={slot} me={me} onError={onError} />
          ))}
        </section>
      ))}
    </div>
  )
}
