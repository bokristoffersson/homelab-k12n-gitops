import { useState } from 'react'
import type { Check, Lesson, LessonNote } from '../types'
import { formatShortDate } from '../format'
import { useAddNote, useToggleCheck } from '../hooks/useKorschema'

interface Props {
  lesson: Lesson
  checks: Map<number, Check>
  notes: LessonNote[]
  done: boolean
  /** Admin annotates the selected student; students are read-only. */
  canCheck: boolean
  student: string
  onError: (error: Error) => void
}

export default function LessonCard({
  lesson,
  checks,
  notes,
  done,
  canCheck,
  student,
  onError,
}: Props) {
  const [open, setOpen] = useState(false)
  const [noteText, setNoteText] = useState('')
  const toggle = useToggleCheck(student, onError)
  const addNote = useAddNote(student, onError)

  const submitNote = (e: React.FormEvent) => {
    e.preventDefault()
    const text = noteText.trim()
    if (!text) return
    addNote.mutate(
      { lessonId: lesson.id, text },
      { onSuccess: () => setNoteText('') },
    )
  }

  return (
    <div className={`lesson-card ${done ? 'done' : ''}`}>
      <button
        type="button"
        className="lesson-head"
        onClick={() => setOpen(!open)}
        aria-expanded={open}
      >
        <span className={`lesson-badge ${done ? 'done' : ''}`}>
          {done ? '✓' : lesson.order}
        </span>
        <span className="lesson-titles">
          <span className="lesson-title">{lesson.title}</span>
          <span className="lesson-subtitle">{lesson.subtitle}</span>
        </span>
        <span className="lesson-chevron">{open ? '▾' : '▸'}</span>
      </button>

      {open && (
        <div className="lesson-body">
          <p className="lesson-goal">
            <strong>Mål:</strong> {lesson.goal}
          </p>

          <ul className="exercise-list">
            {lesson.exercises.map((exercise) => {
              const check = checks.get(exercise.id)
              return (
                <li key={exercise.id}>
                  <label className={`exercise ${canCheck ? 'checkable' : ''}`}>
                    <input
                      type="checkbox"
                      checked={check !== undefined}
                      disabled={!canCheck || toggle.isPending}
                      onChange={(e) =>
                        toggle.mutate({
                          exerciseId: exercise.id,
                          checked: e.target.checked,
                        })
                      }
                    />
                    <span className="exercise-text">
                      {exercise.text}
                      {check && (
                        <span className="exercise-date">
                          {' '}
                          · avbockad {formatShortDate(check.checked_at)}
                        </span>
                      )}
                    </span>
                  </label>
                </li>
              )
            })}
          </ul>

          <div className="discussion-box">
            <h4>💬 Förstå varför — att prata om</h4>
            <ul>
              {lesson.discussion_topics.map((topic) => (
                <li key={topic.id}>{topic.text}</li>
              ))}
            </ul>
          </div>

          {lesson.tip !== null && (
            <p className="tip-box">
              <strong>Handledartips:</strong> {lesson.tip}
            </p>
          )}

          {notes.length > 0 && (
            <div className="note-list">
              <h4>Anteckningar</h4>
              {notes.map((note) => (
                <p key={note.id} className="note">
                  <span className="note-meta">
                    {note.author} · {formatShortDate(note.created_at)}
                  </span>
                  {note.text}
                </p>
              ))}
            </div>
          )}

          <form className="note-form" onSubmit={submitNote}>
            <input
              type="text"
              value={noteText}
              placeholder="Skriv en anteckning om passet…"
              onChange={(e) => setNoteText(e.target.value)}
            />
            <button
              type="submit"
              className="btn"
              disabled={addNote.isPending || noteText.trim() === ''}
            >
              {addNote.isPending ? 'Sparar…' : 'Spara'}
            </button>
          </form>
        </div>
      )}
    </div>
  )
}
