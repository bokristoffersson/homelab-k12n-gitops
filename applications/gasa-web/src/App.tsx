import { useState } from 'react'
import { ApiError } from './api'
import { useMe, useSlots } from './hooks/useSlots'
import { errorMessage } from './format'
import SlotList from './components/SlotList'
import AdminPanel from './components/AdminPanel'
import CalendarInfo from './components/CalendarInfo'

export default function App() {
  const me = useMe()
  const slots = useSlots()
  const [error, setError] = useState<string | null>(null)

  const showError = (err: Error) => {
    const message =
      err instanceof ApiError ? errorMessage(err.code) : errorMessage('error')
    setError(message)
    window.setTimeout(() => setError(null), 5000)
  }

  if (me.isLoading || slots.isLoading) {
    return <div className="page-message">Laddar…</div>
  }
  if (me.isError || slots.isError || !me.data || !slots.data) {
    return (
      <div className="page-message">
        Kunde inte ladda sidan. Prova att ladda om.
      </div>
    )
  }

  return (
    <div className="app">
      <header className="header">
        <div>
          <h1>Övningskörning</h1>
          <p className="subtitle">Boka körpass med bilen</p>
        </div>
        <span className="user-chip">{me.data.username}</span>
      </header>

      {error && <div className="toast">{error}</div>}

      {me.data.is_admin && <AdminPanel onError={showError} />}

      <SlotList slots={slots.data} me={me.data} onError={showError} />

      <CalendarInfo />
    </div>
  )
}
