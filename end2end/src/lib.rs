//! Shared e2e infrastructure: fixture generation with relative dates.
//!
//! Both the `gen-e2e-fixtures` binary and the Cucumber test suite
//! use these helpers so fixture data stays in one place.

use chrono::{Days, Local, NaiveDate};
use serde_json::{json, Value};

use std::fs;
use std::io::Result;
use std::path::Path;

/// Write the standard 8 task fixtures to `dir` with dates relative
/// to today.  Returns the number of fixtures written.
pub fn write_standard_fixtures(dir: &Path) -> Result<usize> {
    let today = Local::now().date_naive();
    let tasks = build_tasks(today);
    for (filename, value) in &tasks {
        let path = dir.join(filename);
        let json = serde_json::to_string_pretty(value).expect("serialize");
        fs::write(&path, json)?;
    }
    Ok(tasks.len())
}

/// Remove all `task-*.json` files from `dir`.
pub fn clean_task_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let dominated = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("task-") && n.ends_with(".json"));
        if dominated {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

// ── date helpers ─────────────────────────────────────────────────────

/// Format a date + time as `YYYY-MM-DDTHH:MM:SS+02:00`.
fn dt(base: NaiveDate, day_offset: i64, hour: u32, min: u32, sec: u32) -> String {
    let date = offset_date(base, day_offset);
    format!("{date}T{hour:02}:{min:02}:{sec:02}+02:00")
}

/// End-of-day timestamp (for due dates).
fn eod(base: NaiveDate, day_offset: i64) -> String {
    dt(base, day_offset, 23, 59, 59)
}

/// Start-of-day timestamp (for start dates).
fn sod(base: NaiveDate, day_offset: i64) -> String {
    dt(base, day_offset, 0, 0, 0)
}

fn offset_date(base: NaiveDate, offset: i64) -> NaiveDate {
    if offset >= 0 {
        base.checked_add_days(Days::new(offset as u64)).unwrap()
    } else {
        base.checked_sub_days(Days::new((-offset) as u64)).unwrap()
    }
}

// ── fixture definitions ──────────────────────────────────────────────
//
// Day offsets are relative to `today`.  The spread of author timestamps
// ensures that "Recent Changes" (today + 2 days) always has content.
//
// Today  :  tasks 2, 7, 8
// Yesterday:  tasks 3, 5
// −2 days :  tasks 1, 4, 6

fn build_tasks(d: NaiveDate) -> Vec<(&'static str, Value)> {
    vec![
        // ── 1  Steuererklärung  (ToDo, due +28d, started −18d, prio A) ──
        ("task-019700000001700180a1000000000001.json", json!({
            "summary": "Steuererklärung einreichen",
            "status": { "ToDo": { "since": dt(d, -63, 8, 0, 0) } },
            "category": "Finanzen",
            "priority": "A",
            "contexts": ["@computer"],
            "due_date":   { "date": eod(d, 28), "soft": false },
            "start_date": { "date": sod(d, -18), "soft": false },
            "time_estimate": "Hours2",
            "notes": "Belege scannen, Elster-Formular ausfüllen. Steuerberater-Kontakt: Müller & Partner.",
            "authors": { "e2e-test": [dt(d, -2, 10, 0, 0)] }
        })),

        // ── 2  Küchenhahn  (ToDo, due +1d, prio B) ─────────────────────
        ("task-019700000002700280a2000000000002.json", json!({
            "summary": "Tropfenden Küchenhahn reparieren",
            "status": { "ToDo": { "since": dt(d, -23, 9, 0, 0) } },
            "category": "Küche",
            "priority": "B",
            "contexts": ["@zuhause", "@wochenende"],
            "due_date": { "date": eod(d, 1), "soft": false },
            "time_estimate": "Min45",
            "notes": "Ersatzdichtung bei OBI kaufen. Hauptventil vorher abdrehen!",
            "authors": { "e2e-test": [dt(d, 0, 11, 0, 0)] }
        })),

        // ── 3  Zahnarzttermin  (ToDo, due −1d = overdue, prio A) ───────
        ("task-019700000003700380a3000000000003.json", json!({
            "summary": "Zahnarzttermin für die Kinder buchen",
            "status": { "ToDo": { "since": dt(d, -13, 8, 30, 0) } },
            "category": "Gesundheit",
            "priority": "A",
            "contexts": ["@telefon"],
            "due_date": { "date": eod(d, -1), "soft": false },
            "authors": { "e2e-test": [dt(d, -1, 12, 0, 0)] }
        })),

        // ── 4  Sommercamp  (ToDo, no date, estimate 1h) ────────────────
        ("task-019700000004700480a4000000000004.json", json!({
            "summary": "Sommercamp-Optionen recherchieren",
            "status": { "ToDo": { "since": dt(d, -28, 14, 0, 0) } },
            "category": "Kinder",
            "contexts": ["@computer"],
            "time_estimate": "Hours1",
            "authors": { "e2e-test": [dt(d, -2, 13, 0, 0)] }
        })),

        // ── 5  Garage  (ToDo, start −13d = ready, prio C) ──────────────
        ("task-019700000005700580a5000000000005.json", json!({
            "summary": "Garage entrümpeln",
            "status": { "ToDo": { "since": dt(d, -49, 10, 0, 0) } },
            "category": "Haus",
            "priority": "C",
            "contexts": ["@zuhause", "@wochenende"],
            "start_date": { "date": sod(d, -13), "soft": false },
            "time_estimate": "Day1",
            "notes": "Kartons für Spenden vorbereiten. Sperrmüll-Termin danach anmelden.",
            "authors": { "e2e-test": [dt(d, -1, 14, 0, 0)] }
        })),

        // ── 6  Kfz-Zulassung  (Done, due −3d) ──────────────────────────
        ("task-019700000006700680a6000000000006.json", json!({
            "summary": "Kfz-Zulassung erneuern",
            "status": { "Done": { "since": dt(d, -5, 15, 0, 0) } },
            "category": "Verwaltung",
            "priority": "A",
            "contexts": ["@besorgungen"],
            "due_date": { "date": eod(d, -3), "soft": false },
            "notes": "Online über i-Kfz erledigt. Bestätigung im Postfach.",
            "authors": { "e2e-test": [dt(d, -5, 15, 0, 0), dt(d, -2, 15, 0, 0)] }
        })),

        // ── 7  Spülmaschine  (Done, author today) ──────────────────────
        ("task-019700000007700780a7000000000007.json", json!({
            "summary": "Neue Spülmaschine bestellen",
            "status": { "Done": { "since": dt(d, -8, 18, 0, 0) } },
            "category": "Küche",
            "priority": "B",
            "notes": "Bosch SMV4HAX48E bestellt bei MediaMarkt.",
            "authors": { "e2e-test": [dt(d, -8, 18, 0, 0), dt(d, 0, 16, 0, 0)] }
        })),

        // ── 8  Frühjahrsputz  (ToDo, due +7d, soft) ────────────────────
        ("task-019700000008700880a8000000000008.json", json!({
            "summary": "Frühjahrsputz planen",
            "status": { "ToDo": { "since": dt(d, -32, 9, 0, 0) } },
            "category": "Haus",
            "priority": "B",
            "contexts": ["@zuhause", "@wochenende"],
            "due_date": { "date": eod(d, 7), "soft": true },
            "time_estimate": "Day2",
            "notes": "Fenster, Keller, Garten. Kinder einteilen!",
            "authors": { "e2e-test": [dt(d, 0, 17, 0, 0)] }
        })),
    ]
}
