//! Hevy's answers as short text for the model: a workout is a few lines, not a page of JSON
//! (every tool result is sent again on each later round of the turn).

use serde_json::{Map, Value};

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn n(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(|x| x.as_f64())
}

fn num(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{}", x as i64)
    } else {
        format!("{}", (x * 100.0).round() / 100.0)
    }
}

/// "2026-10-12 07:30" from an ISO 8601 time (as given, keeping its own offset).
fn when(iso: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(iso) {
        Ok(t) => t.format("%Y-%m-%d %H:%M").to_string(),
        Err(_) => iso.to_string(),
    }
}

fn minutes_between(start: &str, end: &str) -> Option<i64> {
    let (a, b) = (chrono::DateTime::parse_from_rfc3339(start).ok()?, chrono::DateTime::parse_from_rfc3339(end).ok()?);
    Some((b - a).num_minutes())
}

/// One set: "60kg×8", "×12", "5km in 25:00", "45s", with its kind when not a normal set.
fn set_text(set: &Value) -> String {
    let mut parts = Vec::new();
    match (n(set, "weight_kg"), n(set, "reps")) {
        (Some(w), Some(r)) => parts.push(format!("{}kg×{}", num(w), num(r))),
        (None, Some(r)) => parts.push(format!("×{}", num(r))),
        (Some(w), None) => parts.push(format!("{}kg", num(w))),
        (None, None) => {}
    }
    if let Some(range) = set.get("rep_range").filter(|r| !r.is_null()) {
        if let (Some(a), Some(b)) = (n(range, "start"), n(range, "end")) {
            parts.push(format!("{}-{} reps", num(a), num(b)));
        }
    }
    if let Some(d) = n(set, "distance_meters") {
        parts.push(if d >= 1000.0 { format!("{}km", num(d / 1000.0)) } else { format!("{}m", num(d)) });
    }
    if let Some(secs) = n(set, "duration_seconds") {
        let secs = secs as i64;
        parts.push(if secs >= 60 { format!("{}:{:02}", secs / 60, secs % 60) } else { format!("{secs}s") });
    }
    if let Some(rpe) = n(set, "rpe") {
        parts.push(format!("RPE {}", num(rpe)));
    }
    let kind = s(set, "type");
    if !kind.is_empty() && kind != "normal" {
        parts.push(format!("({kind})"));
    }
    if parts.is_empty() {
        "set".to_string()
    } else {
        parts.join(" ")
    }
}

/// Sets in order, repeats folded: "3×(60kg×8), 70kg×6".
fn sets_text(sets: &[Value]) -> String {
    let texts: Vec<String> = sets.iter().map(set_text).collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < texts.len() {
        let mut j = i + 1;
        while j < texts.len() && texts[j] == texts[i] {
            j += 1;
        }
        out.push(if j - i > 1 { format!("{}×({})", j - i, texts[i]) } else { texts[i].clone() });
        i = j;
    }
    out.join(", ")
}

fn exercises_text(exercises: &[Value], with_rest: bool) -> String {
    exercises
        .iter()
        .map(|e| {
            let sets = e.get("sets").and_then(|x| x.as_array()).map(|s| sets_text(s)).unwrap_or_default();
            let mut line = format!("  - {} [{}]: {sets}", s(e, "title"), s(e, "exercise_template_id"));
            if with_rest {
                if let Some(rest) = n(e, "rest_seconds") {
                    line.push_str(&format!("; rest {}s", num(rest)));
                }
            }
            if let Some(superset) = e.get("supersets_id").or_else(|| e.get("superset_id")).and_then(|x| x.as_i64()) {
                line.push_str(&format!("; superset {superset}"));
            }
            let notes = s(e, "notes");
            if !notes.is_empty() {
                line.push_str(&format!("; notes: {notes}"));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn workout(w: &Value) -> String {
    let (start, end) = (s(w, "start_time"), s(w, "end_time"));
    let mut head = format!("{} (id {}) — {}", s(w, "title"), s(w, "id"), when(start));
    if let Some(minutes) = minutes_between(start, end) {
        head.push_str(&format!(", {minutes} min"));
    }
    let description = s(w, "description");
    if !description.is_empty() {
        head.push_str(&format!("\n  {description}"));
    }
    let exercises = w.get("exercises").and_then(|x| x.as_array()).map(|e| exercises_text(e, false)).unwrap_or_default();
    format!("{head}\n{exercises}")
}

fn page_line(body: &Value) -> String {
    format!("(page {} of {})", body.get("page").and_then(|v| v.as_u64()).unwrap_or(1), body.get("page_count").and_then(|v| v.as_u64()).unwrap_or(1))
}

pub fn workouts_page(body: &Value) -> String {
    let list = body.get("workouts").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return "No workouts on this page.".to_string();
    }
    format!("{}\n{}", list.iter().map(workout).collect::<Vec<_>>().join("\n\n"), page_line(body))
}

pub fn workout_events(body: &Value) -> String {
    let events = body.get("events").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if events.is_empty() {
        return "No changes since then.".to_string();
    }
    let lines: Vec<String> = events
        .iter()
        .map(|e| match s(e, "type") {
            "deleted" => format!("Deleted workout {} at {}", s(e, "id"), when(s(e, "deleted_at"))),
            _ => format!("Updated: {}", e.get("workout").map(workout).unwrap_or_default()),
        })
        .collect();
    format!("{}\n{}", lines.join("\n\n"), page_line(body))
}

pub fn routine(r: &Value) -> String {
    let mut head = format!("{} (id {})", s(r, "title"), s(r, "id"));
    if let Some(folder) = r.get("folder_id").and_then(|x| x.as_i64()) {
        head.push_str(&format!(", folder {folder}"));
    }
    let notes = s(r, "notes");
    if !notes.is_empty() {
        head.push_str(&format!("\n  {notes}"));
    }
    let exercises = r.get("exercises").and_then(|x| x.as_array()).map(|e| exercises_text(e, true)).unwrap_or_default();
    format!("{head}\n{exercises}")
}

pub fn routines_page(body: &Value) -> String {
    let list = body.get("routines").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return "No routines on this page.".to_string();
    }
    format!("{}\n{}", list.iter().map(routine).collect::<Vec<_>>().join("\n\n"), page_line(body))
}

pub fn folders_page(body: &Value) -> String {
    let list = body.get("routine_folders").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return "No routine folders (routines go in My Routines).".to_string();
    }
    let lines: Vec<String> = list.iter().map(|f| format!("{} (id {})", s(f, "title"), f.get("id").map(|v| v.to_string()).unwrap_or_default())).collect();
    format!("{}\n{}", lines.join("\n"), page_line(body))
}

/// Exercises whose name has every word of `query` (and that work `muscle`), at most 25.
pub fn find_exercises(all: &[Value], query: Option<&str>, muscle: Option<&str>, custom_only: bool) -> String {
    let words: Vec<String> = query.unwrap_or("").to_lowercase().split_whitespace().map(str::to_string).collect();
    let muscle = muscle.map(|m| m.to_lowercase().replace(' ', "_"));
    let matches: Vec<&Value> = all
        .iter()
        .filter(|e| {
            let title = s(e, "title").to_lowercase();
            words.iter().all(|w| title.contains(w.as_str()))
        })
        .filter(|e| match &muscle {
            None => true,
            Some(m) => {
                s(e, "primary_muscle_group") == m
                    || e.get("secondary_muscle_groups").and_then(|x| x.as_array()).is_some_and(|g| g.iter().any(|x| x.as_str() == Some(m)))
            }
        })
        .filter(|e| !custom_only || e.get("is_custom").and_then(|x| x.as_bool()).unwrap_or(false))
        .collect();
    if matches.is_empty() {
        return "No exercise matches. Try fewer words, or create a custom exercise.".to_string();
    }
    let shown: Vec<String> = matches
        .iter()
        .take(25)
        .map(|e| {
            let custom = if e.get("is_custom").and_then(|x| x.as_bool()).unwrap_or(false) { ", custom" } else { "" };
            format!("{} — id {} ({}, {}{custom})", s(e, "title"), s(e, "id"), s(e, "type"), s(e, "primary_muscle_group"))
        })
        .collect();
    let more = if matches.len() > 25 { format!("\n…and {} more; narrow the search.", matches.len() - 25) } else { String::new() };
    format!("{}{more}", shown.join("\n"))
}

/// Sets grouped by workout, oldest first, with the best set and total volume.
pub fn exercise_history(body: &Value) -> String {
    let entries = body.get("exercise_history").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if entries.is_empty() {
        return "No sets of this exercise logged in that time.".to_string();
    }
    let mut groups: Vec<(String, String, Vec<Value>)> = Vec::new();
    for e in &entries {
        let id = s(e, "workout_id").to_string();
        match groups.last_mut() {
            Some((last, _, sets)) if *last == id => sets.push(e.clone()),
            _ => groups.push((id, format!("{} {}", when(s(e, "workout_start_time")).get(..10).unwrap_or(""), s(e, "workout_title")), vec![e.clone()])),
        }
    }
    groups.sort_by(|a, b| a.1.cmp(&b.1));
    let mut best: Option<(f64, f64)> = None;
    let mut volume = 0.0;
    let lines: Vec<String> = groups
        .iter()
        .map(|(_, label, sets)| {
            for set in sets {
                if let (Some(w), Some(r)) = (n(set, "weight_kg"), n(set, "reps")) {
                    volume += w * r;
                    if best.is_none_or(|(bw, br)| w > bw || (w == bw && r > br)) {
                        best = Some((w, r));
                    }
                }
            }
            let as_sets: Vec<Value> = sets
                .iter()
                .map(|set| {
                    let mut v = set.clone();
                    if let Some(kind) = set.get("set_type").cloned() {
                        v["type"] = kind;
                    }
                    v
                })
                .collect();
            format!("{label}: {}", sets_text(&as_sets))
        })
        .collect();
    let mut summary = format!("{} sessions, {} sets", groups.len(), entries.len());
    if let Some((w, r)) = best {
        summary.push_str(&format!("; heaviest set {}kg×{}", num(w), num(r)));
    }
    if volume > 0.0 {
        summary.push_str(&format!("; total volume {}kg", num(volume.round())));
    }
    format!("{summary}\n{}", lines.join("\n"))
}

pub fn measurement_values(values: &Map<String, Value>) -> String {
    values.iter().filter(|(k, v)| *k != "date" && !v.is_null()).map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", ")
}

pub fn measurements_page(body: &Value) -> String {
    let list = body.get("body_measurements").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return "No body measurements logged.".to_string();
    }
    let lines: Vec<String> = list
        .iter()
        .map(|m| format!("{}: {}", s(m, "date"), m.as_object().map(measurement_values).unwrap_or_default()))
        .collect();
    format!("{}\n{}", lines.join("\n"), page_line(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_workout_reads_as_a_few_lines_with_repeated_sets_folded() {
        let w = json!({
            "id": "w1", "title": "Push day", "start_time": "2026-10-12T07:30:00+07:00", "end_time": "2026-10-12T08:35:00+07:00",
            "exercises": [
                {"title": "Bench Press (Barbell)", "exercise_template_id": "79D0BB3A", "sets": [
                    {"type": "warmup", "weight_kg": 40, "reps": 10},
                    {"type": "normal", "weight_kg": 60, "reps": 8}, {"type": "normal", "weight_kg": 60, "reps": 8}, {"type": "normal", "weight_kg": 60, "reps": 8},
                    {"type": "normal", "weight_kg": 62.5, "reps": 6, "rpe": 9}
                ]},
                {"title": "Running", "exercise_template_id": "AC1BB830", "sets": [{"type": "normal", "distance_meters": 5000, "duration_seconds": 1500}]}
            ]
        });
        assert_eq!(
            workout(&w),
            "Push day (id w1) — 2026-10-12 07:30, 65 min\n  - Bench Press (Barbell) [79D0BB3A]: 40kg×10 (warmup), 3×(60kg×8), 62.5kg×6 RPE 9\n  - Running [AC1BB830]: 5km 25:00"
        );
    }

    #[test]
    fn exercise_search_matches_every_word_and_muscle() {
        let all = vec![
            json!({"id": "A", "title": "Bench Press (Barbell)", "type": "weight_reps", "primary_muscle_group": "chest", "secondary_muscle_groups": ["triceps"], "is_custom": false}),
            json!({"id": "B", "title": "Incline Bench Press (Dumbbell)", "type": "weight_reps", "primary_muscle_group": "chest", "is_custom": false}),
            json!({"id": "C", "title": "Squat (Barbell)", "type": "weight_reps", "primary_muscle_group": "quadriceps", "is_custom": true}),
        ];
        let found = find_exercises(&all, Some("bench barbell"), None, false);
        assert!(found.contains("id A") && !found.contains("id B"), "{found}");
        assert!(find_exercises(&all, None, Some("triceps"), false).contains("id A"));
        assert!(find_exercises(&all, None, None, true).contains("id C, ") || find_exercises(&all, None, None, true).contains("id C ("));
        assert!(find_exercises(&all, Some("deadlift"), None, false).starts_with("No exercise"));
    }

    #[test]
    fn history_sums_up_progress() {
        let body = json!({"exercise_history": [
            {"workout_id": "w1", "workout_title": "Push", "workout_start_time": "2026-10-01T07:00:00Z", "weight_kg": 60, "reps": 8, "set_type": "normal"},
            {"workout_id": "w1", "workout_title": "Push", "workout_start_time": "2026-10-01T07:00:00Z", "weight_kg": 60, "reps": 8, "set_type": "normal"},
            {"workout_id": "w2", "workout_title": "Push", "workout_start_time": "2026-10-08T07:00:00Z", "weight_kg": 65, "reps": 6, "set_type": "normal"}
        ]});
        let text = exercise_history(&body);
        assert!(text.starts_with("2 sessions, 3 sets; heaviest set 65kg×6; total volume 1350kg"), "{text}");
        assert!(text.contains("2026-10-01 Push: 2×(60kg×8)"));
    }
}
