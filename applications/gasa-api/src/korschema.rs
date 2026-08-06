//! Körschema module: a 30-lesson driving curriculum with per-student
//! progress. Static content is seeded into korschema_* tables by the
//! migrations; this module serves it and manages the dynamic state
//! (exercise checks and lesson notes).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::{auth::CurrentUser, error::AppError, handlers::AppState, models::display_name};

// --- Permissions -----------------------------------------------------------

/// Only the handledare (admin) may check exercises off. This is a deliberate
/// pedagogical choice; relax this single line if students should self-check.
fn can_edit_checks(user: &CurrentUser) -> bool {
    user.is_admin
}

/// A student may see and annotate their own progress; admin may see everyone's.
fn can_access_student(user: &CurrentUser, student: &str) -> bool {
    user.is_admin || user.username == student
}

// --- Response models -------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub phases: Vec<PhaseResponse>,
    pub milestones: Vec<Milestone>,
    pub static_sections: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct PhaseResponse {
    pub id: i32,
    pub order: i32,
    pub name: String,
    pub meta: String,
    pub intro: String,
    pub lessons: Vec<LessonResponse>,
}

#[derive(Debug, Serialize)]
pub struct LessonResponse {
    pub id: i32,
    pub order: i32,
    pub title: String,
    pub subtitle: String,
    pub goal: String,
    /// Handledartips: only serialized for admins, always null for students.
    pub tip: Option<String>,
    pub exercises: Vec<Item>,
    pub discussion_topics: Vec<Item>,
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub id: i32,
    pub text: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Milestone {
    pub id: i32,
    pub after_phase: i32,
    pub icon: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct StudentResponse {
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct ProgressResponse {
    pub student: String,
    pub checks: Vec<Check>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Check {
    pub exercise_id: i32,
    pub checked_by: String,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub lesson_id: i32,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NoteRequest {
    pub text: String,
}

// --- Repository ------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
struct PhaseRow {
    id: i32,
    ord: i32,
    name: String,
    meta: String,
    intro: String,
}

#[derive(Debug, sqlx::FromRow)]
struct LessonRow {
    id: i32,
    phase_id: i32,
    ord: i32,
    title: String,
    subtitle: String,
    goal: String,
    tip: String,
}

#[derive(Debug, sqlx::FromRow)]
struct ItemRow {
    id: i32,
    lesson_id: i32,
    text: String,
}

#[derive(Clone)]
pub struct KorschemaRepository {
    pool: PgPool,
}

impl KorschemaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load the full static schedule. `include_tips` controls whether the
    /// admin-only handledartips are included in the payload at all.
    pub async fn schedule(&self, include_tips: bool) -> Result<ScheduleResponse, sqlx::Error> {
        let phases = sqlx::query_as::<_, PhaseRow>(
            "SELECT id, ord, name, meta, intro FROM korschema_phases ORDER BY ord",
        )
        .fetch_all(&self.pool)
        .await?;
        let lessons = sqlx::query_as::<_, LessonRow>(
            "SELECT id, phase_id, ord, title, subtitle, goal, tip \
             FROM korschema_lessons ORDER BY ord",
        )
        .fetch_all(&self.pool)
        .await?;
        let exercises = sqlx::query_as::<_, ItemRow>(
            "SELECT id, lesson_id, text FROM korschema_exercises ORDER BY lesson_id, ord",
        )
        .fetch_all(&self.pool)
        .await?;
        let topics = sqlx::query_as::<_, ItemRow>(
            "SELECT id, lesson_id, text FROM korschema_discussion_topics ORDER BY lesson_id, ord",
        )
        .fetch_all(&self.pool)
        .await?;
        let milestones = sqlx::query_as::<_, Milestone>(
            "SELECT id, after_phase, icon, title, text FROM korschema_milestones ORDER BY after_phase",
        )
        .fetch_all(&self.pool)
        .await?;
        let sections = sqlx::query_as::<_, (String, Value)>(
            "SELECT key, content FROM korschema_static_sections",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(assemble_schedule(
            phases,
            lessons,
            exercises,
            topics,
            milestones,
            sections,
            include_tips,
        ))
    }

    pub async fn progress(&self, student: &str) -> Result<ProgressResponse, sqlx::Error> {
        let checks = sqlx::query_as::<_, Check>(
            "SELECT exercise_id, checked_by, checked_at \
             FROM korschema_exercise_checks WHERE student = $1 ORDER BY exercise_id",
        )
        .bind(student)
        .fetch_all(&self.pool)
        .await?;
        let notes = sqlx::query_as::<_, Note>(
            "SELECT id, lesson_id, author, text, created_at \
             FROM korschema_lesson_notes WHERE student = $1 ORDER BY created_at",
        )
        .bind(student)
        .fetch_all(&self.pool)
        .await?;
        Ok(ProgressResponse {
            student: student.to_string(),
            checks,
            notes,
        })
    }

    pub async fn set_check(
        &self,
        student: &str,
        exercise_id: i32,
        checked_by: &str,
    ) -> Result<Check, sqlx::Error> {
        sqlx::query_as::<_, Check>(
            "INSERT INTO korschema_exercise_checks (student, exercise_id, checked_by) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (student, exercise_id) \
             DO UPDATE SET checked_by = EXCLUDED.checked_by, checked_at = NOW() \
             RETURNING exercise_id, checked_by, checked_at",
        )
        .bind(student)
        .bind(exercise_id)
        .bind(checked_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn clear_check(&self, student: &str, exercise_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM korschema_exercise_checks WHERE student = $1 AND exercise_id = $2",
        )
        .bind(student)
        .bind(exercise_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_note(
        &self,
        student: &str,
        lesson_id: i32,
        author: &str,
        text: &str,
    ) -> Result<Note, sqlx::Error> {
        sqlx::query_as::<_, Note>(
            "INSERT INTO korschema_lesson_notes (student, lesson_id, author, text) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, lesson_id, author, text, created_at",
        )
        .bind(student)
        .bind(lesson_id)
        .bind(author)
        .bind(text)
        .fetch_one(&self.pool)
        .await
    }
}

fn assemble_schedule(
    phases: Vec<PhaseRow>,
    lessons: Vec<LessonRow>,
    exercises: Vec<ItemRow>,
    topics: Vec<ItemRow>,
    milestones: Vec<Milestone>,
    sections: Vec<(String, Value)>,
    include_tips: bool,
) -> ScheduleResponse {
    let mut exercises_by_lesson: HashMap<i32, Vec<Item>> = HashMap::new();
    for row in exercises {
        exercises_by_lesson
            .entry(row.lesson_id)
            .or_default()
            .push(Item {
                id: row.id,
                text: row.text,
            });
    }
    let mut topics_by_lesson: HashMap<i32, Vec<Item>> = HashMap::new();
    for row in topics {
        topics_by_lesson
            .entry(row.lesson_id)
            .or_default()
            .push(Item {
                id: row.id,
                text: row.text,
            });
    }
    let mut lessons_by_phase: HashMap<i32, Vec<LessonResponse>> = HashMap::new();
    for row in lessons {
        lessons_by_phase
            .entry(row.phase_id)
            .or_default()
            .push(LessonResponse {
                id: row.id,
                order: row.ord,
                title: row.title,
                subtitle: row.subtitle,
                goal: row.goal,
                tip: include_tips.then_some(row.tip),
                exercises: exercises_by_lesson.remove(&row.id).unwrap_or_default(),
                discussion_topics: topics_by_lesson.remove(&row.id).unwrap_or_default(),
            });
    }
    ScheduleResponse {
        phases: phases
            .into_iter()
            .map(|p| PhaseResponse {
                id: p.id,
                order: p.ord,
                name: p.name,
                meta: p.meta,
                intro: p.intro,
                lessons: lessons_by_phase.remove(&p.id).unwrap_or_default(),
            })
            .collect(),
        milestones,
        static_sections: sections.into_iter().collect(),
    }
}

/// An INSERT that violates the FK to a static content table means the client
/// referenced a nonexistent exercise/lesson: 404, not 500.
fn map_fk_violation(e: sqlx::Error) -> AppError {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => AppError::NotFound,
        _ => AppError::Database(e),
    }
}

// --- Handlers --------------------------------------------------------------

pub async fn schedule(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<ScheduleResponse>, AppError> {
    Ok(Json(state.korschema.schedule(user.is_admin).await?))
}

pub async fn students(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<StudentResponse>>, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    Ok(Json(
        state
            .config
            .app
            .students
            .iter()
            .map(|s| StudentResponse {
                username: s.clone(),
                display_name: display_name(s),
            })
            .collect(),
    ))
}

/// Match a raw student name against the configured list, returning the
/// configured spelling so database rows are always keyed consistently.
fn canonical_student<'a>(students: &'a [String], raw: &str) -> Option<&'a str> {
    let raw = raw.trim();
    students
        .iter()
        .find(|s| s.eq_ignore_ascii_case(raw))
        .map(String::as_str)
}

/// Resolve the {student} path segment: must be a configured student and the
/// caller must be allowed to access them.
fn authorize_student<'a>(
    state: &'a AppState,
    user: &CurrentUser,
    student: &str,
) -> Result<&'a str, AppError> {
    let canonical =
        canonical_student(&state.config.app.students, student).ok_or(AppError::NotFound)?;
    if !can_access_student(user, canonical) {
        return Err(AppError::Forbidden);
    }
    Ok(canonical)
}

pub async fn progress(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(student): Path<String>,
) -> Result<Json<ProgressResponse>, AppError> {
    let student = authorize_student(&state, &user, &student)?;
    Ok(Json(state.korschema.progress(student).await?))
}

pub async fn set_check(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path((student, exercise_id)): Path<(String, i32)>,
) -> Result<Json<Check>, AppError> {
    let student = authorize_student(&state, &user, &student)?;
    if !can_edit_checks(&user) {
        return Err(AppError::Forbidden);
    }
    let check = state
        .korschema
        .set_check(student, exercise_id, &user.username)
        .await
        .map_err(map_fk_violation)?;
    Ok(Json(check))
}

pub async fn clear_check(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path((student, exercise_id)): Path<(String, i32)>,
) -> Result<StatusCode, AppError> {
    let student = authorize_student(&state, &user, &student)?;
    if !can_edit_checks(&user) {
        return Err(AppError::Forbidden);
    }
    state.korschema.clear_check(student, exercise_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_note(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path((student, lesson_id)): Path<(String, i32)>,
    Json(req): Json<NoteRequest>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    let student = authorize_student(&state, &user, &student)?;
    let text = validate_note_text(&req.text).map_err(AppError::Validation)?;
    let note = state
        .korschema
        .add_note(student, lesson_id, &user.username, text)
        .await
        .map_err(map_fk_violation)?;
    Ok((StatusCode::CREATED, Json(note)))
}

fn validate_note_text(text: &str) -> Result<&str, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("text must not be empty".to_string());
    }
    if text.chars().count() > 2000 {
        return Err("text too long (max 2000 characters)".to_string());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(username: &str, is_admin: bool) -> CurrentUser {
        CurrentUser {
            username: username.to_string(),
            email: None,
            is_admin,
        }
    }

    #[test]
    fn test_only_admin_edits_checks() {
        assert!(can_edit_checks(&user("johanna", true)));
        assert!(!can_edit_checks(&user("arvid", false)));
    }

    #[test]
    fn test_student_access_matrix() {
        // Student sees their own progress
        assert!(can_access_student(&user("arvid", false), "arvid"));
        // ...but not their sibling's
        assert!(!can_access_student(&user("arvid", false), "ludvig"));
        // Admin sees everyone's
        assert!(can_access_student(&user("johanna", true), "arvid"));
        assert!(can_access_student(&user("johanna", true), "ludvig"));
    }

    #[test]
    fn test_canonical_student_normalizes() {
        let students = vec!["arvid".to_string(), "ludvig".to_string()];
        assert_eq!(canonical_student(&students, "arvid"), Some("arvid"));
        assert_eq!(canonical_student(&students, "Arvid"), Some("arvid"));
        assert_eq!(canonical_student(&students, " ludvig "), Some("ludvig"));
        assert_eq!(canonical_student(&students, "erik"), None);
        assert_eq!(canonical_student(&students, ""), None);
    }

    #[test]
    fn test_validate_note_text() {
        assert_eq!(validate_note_text("  gick bra  "), Ok("gick bra"));
        assert!(validate_note_text("").is_err());
        assert!(validate_note_text("   ").is_err());
        assert!(validate_note_text(&"x".repeat(2001)).is_err());
        assert!(validate_note_text(&"x".repeat(2000)).is_ok());
    }

    #[test]
    fn test_schedule_tip_stripped_for_students() {
        let phases = vec![PhaseRow {
            id: 1,
            ord: 1,
            name: "Fas".into(),
            meta: "meta".into(),
            intro: "intro".into(),
        }];
        let lessons = vec![LessonRow {
            id: 1,
            phase_id: 1,
            ord: 1,
            title: "t".into(),
            subtitle: "s".into(),
            goal: "g".into(),
            tip: "hemligt tips".into(),
        }];
        let exercises = vec![ItemRow {
            id: 11,
            lesson_id: 1,
            text: "ex".into(),
        }];

        let for_student =
            assemble_schedule(phases, lessons, exercises, vec![], vec![], vec![], false);
        assert_eq!(for_student.phases[0].lessons[0].tip, None);
        assert_eq!(for_student.phases[0].lessons[0].exercises.len(), 1);
    }

    #[test]
    fn test_schedule_tip_included_for_admin() {
        let phases = vec![PhaseRow {
            id: 1,
            ord: 1,
            name: "Fas".into(),
            meta: "meta".into(),
            intro: "intro".into(),
        }];
        let lessons = vec![LessonRow {
            id: 1,
            phase_id: 1,
            ord: 1,
            title: "t".into(),
            subtitle: "s".into(),
            goal: "g".into(),
            tip: "hemligt tips".into(),
        }];

        let for_admin = assemble_schedule(phases, lessons, vec![], vec![], vec![], vec![], true);
        assert_eq!(
            for_admin.phases[0].lessons[0].tip.as_deref(),
            Some("hemligt tips")
        );
    }
}
