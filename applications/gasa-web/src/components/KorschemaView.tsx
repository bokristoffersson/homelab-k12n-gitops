import { useState } from 'react'
import type { Check, LessonNote, Me, Progress, Schedule } from '../types'
import { ApiError } from '../api'
import {
  useAllProgress,
  useProgress,
  useSchedule,
  useStudents,
} from '../hooks/useKorschema'
import { checkedSet, isLessonDone, summarizeProgress } from '../korschema'
import LessonCard from './LessonCard'
import KorschemaInfoSections from './KorschemaInfoSections'

interface Props {
  me: Me
  onError: (error: Error) => void
}

export default function KorschemaView({ me, onError }: Props) {
  const schedule = useSchedule()
  const students = useStudents(me.is_admin)
  const [selected, setSelected] = useState<string | null>(null)

  const studentList = students.data ?? []
  const student = me.is_admin
    ? (selected ?? studentList[0]?.username ?? null)
    : me.username

  // Admin fetches every student's progress (overview + picker); a student
  // only their own. Both use the same query keys, so check/note mutations
  // made from the lesson cards invalidate the right caches.
  const allProgress = useAllProgress(me.is_admin ? studentList : [])
  const ownProgress = useProgress(me.is_admin ? null : student)

  if (schedule.isLoading || (me.is_admin && students.isLoading)) {
    return <div className="page-message">Laddar…</div>
  }
  if (!schedule.data) {
    return (
      <div className="page-message">
        Kunde inte ladda körschemat. Prova att ladda om.
      </div>
    )
  }
  if (!me.is_admin && ownProgress.isLoading) {
    return <div className="page-message">Laddar…</div>
  }
  if (me.is_admin && studentList.length === 0) {
    return (
      <div className="page-message">
        Inga elever är konfigurerade (app.students i gasa-api-config).
      </div>
    )
  }

  const notMapped =
    !me.is_admin &&
    ownProgress.error instanceof ApiError &&
    ownProgress.error.status === 404
  if (notMapped) {
    return (
      <div className="page-message">
        Det finns inget körschema upplagt för ditt konto.
      </div>
    )
  }

  const progress: Progress | undefined = me.is_admin
    ? allProgress[studentList.findIndex((s) => s.username === student)]?.data
    : ownProgress.data

  return (
    <div className="korschema">
      {me.is_admin && (
        <div className="student-tabs">
          {studentList.map((s, i) => {
            const data = allProgress[i]?.data
            const summary = data
              ? summarizeProgress(schedule.data, data.checks)
              : null
            return (
              <button
                key={s.username}
                type="button"
                className={`student-tab ${s.username === student ? 'active' : ''}`}
                onClick={() => setSelected(s.username)}
              >
                <span className="student-name">{s.display_name}</span>
                {summary && (
                  <span className="student-summary">
                    {summary.percent} % · {summary.doneLessons}/
                    {summary.totalLessons} pass
                    {summary.latestCheckedLesson && (
                      <> · senast: pass {summary.latestCheckedLesson.order}</>
                    )}
                  </span>
                )}
              </button>
            )
          })}
        </div>
      )}

      {progress ? (
        <ScheduleBody
          schedule={schedule.data}
          progress={progress}
          isAdmin={me.is_admin}
          onError={onError}
        />
      ) : (
        <div className="page-message">Laddar…</div>
      )}
    </div>
  )
}

function ScheduleBody({
  schedule,
  progress,
  isAdmin,
  onError,
}: {
  schedule: Schedule
  progress: Progress
  isAdmin: boolean
  onError: (error: Error) => void
}) {
  const summary = summarizeProgress(schedule, progress.checks)
  const checked = checkedSet(progress.checks)

  const checksById = new Map<number, Check>(
    progress.checks.map((c) => [c.exercise_id, c]),
  )
  const notesByLesson = new Map<number, LessonNote[]>()
  for (const note of progress.notes) {
    const list = notesByLesson.get(note.lesson_id) ?? []
    list.push(note)
    notesByLesson.set(note.lesson_id, list)
  }

  return (
    <>
      <section className="progress-card">
        <div className="progress-row">
          <span>
            {summary.doneLessons} av {summary.totalLessons} pass klara
          </span>
          <strong>{summary.percent} %</strong>
        </div>
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{ width: `${summary.percent}%` }}
          />
        </div>
        {summary.nextLesson ? (
          <div className="next-lesson">
            <h3>
              Nästa pass: {summary.nextLesson.order}.{' '}
              {summary.nextLesson.title}
            </h3>
            <p>
              <strong>Mål:</strong> {summary.nextLesson.goal}
            </p>
            <p className="next-lesson-prep">
              Förbered dig — fundera på det här innan passet:
            </p>
            <ul>
              {summary.nextLesson.discussion_topics.map((topic) => (
                <li key={topic.id}>{topic.text}</li>
              ))}
            </ul>
          </div>
        ) : (
          <p className="next-lesson-done">
            Alla pass är avklarade — dags för uppkörning!
          </p>
        )}
      </section>

      <KorschemaInfoSections
        beforeYouStart={schedule.static_sections.beforeYouStart}
        assessment={schedule.static_sections.assessment}
      />

      {schedule.phases.map((phase) => (
        <section key={phase.id} className="phase">
          <h2 className="phase-name">
            Fas {phase.order}: {phase.name}
          </h2>
          <p className="phase-meta">{phase.meta}</p>
          <p className="phase-intro">{phase.intro}</p>
          {phase.lessons.map((lesson) => (
            <LessonCard
              key={lesson.id}
              lesson={lesson}
              checks={checksById}
              notes={notesByLesson.get(lesson.id) ?? []}
              done={isLessonDone(lesson, checked)}
              isAdmin={isAdmin}
              student={progress.student}
              onError={onError}
            />
          ))}
          {schedule.milestones
            .filter((m) => m.after_phase === phase.order)
            .map((milestone) => (
              <div key={milestone.id} className="milestone-card">
                <h3>
                  {milestone.icon} {milestone.title}
                </h3>
                <p>{milestone.text}</p>
              </div>
            ))}
        </section>
      ))}
    </>
  )
}
