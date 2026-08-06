import type { Check, Lesson, Schedule } from './types'

export interface ProgressSummary {
  totalExercises: number
  checkedExercises: number
  percent: number
  totalLessons: number
  doneLessons: number
  /** First lesson (in order) whose exercises are not all checked; null when done. */
  nextLesson: Lesson | null
  /** Lesson containing the most recently checked exercise, for the admin overview. */
  latestCheckedLesson: Lesson | null
}

export function isLessonDone(lesson: Lesson, checked: Set<number>): boolean {
  return (
    lesson.exercises.length > 0 &&
    lesson.exercises.every((e) => checked.has(e.id))
  )
}

export function checkedSet(checks: Check[]): Set<number> {
  return new Set(checks.map((c) => c.exercise_id))
}

export function summarizeProgress(
  schedule: Schedule,
  checks: Check[],
): ProgressSummary {
  const checked = checkedSet(checks)
  const lessons = schedule.phases.flatMap((p) => p.lessons)
  const exerciseToLesson = new Map<number, Lesson>()
  for (const lesson of lessons) {
    for (const exercise of lesson.exercises) {
      exerciseToLesson.set(exercise.id, lesson)
    }
  }

  const totalExercises = exerciseToLesson.size
  // Count via the schedule so stray checks can never push progress past 100%
  const checkedExercises = [...checked].filter((id) =>
    exerciseToLesson.has(id),
  ).length

  let latestCheck: Check | null = null
  for (const check of checks) {
    if (!exerciseToLesson.has(check.exercise_id)) continue
    if (!latestCheck || check.checked_at > latestCheck.checked_at) {
      latestCheck = check
    }
  }

  return {
    totalExercises,
    checkedExercises,
    percent:
      totalExercises === 0
        ? 0
        : Math.round((100 * checkedExercises) / totalExercises),
    totalLessons: lessons.length,
    doneLessons: lessons.filter((l) => isLessonDone(l, checked)).length,
    nextLesson: lessons.find((l) => !isLessonDone(l, checked)) ?? null,
    latestCheckedLesson: latestCheck
      ? (exerciseToLesson.get(latestCheck.exercise_id) ?? null)
      : null,
  }
}
