import type { Me } from '../types'
import { useSlots } from '../hooks/useSlots'
import SlotList from './SlotList'
import AdminPanel from './AdminPanel'
import CalendarInfo from './CalendarInfo'

interface Props {
  me: Me
  onError: (error: Error) => void
}

export default function BookingView({ me, onError }: Props) {
  const slots = useSlots()

  if (slots.isLoading) {
    return <div className="page-message">Laddar…</div>
  }
  if (slots.isError || !slots.data) {
    return (
      <div className="page-message">
        Kunde inte ladda sidan. Prova att ladda om.
      </div>
    )
  }

  return (
    <>
      {me.is_admin && <AdminPanel onError={onError} />}
      <SlotList slots={slots.data} me={me} onError={onError} />
      <CalendarInfo />
    </>
  )
}
