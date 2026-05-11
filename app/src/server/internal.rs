cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use chrono::{Datelike, Days};
        use indexmap::IndexMap;
        use kid_types::{TaskDetails, TaskInfos};
        use kid_types::TaskAvailability;
        use kid_types::TaskCategory;
        use kid_types::TaskTimeEstimate;
        use std::collections::BTreeMap;
    }
}

use chrono::NaiveDate;

use kid_types::task;
use kid_types::Uuid;

use crate::DeadlineGroup;

pub(super) type UpcomingGroups = Vec<(DeadlineGroup, Vec<(Uuid, task::Infos, Option<NaiveDate>)>)>;
pub(super) type UpcomingBacklog = Vec<(Uuid, task::Infos)>;

/// Assign a deadline group based on the task's attention date.
///
/// Overdue is always based on the actual `due_date`. For non-overdue
/// tasks, the attention date (`due_date` minus `lead_days` eligible
/// days for the given `availability`) determines the bucket.
#[cfg(feature = "ssr")]
pub(super) fn deadline_group(
    due_date: NaiveDate,
    estimate: Option<TaskTimeEstimate>,
    availability: TaskAvailability,
    today: NaiveDate,
    this_sunday: NaiveDate,
    next_sunday: NaiveDate,
) -> DeadlineGroup {
    if due_date < today {
        return DeadlineGroup::Overdue;
    }

    // Attention date: due - lead eligible days.
    let group_date = match estimate {
        Some(est) if est.lead_days() > 0 => {
            let lead = est.lead_days();
            let mut remaining = lead;
            let mut date = due_date;
            while remaining > 0 {
                date = date.pred_opt().expect("date underflow");
                if availability.is_eligible(date) {
                    remaining -= 1;
                }
            }
            date
        }
        _ => due_date,
    };

    if group_date <= today {
        DeadlineGroup::Today
    } else if group_date <= this_sunday {
        DeadlineGroup::ThisWeek
    } else if group_date <= next_sunday {
        DeadlineGroup::NextWeek
    } else {
        DeadlineGroup::Later
    }
}

/// Group open tasks by temporal urgency for the Upcoming view.
///
/// Returns `(groups, backlog)` where `backlog` contains open tasks
/// without any date — unaffected by context filters (UXDR).
///
/// Each grouped task carries an optional attention date (shown as
/// "start by {date}" when the task landed in an earlier group than
/// its raw `due_date` would place it).
#[cfg(feature = "ssr")]
pub(super) fn group_upcoming<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
    today: NaiveDate,
) -> (UpcomingGroups, UpcomingBacklog) {
    // ISO 8601 week boundaries (Monday = start of week).
    let days_until_sunday = 6 - today.weekday().num_days_from_monday();
    let this_sunday = today + Days::new(days_until_sunday as u64);
    let next_sunday = this_sunday + Days::new(7);

    let mut backlog: UpcomingBacklog = Vec::new();
    // (id, info, group, sort_date, attention_label, soft)
    //
    // attention_label: Some when the task was shifted to an earlier
    //   group by lead-time or start_date, so the UI can show
    //   "start by {date}".
    // soft: true when the task landed in an earlier group because of
    //   start_date rather than attention_date — these sort after the
    //   "hard" attention-driven tasks within the same group.
    let mut items: Vec<(Uuid, task::Infos, DeadlineGroup, NaiveDate, Option<NaiveDate>, bool)> = Vec::new();

    for (id, task) in tasks {
        if task.info().is_done() {
            continue;
        }
        let due = task.due_date().map(|d| d.date.date_naive());
        let start = task.start_date().map(|d| d.date.date_naive());

        // Inclusion: open AND (has due_date OR start_date <= today).
        // Everything else is backlog.
        if due.is_none() && start.map_or(true, |s| s > today) {
            backlog.push((id.to_owned(), task.info().to_owned()));
            continue;
        }

        // Group by attention date; sort within groups by actual
        // due_date (UXDR).
        let (group, sort_date, attention_label, soft) = if let Some(due_date) = due {
            let est = task.time_estimate().copied();
            let avail = *task.availability();
            let attention_group = deadline_group(
                due_date, est, avail, today, this_sunday, next_sunday,
            );
            // Compute the raw attention date (without start_date
            // override) to decide whether to show the indicator.
            let attention_date = match est {
                Some(e) if e.lead_days() > 0 => {
                    let lead = e.lead_days();
                    let mut remaining = lead;
                    let mut d = due_date;
                    while remaining > 0 {
                        d = d.pred_opt().expect("date underflow");
                        if avail.is_eligible(d) { remaining -= 1; }
                    }
                    Some(d)
                }
                _ => None,
            };
            let effective_attention = attention_date.unwrap_or(due_date);

            // If start_date < attention_date, the user wants to begin
            // earlier than the computed lead time demands. Use
            // start_date for group assignment but mark as "soft" so
            // these sort after attention-driven tasks in the same group.
            let (group, soft) = match start {
                Some(sd) if sd < effective_attention && !matches!(attention_group, DeadlineGroup::Overdue) => {
                    let start_group = if sd <= today {
                        DeadlineGroup::Today
                    } else if sd <= this_sunday {
                        DeadlineGroup::ThisWeek
                    } else if sd <= next_sunday {
                        DeadlineGroup::NextWeek
                    } else {
                        DeadlineGroup::Later
                    };
                    // Only shift if start_date actually pulls into an
                    // earlier group; otherwise keep attention_group.
                    if (start_group as u8) < (attention_group as u8) {
                        (start_group, true)
                    } else {
                        (attention_group, false)
                    }
                }
                _ => (attention_group, false),
            };

            // Only show indicator when attention shifted the group.
            let label = attention_date.filter(|a| *a != due_date);
            (group, due_date, label, soft)
        } else {
            // start_date <= today, no due_date → Ready to Start
            (DeadlineGroup::ReadyToStart, start.unwrap(), None, false)
        };

        items.push((id.to_owned(), task.info().to_owned(), group, sort_date, attention_label, soft));
    }

    // Sort: group order, then within-group rules per UXDR.
    // Within each group, "hard" tasks (attention-driven) come before
    // "soft" tasks (start_date-driven).
    items.sort_by(|a, b| {
        (a.2 as u8).cmp(&(b.2 as u8))
            .then_with(|| a.5.cmp(&b.5))  // false < true → hard before soft
            .then_with(|| {
            let pri = |info: &task::Infos| -> u8 {
                info.priority().map(|p| *p as u8).unwrap_or(u8::MAX)
            };
            match a.2 {
                // Today: priority descending (A before B before C), then UUID.
                DeadlineGroup::Today => {
                    pri(&a.1).cmp(&pri(&b.1)).then_with(|| a.0.cmp(&b.0))
                }
                // All others: date ascending, then priority, then UUID.
                _ => {
                    a.3.cmp(&b.3)
                        .then_with(|| pri(&a.1).cmp(&pri(&b.1)))
                        .then_with(|| a.0.cmp(&b.0))
                }
            }
        })
    });

    // Chunk into contiguous groups (items are already in group order).
    let mut groups: UpcomingGroups = Vec::new();
    for (id, info, group, _, attention_label, _) in items {
        if groups.last().map_or(true, |(key, _)| *key != group) {
            groups.push((group, Vec::new()));
        }
        groups.last_mut().unwrap().1.push((id, info, attention_label));
    }

    // Sort backlog: priority descending (A first), then UUID (creation order).
    backlog.sort_by(|a, b| {
        let pri = |info: &task::Infos| -> u8 {
            info.priority().map(|p| *p as u8).unwrap_or(u8::MAX)
        };
        pri(&a.1).cmp(&pri(&b.1)).then_with(|| a.0.cmp(&b.0))
    });

    (groups, backlog)
}

#[cfg(feature = "ssr")]
pub(super) fn group_by_category(list: Vec<(Uuid, task::Infos)>) -> IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>> {
    let mut btree: BTreeMap<TaskCategory, Vec<(Uuid, task::Infos)>> = BTreeMap::new();
    for item in list {
        btree.entry(item.1.category().parse().unwrap()).or_default().push(item);
    }
    btree.into_iter().collect()
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use kid_types::TaskAvailability as Avail;
    use kid_types::TaskTimeEstimate as Est;

    /// Week anchors for a given `today`.
    fn week_bounds(today: NaiveDate) -> (NaiveDate, NaiveDate) {
        use chrono::Datelike;
        let days_until_sunday = 6 - today.weekday().num_days_from_monday();
        let this_sunday = today + chrono::Days::new(days_until_sunday as u64);
        let next_sunday = this_sunday + chrono::Days::new(7);
        (this_sunday, next_sunday)
    }

    fn group(
        due: &str,
        estimate: Option<Est>,
        availability: Avail,
        today: &str,
    ) -> DeadlineGroup {
        let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap();
        let due = NaiveDate::parse_from_str(due, "%Y-%m-%d").unwrap();
        let (this_sun, next_sun) = week_bounds(today);
        deadline_group(due, estimate, availability, today, this_sun, next_sun)
    }

    // ── Baseline: no estimate, pure due_date grouping ──

    #[test]
    fn overdue_task() {
        // Due yesterday → Overdue
        assert_eq!(
            group("2026-05-08", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Overdue,
        );
    }

    #[test]
    fn due_today_no_estimate() {
        assert_eq!(
            group("2026-05-09", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn due_this_week_no_estimate() {
        // Today is Sat May 9, due Sun May 10 (this week)
        assert_eq!(
            group("2026-05-10", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn due_next_week_no_estimate() {
        // Today is Sat May 9, next week Mon-Sun = May 11-17
        assert_eq!(
            group("2026-05-14", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn due_later_no_estimate() {
        // Today is Sat May 9, next_sunday = May 17
        // Due May 20 → Later
        assert_eq!(
            group("2026-05-20", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Later,
        );
    }

    // ── Attention date: Anytime ──

    #[test]
    fn day2_anytime_surfaces_earlier() {
        // Today Sat May 9 (this_sun=10, next_sun=17)
        // Due Tue May 19 → without estimate: Later (19 > 17)
        assert_eq!(
            group("2026-05-19", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Later,
        );
        // With Day2: attention = May 17 (Sun) → Next Week (17 <= 17)
        assert_eq!(
            group("2026-05-19", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn day1_anytime_surfaces_one_day_earlier() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Mon May 18 → without estimate: Next Week (18 <= 24)
        assert_eq!(
            group("2026-05-18", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
        // With Day1: attention = May 17 (Sun) → This Week (17 <= 17)
        assert_eq!(
            group("2026-05-18", Some(Est::Day1), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn halfday_anytime_no_lead() {
        // HalfDay → lead_days=0 → group_date = due_date → Later
        assert_eq!(
            group("2026-05-20", Some(Est::HalfDay), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Later,
        );
    }

    // ── Attention date: WeekdayOnly ──

    #[test]
    fn day2_weekday_only_skips_weekend() {
        // Today Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day2 WeekdayOnly:
        //   Sun 17 → skip, Sat 16 → skip,
        //   Fri 15 → 1, Thu 14 → 0
        //   attention = May 14 (Thu) → This Week (14 <= 17)
        assert_eq!(
            group("2026-05-18", Some(Est::Day2), Avail::WeekdayOnly, "2026-05-13"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekday_only_due_monday_from_prior_week() {
        // Today Sat May 9, due Mon May 18
        // Day2 WeekdayOnly: attention = Thu May 14 → Next Week
        assert_eq!(
            group("2026-05-18", Some(Est::Day2), Avail::WeekdayOnly, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Attention date: WeekendOnly ──

    #[test]
    fn day2_weekend_only_due_friday_jumps_to_prev_weekend() {
        // Today Sat May 9, due Fri May 22
        // Day2 WeekendOnly: count 2 weekend days back from Fri
        //   Thu→skip … Sun May 17 → 1, Sat May 16 → 0
        //   attention = May 16 → Next Week
        assert_eq!(
            group("2026-05-22", Some(Est::Day2), Avail::WeekendOnly, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn day1_weekend_only_due_friday() {
        // Today Sat May 9, due Fri May 22
        // Day1 WeekendOnly: count 1 weekend day back from Fri
        //   Thu→skip … Sun May 17 → 0
        //   attention = May 17 → Next Week
        assert_eq!(
            group("2026-05-22", Some(Est::Day1), Avail::WeekendOnly, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Attention date reaches today → Today group ──

    #[test]
    fn attention_in_the_past_becomes_today() {
        // Today Wed May 20, due Fri May 22
        // Day2 Anytime: attention = May 20 (today) → Today
        assert_eq!(
            group("2026-05-22", Some(Est::Day2), Avail::Anytime, "2026-05-20"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn attention_passed_still_today_not_overdue() {
        // Today Thu May 21, due Fri May 22
        // Day2 Anytime: attention = May 20 (yesterday) → Today
        // (not Overdue — due_date is still in the future)
        assert_eq!(
            group("2026-05-22", Some(Est::Day2), Avail::Anytime, "2026-05-21"),
            DeadlineGroup::Today,
        );
    }

    // ── Overdue ignores attention date ──

    #[test]
    fn overdue_regardless_of_estimate() {
        // Due yesterday, even with Day2 estimate → still Overdue
        assert_eq!(
            group("2026-05-08", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Overdue,
        );
    }

    #[test]
    fn overdue_with_constrained_availability() {
        assert_eq!(
            group("2026-05-08", Some(Est::Day1), Avail::WeekendOnly, "2026-05-09"),
            DeadlineGroup::Overdue,
        );
    }

    // ── Boundary: group edges ──

    #[test]
    fn due_on_this_sunday_is_this_week() {
        // Today Mon May 11 (this_sun=17)
        // Due Sun May 17 = this_sunday → ThisWeek
        assert_eq!(
            group("2026-05-17", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn due_on_next_sunday_is_next_week() {
        // Today Mon May 11 (next_sun=24)
        // Due Sun May 24 = next_sunday → NextWeek
        assert_eq!(
            group("2026-05-24", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn due_day_after_next_sunday_is_later() {
        // Today Mon May 11 (next_sun=24)
        // Due Mon May 25 → Later
        assert_eq!(
            group("2026-05-25", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::Later,
        );
    }

    #[test]
    fn due_tomorrow_is_this_week_not_today() {
        // Today Sat May 9 (this_sun=10)
        // Due Sun May 10 → ThisWeek (not Today)
        assert_eq!(
            group("2026-05-10", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn attention_on_this_sunday_is_this_week() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 19, Day2: attention = May 17 (Sun) = this_sunday
        // 17 <= 17 → ThisWeek
        assert_eq!(
            group("2026-05-19", Some(Est::Day2), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn attention_on_next_sunday_is_next_week() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 26, Day2: attention = May 24 (Sun) = next_sunday
        // 24 <= 24 → NextWeek
        assert_eq!(
            group("2026-05-26", Some(Est::Day2), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Today is Sunday: ThisWeek is empty ──

    #[test]
    fn today_is_sunday_due_tomorrow_is_next_week() {
        // Today Sun May 10 (this_sun=10, next_sun=17)
        // ThisWeek range is (today..this_sun] = empty.
        // Due Mon May 11: group_date > today and > this_sun → NextWeek
        assert_eq!(
            group("2026-05-11", None, Avail::Anytime, "2026-05-10"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn today_is_sunday_due_today_is_today() {
        // Today Sun May 10 (this_sun=10)
        assert_eq!(
            group("2026-05-10", None, Avail::Anytime, "2026-05-10"),
            DeadlineGroup::Today,
        );
    }

    // ── Availability has no effect without lead time ──

    #[test]
    fn no_estimate_ignores_availability() {
        // Without estimate, availability changes nothing.
        let base = group("2026-05-20", None, Avail::Anytime, "2026-05-09");
        assert_eq!(
            group("2026-05-20", None, Avail::WeekdayOnly, "2026-05-09"),
            base,
        );
        assert_eq!(
            group("2026-05-20", None, Avail::WeekendOnly, "2026-05-09"),
            base,
        );
    }

    #[test]
    fn sub_day_estimate_ignores_availability() {
        // lead_days=0 → group_date = due_date regardless of availability.
        let base = group("2026-05-20", Some(Est::Hours2), Avail::Anytime, "2026-05-09");
        assert_eq!(
            group("2026-05-20", Some(Est::Hours2), Avail::WeekdayOnly, "2026-05-09"),
            base,
        );
        assert_eq!(
            group("2026-05-20", Some(Est::Hours2), Avail::WeekendOnly, "2026-05-09"),
            base,
        );
    }

    #[test]
    fn all_sub_day_estimates_behave_identically() {
        // Min15..HalfDay all have lead_days=0 → same group as no estimate.
        let today = "2026-05-09";
        let due = "2026-05-20";
        let expected = group(due, None, Avail::Anytime, today);
        for est in [Est::Min15, Est::Min30, Est::Min45,
                    Est::Hours1, Est::Hours2, Est::HalfDay] {
            assert_eq!(
                group(due, Some(est), Avail::Anytime, today),
                expected,
                "estimate {:?} should match no-estimate grouping", est,
            );
        }
    }

    // ── Day1 + WeekdayOnly ──

    #[test]
    fn day1_weekday_only_due_monday_skips_weekend() {
        // Today Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day1 WeekdayOnly:
        //   Sun 17 → skip, Sat 16 → skip, Fri 15 → 0
        //   attention = Fri May 15 → ThisWeek (15 <= 17)
        assert_eq!(
            group("2026-05-18", Some(Est::Day1), Avail::WeekdayOnly, "2026-05-13"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day1_weekday_only_due_tuesday() {
        // Today Wed May 13 (this_sun=17)
        // Due Tue May 19, Day1 WeekdayOnly:
        //   Mon 18 → eligible → 0
        //   attention = Mon May 18 → NextWeek (18 > 17, 18 <= 24)
        assert_eq!(
            group("2026-05-19", Some(Est::Day1), Avail::WeekdayOnly, "2026-05-13"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── WeekendOnly edge cases ──

    #[test]
    fn day1_weekend_only_due_saturday_jumps_to_prev_sunday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sat May 23, Day1 WeekendOnly:
        //   Fri 22→skip, Thu 21→skip, Wed 20→skip,
        //   Tue 19→skip, Mon 18→skip, Sun 17→0
        //   attention = Sun May 17 → ThisWeek (17 <= 17)
        assert_eq!(
            group("2026-05-23", Some(Est::Day1), Avail::WeekendOnly, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_saturday_jumps_to_prev_saturday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sat May 23, Day2 WeekendOnly:
        //   Fri→skip … Sun 17→1, Sat 16→0
        //   attention = Sat May 16 → ThisWeek (16 <= 17)
        assert_eq!(
            group("2026-05-23", Some(Est::Day2), Avail::WeekendOnly, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_sunday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sun May 24, Day2 WeekendOnly:
        //   Sat 23→1, Fri→skip … Sun 17→0
        //   attention = Sun May 17 → ThisWeek (17 <= 17)
        assert_eq!(
            group("2026-05-24", Some(Est::Day2), Avail::WeekendOnly, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_monday_jumps_far_back() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Mon May 25, Day2 WeekendOnly:
        //   Sun 24→1, Sat 23→0
        //   attention = Sat May 23 → NextWeek (23 <= 24)
        assert_eq!(
            group("2026-05-25", Some(Est::Day2), Avail::WeekendOnly, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Due today with estimate ──

    #[test]
    fn due_today_with_day2_estimate() {
        // Due today → not overdue. Attention has passed → Today.
        assert_eq!(
            group("2026-05-09", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn due_today_with_day1_weekend_only() {
        // Due Sat May 9 (today). Not overdue.
        // Day1 WeekendOnly: attention = previous Sun May 3.
        // group_date <= today → Today.
        assert_eq!(
            group("2026-05-09", Some(Est::Day1), Avail::WeekendOnly, "2026-05-09"),
            DeadlineGroup::Today,
        );
    }
}

// ── End-to-end tests for group_upcoming ─────────────────────────
//
// Each scenario creates a minimal TaskCache and asserts the full
// output of group_upcoming: group order, task order within groups,
// attention labels, and backlog separation.

#[cfg(all(test, feature = "ssr"))]
mod upcoming_tests {
    use super::*;
    use chrono::NaiveDate;
    use kid_types::{
        TaskAvailability as Avail,
        TaskTimeEstimate as Est,
        task::{Date, Summary, Task},
    };
    use kid_types::server::TaskCache;

    fn date(today: &str) -> NaiveDate {
        NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap()
    }

    fn task_date(s: &str) -> Date {
        let nd = date(s);
        Date {
            date: nd.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset(),
            soft: false,
        }
    }

    fn add(
        cache: &mut TaskCache,
        name: &str,
        due: Option<&str>,
        start: Option<&str>,
        estimate: Option<Est>,
        avail: Avail,
    ) -> Uuid {
        use kid_types::{TaskDetails, TaskInfos};
        let mut t = Task::new(name.parse::<Summary>().unwrap());
        if let Some(d) = due { t.set_due_date(task_date(d)); }
        if let Some(s) = start { t.set_start_date(task_date(s)); }
        if let Some(e) = estimate { t.set_time_estimate(e); }
        t.set_availability(avail);
        cache.add(t, "test")
    }

    /// Flatten groups into `(group, summary, attention_label)` for easy asserts.
    fn flatten(
        groups: &[(DeadlineGroup, Vec<(Uuid, kid_types::task::Infos, Option<NaiveDate>)>)],
    ) -> Vec<(DeadlineGroup, &str, Option<NaiveDate>)> {
        groups.iter().flat_map(|(dg, tasks)| {
            tasks.iter().map(move |(_, info, attn)| (*dg, info.summary(), *attn))
        }).collect()
    }

    /// Just the group labels in order.
    fn group_names(
        groups: &[(DeadlineGroup, Vec<(Uuid, kid_types::task::Infos, Option<NaiveDate>)>)],
    ) -> Vec<DeadlineGroup> {
        groups.iter().map(|(dg, _)| *dg).collect()
    }

    /// Just the summaries in order.
    fn summaries(
        groups: &[(DeadlineGroup, Vec<(Uuid, kid_types::task::Infos, Option<NaiveDate>)>)],
    ) -> Vec<&str> {
        groups.iter()
            .flat_map(|(_, tasks)| tasks.iter().map(|(_, info, _)| info.summary()))
            .collect()
    }

    // ── Scenario 1: baseline grouping by due_date ──

    #[test]
    fn baseline_groups_by_due_date() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // 3 tasks: due today, due this week, due later.
        let mut cache = TaskCache::default();
        add(&mut cache, "today-task",
            Some("2026-05-11"), None, None, Avail::Anytime);
        add(&mut cache, "this-week-task",
            Some("2026-05-15"), None, None, Avail::Anytime);
        add(&mut cache, "later-task",
            Some("2026-05-30"), None, None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(backlog.is_empty());
        assert_eq!(
            group_names(&groups),
            vec![DeadlineGroup::Today, DeadlineGroup::ThisWeek, DeadlineGroup::Later],
        );
        assert_eq!(summaries(&groups).len(), 3);
    }

    // ── Scenario 2: estimate shifts task to earlier group ──

    #[test]
    fn estimate_pulls_task_forward() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Task due Tue May 19, Day2 Anytime:
        //   attention = May 17 (Sun) → This Week
        // Without estimate it would be Next Week.
        let mut cache = TaskCache::default();
        add(&mut cache, "shifted",
            Some("2026-05-19"), None, Some(Est::Day2), Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Attention label should be set (attention != due)
        assert_eq!(flat[0].2, Some(date("2026-05-17")));
    }

    // ── Scenario 3: sub-day estimate → no shift, no label ──

    #[test]
    fn sub_day_estimate_no_shift() {
        // Today: Mon May 11. Due Tue May 19, HalfDay.
        // lead_days=0 → group_date = due_date → Next Week, no label.
        let mut cache = TaskCache::default();
        add(&mut cache, "quick",
            Some("2026-05-19"), None, Some(Est::HalfDay), Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::NextWeek);
        assert_eq!(flat[0].2, None); // no attention label
    }

    // ── Scenario 4: weekend availability skips weekdays ──

    #[test]
    fn weekend_only_skips_weekdays() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Fri May 22, Day2 WeekendOnly:
        //   count 2 weekend days back: Sun 17→1, Sat 16→0
        //   attention = Sat May 16 → This Week
        let mut cache = TaskCache::default();
        add(&mut cache, "weekend-task",
            Some("2026-05-22"), None, Some(Est::Day2), Avail::WeekendOnly);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        assert_eq!(flat[0].2, Some(date("2026-05-16")));
    }

    // ── Scenario 5: backlog — no dates at all ──

    #[test]
    fn task_without_dates_goes_to_backlog() {
        let mut cache = TaskCache::default();
        add(&mut cache, "backlog-task",
            None, None, None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].1.summary(), "backlog-task");
    }

    // ── Scenario 6: start_date without due_date → Ready to Start ──

    #[test]
    fn start_date_only_ready_to_start() {
        // start_date = today, no due_date → ReadyToStart
        let mut cache = TaskCache::default();
        add(&mut cache, "started",
            None, Some("2026-05-11"), None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(backlog.is_empty());
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ReadyToStart);
    }

    // ── Scenario 7: start_date in future, no due_date → backlog ──

    #[test]
    fn future_start_date_no_due_is_backlog() {
        let mut cache = TaskCache::default();
        add(&mut cache, "not-yet",
            None, Some("2026-05-20"), None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert_eq!(backlog.len(), 1);
    }

    // ── Scenario 8: start_date pulls task into earlier group (soft) ──

    #[test]
    fn start_date_earlier_than_attention_pulls_forward_soft() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Task: due May 30 (Later), no estimate → attention = May 30.
        // start_date = May 14 (This Week).
        // start_date < attention → group by start_date → This Week, soft.
        let mut cache = TaskCache::default();
        add(&mut cache, "early-start",
            Some("2026-05-30"), Some("2026-05-14"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
    }

    // ── Scenario 9: soft tasks sort after hard tasks in same group ──

    #[test]
    fn soft_tasks_sort_after_hard_in_same_group() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        //
        // Hard task: due Tue May 19, Day2 Anytime →
        //   attention = May 17 → This Week (hard, attention-driven)
        // Soft task: due May 30, start_date May 14 →
        //   attention = May 30, but start_date pulls to This Week (soft)
        let mut cache = TaskCache::default();
        add(&mut cache, "hard-attention",
            Some("2026-05-19"), None, Some(Est::Day2), Avail::Anytime);
        add(&mut cache, "soft-start",
            Some("2026-05-30"), Some("2026-05-14"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let names = summaries(&groups);
        // Hard before soft within the same group.
        assert_eq!(names, vec!["hard-attention", "soft-start"]);
        // Both in This Week.
        assert_eq!(group_names(&groups), vec![DeadlineGroup::ThisWeek]);
    }

    // ── Scenario 10: start_date doesn't shift when same group ──

    #[test]
    fn start_date_same_group_no_shift() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due May 15 (This Week), start May 13 (also This Week).
        // start < due but same group → no soft shift.
        let mut cache = TaskCache::default();
        add(&mut cache, "same-group",
            Some("2026-05-15"), Some("2026-05-13"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
    }

    // ── Scenario 11: overdue ignores start_date ──

    #[test]
    fn overdue_ignores_start_date() {
        // Due yesterday. start_date doesn't rescue from Overdue.
        let mut cache = TaskCache::default();
        add(&mut cache, "overdue",
            Some("2026-05-10"), Some("2026-05-01"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Overdue);
    }

    // ── Scenario 12: done tasks excluded ──

    #[test]
    fn done_tasks_excluded() {
        let mut cache = TaskCache::default();
        let id = add(&mut cache, "done-task",
            Some("2026-05-15"), None, None, Avail::Anytime);
        {
            let mut guard = cache.get_mut(&id, "test").unwrap();
            guard.mark_done();
        }

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert!(backlog.is_empty());
    }

    // ── Scenario 13: mixed — overdue, today, shifted, backlog ──

    #[test]
    fn mixed_scenario() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        let mut cache = TaskCache::default();
        add(&mut cache, "overdue",
            Some("2026-05-10"), None, None, Avail::Anytime);
        add(&mut cache, "today",
            Some("2026-05-11"), None, None, Avail::Anytime);
        add(&mut cache, "shifted-to-this-week",
            Some("2026-05-19"), None, Some(Est::Day2), Avail::Anytime);
        add(&mut cache, "backlog",
            None, None, None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(
            group_names(&groups),
            vec![DeadlineGroup::Overdue, DeadlineGroup::Today, DeadlineGroup::ThisWeek],
        );
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].1.summary(), "backlog");
    }

    // ── Scenario 14: start_date in past pulls to Today (soft) ──

    #[test]
    fn past_start_date_pulls_to_today_soft() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due May 30 (Later), start May 5 (past).
        // start < attention (May 30) → start group = Today (soft).
        let mut cache = TaskCache::default();
        add(&mut cache, "started-last-week",
            Some("2026-05-30"), Some("2026-05-05"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Today);
    }

    // ── Scenario 15: soft Today sorts after hard Today ──

    #[test]
    fn soft_today_after_hard_today() {
        // Today: Mon May 11
        // Hard: due May 11 (Today, naturally)
        // Soft: due May 30, start May 05 (Today via start_date)
        let mut cache = TaskCache::default();
        add(&mut cache, "hard-today",
            Some("2026-05-11"), None, None, Avail::Anytime);
        add(&mut cache, "soft-today",
            Some("2026-05-30"), Some("2026-05-05"), None, Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let names = summaries(&groups);
        assert_eq!(names, vec!["hard-today", "soft-today"]);
    }

    // ── Scenario 16: start_date >= attention → no soft shift ──

    #[test]
    fn start_date_after_attention_no_shift() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 19, Day2 → attention May 17 (This Week).
        // start May 18 (after attention) → no soft shift, stays This Week.
        let mut cache = TaskCache::default();
        add(&mut cache, "late-start",
            Some("2026-05-19"), Some("2026-05-18"), Some(Est::Day2), Avail::Anytime);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Attention label should still be set (attention != due)
        assert_eq!(flat[0].2, Some(date("2026-05-17")));
    }

    // ── Scenario 17: WeekdayOnly shifts group + label ──

    #[test]
    fn weekday_only_skips_weekend() {
        // Today: Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day2 WeekdayOnly:
        //   Sun 17→skip, Sat 16→skip, Fri 15→1, Thu 14→0
        //   attention = Thu May 14 → This Week (14 <= 17)
        // Without estimate it would be Next Week (18 > 17).
        let mut cache = TaskCache::default();
        add(&mut cache, "weekday-task",
            Some("2026-05-18"), None, Some(Est::Day2), Avail::WeekdayOnly);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-13"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        assert_eq!(flat[0].2, Some(date("2026-05-14")));
    }

    // ── Scenario 18: WeekdayOnly Day1 ──

    #[test]
    fn weekday_only_day1_skips_weekend() {
        // Today: Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day1 WeekdayOnly:
        //   Sun 17→skip, Sat 16→skip, Fri 15→0
        //   attention = Fri May 15 → This Week
        let mut cache = TaskCache::default();
        add(&mut cache, "weekday-1d",
            Some("2026-05-18"), None, Some(Est::Day1), Avail::WeekdayOnly);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-13"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        assert_eq!(flat[0].2, Some(date("2026-05-15")));
    }

    // ── Scenario 19: availability + start_date interaction ──

    #[test]
    fn weekend_only_with_earlier_start_date() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Fri May 22, Day2 WeekendOnly:
        //   attention = Sat May 16 → This Week
        // start_date = May 12 (Mon) < attention (May 16)
        //   start group = This Week (12 <= 17)
        //   Same group → no soft shift, stays hard This Week.
        let mut cache = TaskCache::default();
        add(&mut cache, "weekend-with-start",
            Some("2026-05-22"), Some("2026-05-12"), Some(Est::Day2), Avail::WeekendOnly);

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Attention label from estimate (not from start_date)
        assert_eq!(flat[0].2, Some(date("2026-05-16")));
    }
}
