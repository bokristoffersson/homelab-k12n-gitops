import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../api'
import type { CalendarUrls, Me, Slot } from '../types'

export function useMe() {
  return useQuery({
    queryKey: ['me'],
    queryFn: () => api.get<Me>('/api/me'),
    staleTime: Infinity,
  })
}

export function useSlots() {
  return useQuery({
    queryKey: ['slots'],
    queryFn: () => api.get<Slot[]>('/api/slots'),
    refetchInterval: 60_000,
  })
}

export function useCalendarUrls() {
  return useQuery({
    queryKey: ['calendar-url'],
    queryFn: () => api.get<CalendarUrls>('/api/calendar-url'),
    staleTime: Infinity,
  })
}

function useInvalidatingMutation<TArgs>(
  mutationFn: (args: TArgs) => Promise<unknown>,
  onError?: (error: Error) => void,
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSettled: () => queryClient.invalidateQueries({ queryKey: ['slots'] }),
    onError,
  })
}

export function useBookSlot(onError?: (error: Error) => void) {
  return useInvalidatingMutation(
    (id: number) => api.post<Slot>(`/api/slots/${id}/book`),
    onError,
  )
}

export function useCancelBooking(onError?: (error: Error) => void) {
  return useInvalidatingMutation(
    (id: number) => api.delete<Slot>(`/api/slots/${id}/book`),
    onError,
  )
}

export interface NewSlot {
  starts_at: string
  ends_at: string
  note?: string
}

export function useCreateSlots(onError?: (error: Error) => void) {
  return useInvalidatingMutation(async (slots: NewSlot[]) => {
    for (const slot of slots) {
      await api.post<Slot>('/api/slots', slot)
    }
  }, onError)
}

export function useDeleteSlot(onError?: (error: Error) => void) {
  return useInvalidatingMutation(
    (id: number) => api.delete<void>(`/api/slots/${id}`),
    onError,
  )
}
