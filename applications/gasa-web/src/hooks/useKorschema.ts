import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query'
import { api } from '../api'
import type { Check, LessonNote, Progress, Schedule, Student } from '../types'

export function useSchedule() {
  return useQuery({
    queryKey: ['korschema', 'schedule'],
    queryFn: () => api.get<Schedule>('/api/korschema/schedule'),
    staleTime: Infinity,
  })
}

export function useStudents(isAdmin: boolean) {
  return useQuery({
    queryKey: ['korschema', 'students'],
    queryFn: () => api.get<Student[]>('/api/korschema/students'),
    staleTime: Infinity,
    enabled: isAdmin,
  })
}

const progressKey = (student: string) => ['korschema', 'progress', student]

const progressQuery = (student: string) => ({
  queryKey: progressKey(student),
  queryFn: () => api.get<Progress>(`/api/korschema/progress/${student}`),
})

export function useProgress(student: string | null) {
  return useQuery({
    ...progressQuery(student ?? ''),
    enabled: student !== null,
  })
}

/** Admin overview: progress for every student at once. */
export function useAllProgress(students: Student[]) {
  return useQueries({
    queries: students.map((s) => progressQuery(s.username)),
  })
}

export function useToggleCheck(student: string, onError?: (e: Error) => void) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({
      exerciseId,
      checked,
    }: {
      exerciseId: number
      checked: boolean
    }) => {
      const path = `/api/korschema/progress/${student}/checks/${exerciseId}`
      if (checked) {
        await api.put<Check>(path)
      } else {
        await api.delete<void>(path)
      }
    },
    // Optimistic toggle so the admin can tick boxes rapid-fire in the car
    onMutate: async ({ exerciseId, checked }) => {
      await queryClient.cancelQueries({ queryKey: progressKey(student) })
      const previous = queryClient.getQueryData<Progress>(progressKey(student))
      if (previous) {
        const checks = checked
          ? [
              ...previous.checks,
              {
                exercise_id: exerciseId,
                checked_by: '',
                checked_at: new Date().toISOString(),
              },
            ]
          : previous.checks.filter((c) => c.exercise_id !== exerciseId)
        queryClient.setQueryData(progressKey(student), { ...previous, checks })
      }
      return { previous }
    },
    onError: (err, _vars, context) => {
      if (context?.previous) {
        queryClient.setQueryData(progressKey(student), context.previous)
      }
      onError?.(err)
    },
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: progressKey(student) }),
  })
}

export function useAddNote(student: string, onError?: (e: Error) => void) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ lessonId, text }: { lessonId: number; text: string }) =>
      api.post<LessonNote>(
        `/api/korschema/progress/${student}/notes/${lessonId}`,
        { text },
      ),
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: progressKey(student) }),
    onError,
  })
}
