import { useEffect, useState } from 'react'
import { ApiError } from './api'
import { useMe } from './hooks/useSlots'
import { errorMessage } from './format'
import BookingView from './components/BookingView'
import KorschemaView from './components/KorschemaView'

export default function App() {
  const me = useMe()
  const [error, setError] = useState<string | null>(null)
  const [hash, setHash] = useState(() => window.location.hash)

  useEffect(() => {
    const onHashChange = () => setHash(window.location.hash)
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  const showError = (err: Error) => {
    const message =
      err instanceof ApiError ? errorMessage(err.code) : errorMessage('error')
    setError(message)
    window.setTimeout(() => setError(null), 5000)
  }

  if (me.isLoading) {
    return <div className="page-message">Laddar…</div>
  }
  if (me.isError || !me.data) {
    return (
      <div className="page-message">
        Kunde inte ladda sidan. Prova att ladda om.
      </div>
    )
  }

  const showKorschema = hash === '#korschema'

  return (
    <div className="app">
      <header className="header">
        <div>
          <h1>Övningskörning</h1>
          <p className="subtitle">
            {showKorschema ? 'Körschema inför B-körkortet' : 'Boka körpass med bilen'}
          </p>
        </div>
        <span className="user-chip">{me.data.username}</span>
      </header>

      <nav className="tabs">
        <a className={`tab ${showKorschema ? '' : 'active'}`} href="#">
          Bokning
        </a>
        <a className={`tab ${showKorschema ? 'active' : ''}`} href="#korschema">
          Körschema
        </a>
      </nav>

      {error && <div className="toast">{error}</div>}

      {showKorschema ? (
        <KorschemaView me={me.data} onError={showError} />
      ) : (
        <BookingView me={me.data} onError={showError} />
      )}
    </div>
  )
}
