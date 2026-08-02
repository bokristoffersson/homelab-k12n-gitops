export interface Me {
  username: string
  email: string | null
  is_admin: boolean
}

export interface BookedBy {
  username: string
  display_name: string
}

export interface Slot {
  id: number
  starts_at: string
  ends_at: string
  note: string | null
  booked_by: BookedBy | null
  booked_at: string | null
}

export interface CalendarUrls {
  webcal_url: string
  https_url: string
}
