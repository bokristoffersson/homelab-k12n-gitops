const dayFormat = new Intl.DateTimeFormat('sv-SE', {
  weekday: 'long',
  day: 'numeric',
  month: 'long',
})

const timeFormat = new Intl.DateTimeFormat('sv-SE', {
  hour: '2-digit',
  minute: '2-digit',
})

export function formatDay(iso: string): string {
  return dayFormat.format(new Date(iso))
}

export function formatTimeRange(startIso: string, endIso: string): string {
  return `${timeFormat.format(new Date(startIso))}–${timeFormat.format(new Date(endIso))}`
}

/** Group key so slots on the same local day render under one heading. */
export function dayKey(iso: string): string {
  const d = new Date(iso)
  return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`
}

export function errorMessage(code: string): string {
  switch (code) {
    case 'slot_taken':
      return 'Passet är redan bokat.'
    case 'slot_past':
      return 'Passet har redan varit.'
    case 'not_booked':
      return 'Passet är inte bokat.'
    case 'forbidden':
      return 'Du har inte behörighet för det här.'
    default:
      return 'Något gick fel. Försök igen.'
  }
}
