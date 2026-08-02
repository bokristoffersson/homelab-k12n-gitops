import { useState } from 'react'
import { useCalendarUrls } from '../hooks/useSlots'

export default function CalendarInfo() {
  const urls = useCalendarUrls()
  const [copied, setCopied] = useState(false)

  if (!urls.data) {
    return null
  }

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(urls.data.webcal_url)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 3000)
    } catch {
      // Clipboard unavailable; the link itself still works
    }
  }

  return (
    <section className="calendar-info">
      <h2>Prenumerera i kalendern</h2>
      <p>
        Lägg till kalendern i telefonen så ser du alla pass direkt.{' '}
        <strong>iPhone:</strong> tryck på länken nedan.{' '}
        <strong>Android/Google:</strong> kopiera länken och lägg till den under
        &quot;Lägg till kalender – Via webbadress&quot; i Google Kalender.
      </p>
      <div className="calendar-actions">
        <a className="btn btn-primary" href={urls.data.webcal_url}>
          Prenumerera
        </a>
        <button className="btn" onClick={copy}>
          {copied ? 'Kopierad!' : 'Kopiera länk'}
        </button>
      </div>
    </section>
  )
}
