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

// --- Körschema -------------------------------------------------------------

export interface KorschemaItem {
  id: number
  text: string
}

export interface Lesson {
  id: number
  order: number
  title: string
  subtitle: string
  goal: string
  /** Handledartips - null unless the caller is admin. */
  tip: string | null
  exercises: KorschemaItem[]
  discussion_topics: KorschemaItem[]
}

export interface Phase {
  id: number
  order: number
  name: string
  meta: string
  intro: string
  lessons: Lesson[]
}

export interface Milestone {
  id: number
  after_phase: number
  icon: string
  title: string
  text: string
}

export interface Requirement {
  title: string
  text: string
}

export interface BeforeYouStart {
  title: string
  requirements: Requirement[]
  beforeExam: string[]
}

export interface AssessmentArea {
  title: string
  text: string
}

export interface Assessment {
  title: string
  intro: string
  areas: AssessmentArea[]
  outro: string
}

export interface Schedule {
  phases: Phase[]
  milestones: Milestone[]
  static_sections: {
    beforeYouStart?: BeforeYouStart
    assessment?: Assessment
  }
}

export interface Student {
  username: string
  display_name: string
}

export interface Check {
  exercise_id: number
  checked_by: string
  checked_at: string
}

export interface LessonNote {
  id: number
  lesson_id: number
  author: string
  text: string
  created_at: string
}

export interface Progress {
  student: string
  checks: Check[]
  notes: LessonNote[]
}
