-- Migration: 002_korschema
-- Description: Tables for the korschema module - a 30-lesson driving
-- curriculum. Static content (phases, lessons, exercises, discussion
-- topics, milestones, info sections) is seeded by 003_korschema_seed.sql;
-- dynamic per-student state lives in korschema_exercise_checks and
-- korschema_lesson_notes. Students are identified by the same proxy-auth
-- username the slots table uses (no user table exists); the valid student
-- set is the app.students list in gasa-api's config.
--
-- Static content uses generated PKs with a stable natural key (ord /
-- lesson_id+ord) that the seed upserts on, so re-seeding never reassigns
-- the ids that korschema_exercise_checks reference.
--
-- All statements are idempotent: this script re-runs on every Flux sync.

CREATE TABLE IF NOT EXISTS korschema_phases (
  id SERIAL PRIMARY KEY,
  ord INTEGER NOT NULL UNIQUE,
  name TEXT NOT NULL,
  meta TEXT NOT NULL,
  intro TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS korschema_lessons (
  id SERIAL PRIMARY KEY,
  phase_id INTEGER NOT NULL REFERENCES korschema_phases (id),
  -- Global lesson number 1-30
  ord INTEGER NOT NULL UNIQUE,
  title TEXT NOT NULL,
  subtitle TEXT NOT NULL,
  goal TEXT NOT NULL,
  -- Handledartips: only ever serialized to admins, see korschema.rs
  tip TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS korschema_exercises (
  id SERIAL PRIMARY KEY,
  lesson_id INTEGER NOT NULL REFERENCES korschema_lessons (id),
  ord INTEGER NOT NULL,
  text TEXT NOT NULL,
  UNIQUE (lesson_id, ord)
);

CREATE TABLE IF NOT EXISTS korschema_discussion_topics (
  id SERIAL PRIMARY KEY,
  lesson_id INTEGER NOT NULL REFERENCES korschema_lessons (id),
  ord INTEGER NOT NULL,
  text TEXT NOT NULL,
  UNIQUE (lesson_id, ord)
);

CREATE TABLE IF NOT EXISTS korschema_milestones (
  id SERIAL PRIMARY KEY,
  after_phase INTEGER NOT NULL UNIQUE,
  icon TEXT NOT NULL,
  title TEXT NOT NULL,
  text TEXT NOT NULL
);

-- "beforeYouStart" / "assessment": nested JSON passed through to the
-- frontend as-is, so no reason to model it relationally.
CREATE TABLE IF NOT EXISTS korschema_static_sections (
  key TEXT PRIMARY KEY,
  content JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS korschema_exercise_checks (
  student TEXT NOT NULL,
  exercise_id INTEGER NOT NULL REFERENCES korschema_exercises (id) ON DELETE CASCADE,
  -- Always an admin username: set_check is admin-gated in korschema.rs
  checked_by TEXT NOT NULL,
  checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (student, exercise_id)
);

CREATE TABLE IF NOT EXISTS korschema_lesson_notes (
  id BIGSERIAL PRIMARY KEY,
  student TEXT NOT NULL,
  lesson_id INTEGER NOT NULL REFERENCES korschema_lessons (id) ON DELETE CASCADE,
  author TEXT NOT NULL,
  text TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_korschema_notes_student
  ON korschema_lesson_notes (student, lesson_id);

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (2, 'korschema')
ON CONFLICT (version) DO NOTHING;
