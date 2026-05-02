pub mod cache;
mod error_template;
pub mod server;

use crate::error_template::ErrorTemplate;

use kid_types::{TaskCategory, TaskContext, TaskDate, TaskDetails, TaskId, TaskInfos, TaskPriority, TaskSummary, TaskTimeEstimate, Uuid};
use kid_types::task;
use strum::IntoEnumIterator;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::TimeDelta;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
    hooks::use_query_map,
};
use strum::{EnumCount, FromRepr};

use std::ops::Deref;
use std::str::FromStr;
use std::fmt::{self, Display, Formatter};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="favicon.ico"/>
                <link rel="icon" type="image/png" href="favicon-16x16.png"/>
                <link rel="icon" type="image/png" href="favicon-32x32.png"/>
                <link rel="icon" type="image/png" href="favicon-64x46.png"/>
                <link rel="icon" type="image/png" href="favicon-128x128.png"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kid.css"/>

        // sets the document title
        <Title text="Tasks"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=TaskList/>
                </Routes>
            </main>
        </Router>
    }
}

mod key {
    pub const ENTER: &str = "Enter";
    pub const ESCAPE: &str = "Escape";
}

/*
 * Copy is intentional:
 * RwSignal is a lightweight handle (essentially an index into a reactive store),
 * so copying it is cheap and semantically correct.
 *
 * Leptos components pass EditMode into multiple independent move closures;
 * without Copy, each closure would need an explicit .clone() before capture:
 * ```rust
 * // without Copy
 * let em1 = edit_mode.clone();
 * let em2 = edit_mode.clone();
 * view! { <button class=move || em1.get() on:click=move |_| em2.update(…)> }
 * // with Copy 
 * view! { <button class=move || edit_mode.get() on:click=move |_| edit_mode.update(…)> }
 * ```
 */
#[derive(Clone, Copy, Default)]
struct EditMode(RwSignal<bool>);

#[derive(Clone, Copy)]
struct AvailableCategoriesCtx(RwSignal<Vec<String>>);

#[derive(Clone, Copy)]
struct AvailableContextsCtx(RwSignal<Vec<String>>);

#[derive(Clone, Copy)]
struct ScrollToTaskId(RwSignal<Option<Uuid>>);

impl Deref for EditMode {
    type Target = RwSignal<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for AvailableCategoriesCtx {
    type Target = RwSignal<Vec<String>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Deref for AvailableContextsCtx {
    type Target = RwSignal<Vec<String>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Deref for ScrollToTaskId {
    type Target = RwSignal<Option<Uuid>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

/// SSR-evaluated auto-expand: when `?expand=first` is set, a [`Memo`]
/// resolves to the first task ID once the resource has loaded.
#[derive(Clone, Copy)]
struct AutoExpandFirst(Memo<Option<Uuid>>);

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumCount, FromRepr)]
enum View {
    Upcoming,
    QuickWins,
    AllOpen,
    WhatIFinished,
    RecentlyChanged,
}

impl View {
    fn title(self) -> &'static str {
        match self {
            View::Upcoming        => "Upcoming",
            View::AllOpen         => "All Open",
            View::WhatIFinished   => "What I Finished",
            View::QuickWins       => "Quick Wins",
            View::RecentlyChanged => "Recent Changes",
        }
    }

    /// Mono-hue gradients — one color family per view for instant
    /// ADHD-friendly recognition ("the teal one", "the green one", …).
    fn header_gradient(self) -> &'static str {
        match self {
            View::Upcoming        => "from-cyan-500 to-cyan-700",
            View::QuickWins       => "from-emerald-500 to-emerald-700",
            View::AllOpen         => "from-violet-500 to-violet-700",
            View::WhatIFinished   => "from-amber-500 to-amber-700",
            View::RecentlyChanged => "from-teal-500 to-teal-700",
        }
    }

    fn dot_active_color(self) -> &'static str {
        match self {
            View::Upcoming        => "bg-cyan-200",
            View::QuickWins       => "bg-emerald-200",
            View::AllOpen         => "bg-violet-200",
            View::WhatIFinished   => "bg-amber-200",
            View::RecentlyChanged => "bg-teal-200",
        }
    }

    fn prev(self) -> Option<Self> {
        (self as usize).checked_sub(1).and_then(Self::from_repr)
    }

    fn next(self) -> Option<Self> {
        Self::from_repr(self as usize + 1)
    }

    fn subtitle(self) -> &'static str {
        match self {
            View::Upcoming        => "Open tasks with dates · grouped by urgency",
            View::AllOpen         => "Open tasks · ↓ category · oldest first",
            View::WhatIFinished   => "Completed tasks · ↓ category · recent first",
            View::QuickWins       => "Open tasks with estimate · grouped by duration",
            View::RecentlyChanged => "Today + 2 days · grouped by day",
        }
    }

    fn empty_message(self) -> &'static str {
        match self {
            View::Upcoming        => "Nothing on the horizon.",
            View::AllOpen         => "No open tasks.",
            View::WhatIFinished   => "Nothing finished yet.",
            View::QuickWins       => "No estimated tasks.",
            View::RecentlyChanged => "No recent changes.",
        }
    }

    fn checkbox_checked_classes(self) -> &'static str {
        match self {
            View::Upcoming => {
                "checked:from-cyan-400 checked:to-cyan-600 checked:border-cyan-400"
            }
            View::QuickWins => {
                "checked:from-emerald-400 checked:to-emerald-600 checked:border-emerald-400"
            }
            View::AllOpen => {
                "checked:from-violet-400 checked:to-violet-600 checked:border-violet-400"
            }
            View::WhatIFinished => {
                "checked:from-amber-400 checked:to-amber-600 checked:border-amber-400"
            }
            View::RecentlyChanged => {
                "checked:from-teal-400 checked:to-teal-600 checked:border-teal-400"
            }
        }
    }

    fn spinner_gradient(self) -> &'static str {
        match self {
            View::Upcoming        => "from-cyan-400 to-cyan-600",
            View::QuickWins       => "from-emerald-400 to-emerald-600",
            View::AllOpen         => "from-violet-400 to-violet-600",
            View::WhatIFinished   => "from-amber-400 to-amber-600",
            View::RecentlyChanged => "from-teal-400 to-teal-600",
        }
    }

    /// Parse a `?view=` query-parameter value into a [`View`].
    ///
    /// Used by SSR to render a specific view on initial page load,
    /// enabling screenshot automation without WASM hydration.
    fn from_query_param(s: &str) -> Option<Self> {
        match s {
            "upcoming"  => Some(View::Upcoming),
            "quickwins" => Some(View::QuickWins),
            "allopen"   => Some(View::AllOpen),
            "finished"  => Some(View::WhatIFinished),
            "recent"    => Some(View::RecentlyChanged),
            _           => None,
        }
    }

    fn priority_a_border(self) -> &'static str {
        match self {
            View::Upcoming        => "border-l-[3px] border-l-cyan-500",
            View::QuickWins       => "border-l-[3px] border-l-emerald-500",
            View::AllOpen         => "border-l-[3px] border-l-violet-500",
            View::WhatIFinished   => "border-l-[3px] border-l-amber-500",
            View::RecentlyChanged => "",  // left border reserved for AI-involvement
        }
    }
}

fn arrow_opacity_class(switch_count: u32) -> &'static str {
    match switch_count {
        0..=10 => "opacity-80",
        11..=50 => "opacity-40 hover:opacity-80 focus-visible:opacity-80",
        _ => "opacity-20 hover:opacity-80 focus-visible:opacity-80",
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum TaskListData {
    Grouped(IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>>),
    EstimateGrouped(Vec<(TaskTimeEstimate, Vec<(Uuid, task::Infos)>)>),
    DeadlineGrouped(Vec<(DeadlineGroup, Vec<(Uuid, task::Infos)>)>, usize),
    DayGrouped(Vec<(NaiveDate, Vec<server::RecentChange>)>),
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadlineGroup {
    Overdue,
    Today,
    ThisWeek,
    NextWeek,
    Later,
    ReadyToStart,
}

impl DeadlineGroup {
    fn label(self) -> &'static str {
        match self {
            Self::Overdue => "Overdue",
            Self::Today => "Today",
            Self::ThisWeek => "This Week",
            Self::NextWeek => "Next Week",
            Self::Later => "Later",
            Self::ReadyToStart => "Ready to Start",
        }
    }

    fn is_overdue(self) -> bool {
        matches!(self, Self::Overdue)
    }
}

#[derive(Copy, Clone)]
struct GroupCollapseState {
    owner: StoredValue<Owner>,
    map: StoredValue<HashMap<TaskCategory, RwSignal<bool>>>,
}

impl GroupCollapseState {
    fn new() -> Self {
        Self {
            owner: StoredValue::new(Owner::current().expect("must be in reactive context")),
            map: StoredValue::new(HashMap::new()),
        }
    }

    fn ensure(&self, category: &TaskCategory) {
        let category = category.clone();
        let owner = self.owner;
        self.map.update_value(|map| {
            map.entry(category).or_insert_with(|| {
                owner.with_value(|o| o.with(|| RwSignal::new(false)))
            });
        });
    }

    fn signal_for(&self, category: &TaskCategory) -> RwSignal<bool> {
        self.map.with_value(|map| map[category])
    }
}

fn group_recent_by_day(tasks: Vec<server::RecentChange>) -> Vec<(NaiveDate, Vec<server::RecentChange>)> {
    let mut groups: IndexMap<NaiveDate, Vec<server::RecentChange>> = IndexMap::new();
    for rc in tasks {
        groups.entry(rc.last_changed.date_naive()).or_default().push(rc);
    }
    groups.into_iter().collect()
}

fn day_label(day: NaiveDate) -> String {
    let today = Utc::now().date_naive();
    if day == today {
        "Today".to_string()
    } else if day == today.checked_sub_days(chrono::Days::new(1)).unwrap() {
        "Yesterday".to_string()
    } else {
        day.format("%A, %d.%m.").to_string()
    }
}

impl TaskListData {
    /// Return the ID of the first task across all groups.
    fn first_task_id(&self) -> Option<Uuid> {
        match self {
            TaskListData::Grouped(m) => {
                m.values().flatten().next().map(|(id, _)| *id)
            }
            TaskListData::EstimateGrouped(v) => {
                v.iter().flat_map(|(_, t)| t).next().map(|(id, _)| *id)
            }
            TaskListData::DeadlineGrouped(v, _) => {
                v.iter().flat_map(|(_, t)| t).next().map(|(id, _)| *id)
            }
            TaskListData::DayGrouped(v) => {
                v.iter().flat_map(|(_, t)| t).next().map(|rc| rc.id)
            }
        }
    }
}

fn apply_filter(data: TaskListData, filters: &[String]) -> TaskListData {
    if filters.is_empty() { return data; }
    let matches = |info: &task::Infos| -> bool {
        filters.iter().all(|f| info.contexts().iter().any(|c| c.to_string() == *f))
    };
    match data {
        TaskListData::EstimateGrouped(groups) => TaskListData::EstimateGrouped(
            groups.into_iter().filter_map(|(est, tasks)| {
                let filtered: Vec<_> = tasks.into_iter().filter(|(_, info)| matches(info)).collect();
                if filtered.is_empty() { None } else { Some((est, filtered)) }
            }).collect()
        ),
        // Backlog count is unaffected by context filters (UXDR).
        TaskListData::DeadlineGrouped(groups, backlog) => TaskListData::DeadlineGrouped(
            groups.into_iter().filter_map(|(dg, tasks)| {
                let filtered: Vec<_> = tasks.into_iter().filter(|(_, info)| matches(info)).collect();
                if filtered.is_empty() { None } else { Some((dg, filtered)) }
            }).collect(),
            backlog,
        ),
        TaskListData::DayGrouped(groups) => TaskListData::DayGrouped(
            groups.into_iter().filter_map(|(day, tasks)| {
                let filtered: Vec<_> = tasks.into_iter().filter(|rc| matches(&rc.info)).collect();
                if filtered.is_empty() { None } else { Some((day, filtered)) }
            }).collect()
        ),
        TaskListData::Grouped(groups) => TaskListData::Grouped(
            groups.into_iter().filter_map(|(cat, tasks)| {
                let filtered: Vec<_> = tasks.into_iter().filter(|(_, info)| matches(info)).collect();
                if filtered.is_empty() { None } else { Some((cat, filtered)) }
            }).collect()
        ),
    }
}

#[component]
fn TaskList() -> impl IntoView {
    let (expanded_task_id, set_expanded_task_id) = signal(None::<Uuid>);
    let scroll_to_task_id = ScrollToTaskId(RwSignal::new(None));
    provide_context(scroll_to_task_id);
    let group_collapse = GroupCollapseState::new();

    let add_task = Action::new(move |summary: &String| {
        let summary = summary.clone();
        async move {
            match summary.parse::<TaskSummary>() {
                Ok(s) => match server::add_task(s).await {
                    Ok(id) => {
                        set_expanded_task_id.set(Some(id));
                        scroll_to_task_id.set(Some(id));
                    }
                    Err(e) => tracing::error!("add task failed: {e}"),
                },
                Err(e) => tracing::error!("invalid summary: {e}"),
            }
        }
    });
    let delete_task = ServerAction::<server::DeleteTask>::new();
    let (completion_version, set_completion_version) = signal(0u32);
    let category_version = RwSignal::new(0u32);
    provide_context(category_version);
    let available_categories_res = Resource::new(move || category_version.get(), |_| server::fetch_categories());
    let available_categories_ctx: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    Effect::new(move |_| {
        if let Some(Ok(fetched)) = available_categories_res.get() {
            available_categories_ctx.update(|v| {
                for cat in fetched {
                    let s = cat.to_string();
                    if !v.contains(&s) { v.push(s); }
                }
            });
        }
    });
    provide_context(AvailableCategoriesCtx(available_categories_ctx));

    let params = use_query_map().get_untracked();
    let current_view = RwSignal::new(
        params.get("view")
            .and_then(|v| View::from_query_param(&v))
            .unwrap_or(View::Upcoming)
    );
    let expand_first = params.get("expand").is_some_and(|v| v == "first");
    let switch_count = RwSignal::new(0u32);
    let edit_mode = EditMode::default();
    provide_context(edit_mode);

    let filter_open = RwSignal::new(false);
    let active_filters: RwSignal<HashMap<View, Vec<String>>> = RwSignal::new(HashMap::new());
    let filter_ctx_resource = Resource::new(|| (), |_| server::fetch_contexts());
    let available_contexts_ctx: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    Effect::new(move |_| {
        if let Some(Ok(fetched)) = filter_ctx_resource.get() {
            available_contexts_ctx.update(|v| {
                for ctx in fetched {
                    let s = ctx.to_string();
                    if !v.contains(&s) { v.push(s); }
                }
            });
        }
    });
    provide_context(AvailableContextsCtx(available_contexts_ctx));

    window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == key::ESCAPE && !ev.default_prevented() {
            if edit_mode.get() { edit_mode.update(|m| *m = false); }
            else if filter_open.get() { filter_open.set(false); }
        }
    });

    // Midnight rollover: check every 60 s whether the UTC date changed.
    // Bumping this signal triggers a resource refetch + fresh day labels.
    // Guard: set_interval is wasm-bindgen and panics on native (SSR).
    let today_signal = RwSignal::new(Utc::now().date_naive());
    if cfg!(target_arch = "wasm32") {
        set_interval(move || {
            let now = Utc::now().date_naive();
            if now != today_signal.get_untracked() {
                today_signal.set(now);
            }
        }, std::time::Duration::from_secs(60));
    }

    let extra_days = RwSignal::new(0u32);

    let task_list = Resource::new(
        move || (add_task.version().get(), delete_task.version().get(), completion_version.get(), category_version.get(), current_view.get(), today_signal.get(), extra_days.get()),
        move |_| async move {
            match current_view.get_untracked() {
                View::Upcoming       => {
                    server::fetch_upcoming(today_signal.get_untracked())
                        .await.map(|(groups, backlog)| TaskListData::DeadlineGrouped(groups, backlog))
                },
                View::AllOpen        => server::fetch_all_open().await.map(TaskListData::Grouped),
                View::WhatIFinished  => server::fetch_what_i_finished().await.map(TaskListData::Grouped),
                View::QuickWins      => server::fetch_quick_wins().await.map(TaskListData::EstimateGrouped),
                View::RecentlyChanged => {
                    server::fetch_recently_changed(extra_days.get_untracked()).await.map(|v| TaskListData::DayGrouped(group_recent_by_day(v)))
                },
            }
        },
    );

    // ?expand=first — pre-expand the first task (used by screenshot tests).
    // Provided as context so TaskItem can check it during the SSR pass
    // (Effect doesn't run during SSR).
    if expand_first {
        let auto_id = Memo::new(move |_| {
            task_list.get()
                .and_then(|r| r.ok())
                .and_then(|d| d.first_task_id())
        });
        provide_context(AutoExpandFirst(auto_id));
    }

    let go_prev = move |_| {
        if let Some(prev) = current_view.get_untracked().prev() {
            current_view.set(prev);
            switch_count.update(|c| *c += 1);
        }
    };
    let go_next = move |_| {
        if let Some(next) = current_view.get_untracked().next() {
            current_view.set(next);
            switch_count.update(|c| *c += 1);
        }
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-950 to-slate-900">
            <div class="max-w-2xl mx-auto min-h-screen bg-slate-900 shadow-2xl">
                <header class=move || format!(
                    "px-6 pt-4 pb-5 bg-gradient-to-b {} text-white select-none",
                    current_view.get().header_gradient()
                )>
                    <div class="relative text-center">
                        // Left arrow
                        <button
                            type="button"
                            class=move || {
                                let opacity = match current_view.get().prev() {
                                    Some(_) => arrow_opacity_class(switch_count.get()),
                                    None => "opacity-0 pointer-events-none",
                                };
                                format!("absolute -left-2 top-[58%] -translate-y-1/2 w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
                            }
                            on:click=go_prev
                            aria-label=move || current_view.get().prev()
                                .map(|v| format!("Previous view: {}", v.title()))
                                .unwrap_or_default()
                        >
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                                <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd"/>
                            </svg>
                        </button>
                        // Title
                        <h1 aria-live="polite" class="text-3xl font-semibold">
                            {move || current_view.get().title()}
                        </h1>
                        <p class="text-xs opacity-60 mt-0.5">
                            {move || current_view.get().subtitle()}
                        </p>
                        // Right arrow
                        <button
                            type="button"
                            class=move || {
                                let opacity = match current_view.get().next() {
                                    Some(_) => arrow_opacity_class(switch_count.get()),
                                    None => "opacity-0 pointer-events-none",
                                };
                                format!("absolute -right-2 top-[58%] -translate-y-1/2 w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
                            }
                            on:click=go_next
                            aria-label=move || current_view.get().next()
                                .map(|v| format!("Next view: {}", v.title()))
                                .unwrap_or_default()
                        >
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                                <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                            </svg>
                        </button>
                    </div>
                    // Page indicator dots + toolbar
                    <div class="relative flex justify-center items-center gap-2 mt-3">
                        {(0..View::COUNT).map(|i| {
                            let v = View::from_repr(i).unwrap();
                            view! {
                                <button
                                    type="button"
                                    class="p-3 -m-3"
                                    on:click=move |_| {
                                        if current_view.get_untracked() != v {
                                            current_view.set(v);
                                            switch_count.update(|c| *c += 1);
                                        }
                                    }
                                    aria-label={v.title()}
                                >
                                    <div class=move || if current_view.get() == v {
                                        format!("h-1.5 w-4 rounded-full transition-all {}", v.dot_active_color())
                                    } else {
                                        "h-1.5 w-1.5 rounded-full bg-white opacity-40 hover:opacity-60 transition-all".to_string()
                                    }/>
                                </button>
                            }
                        }).collect_view()}
                        // Filter button (left)
                        <button
                            type="button"
                            class=move || if filter_open.get() || active_filters.with(|m| !m.get(&current_view.get()).map(Vec::is_empty).unwrap_or(true)) {
                                "absolute -left-2 w-8 h-8 flex items-center justify-center text-teal-300 hover:text-white transition-colors"
                            } else {
                                "absolute -left-2 w-8 h-8 flex items-center justify-center text-white opacity-60 hover:opacity-100 transition-colors"
                            }
                            on:click=move |_| filter_open.update(|o| *o = !*o)
                            aria-label="Toggle filter"
                        >
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-.293.707L13 10.414V17a1 1 0 01-.553.894l-4 2A1 1 0 017 19v-8.586L3.293 6.707A1 1 0 013 6V4z" clip-rule="evenodd"/>
                            </svg>
                        </button>
                        // Edit button (right)
                        <button
                            type="button"
                            class=move || if edit_mode.get() {
                                "absolute -right-2 w-8 h-8 flex items-center justify-center text-amber-300 hover:text-white transition-colors"
                            } else {
                                "absolute -right-2 w-8 h-8 flex items-center justify-center text-white opacity-60 hover:opacity-100 transition-colors"
                            }
                            on:click=move |_| edit_mode.update(|m| *m = !*m)
                            aria-label="Toggle edit mode"
                        >
                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"/>
                            </svg>
                        </button>
                    </div>
                </header>
                // Filter panel
                {move || filter_open.get().then(|| {
                    let view = current_view.get();
                    view! {
                        <div class="px-4 py-3 bg-slate-800 border-b border-slate-700">
                            <div class="flex flex-wrap gap-1.5">
                                {move || available_contexts_ctx.get().into_iter().map(|ctx| {
                                    let label_class = ctx.clone();
                                    let label_click = ctx.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class=move || if active_filters.with(|m| m.get(&view).map(|v| v.contains(&label_class)).unwrap_or(false)) {
                                                "px-2.5 py-0.5 rounded-full text-xs font-medium bg-teal-600 text-white transition-colors"
                                            } else {
                                                "px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors"
                                            }
                                            on:click=move |_| {
                                                active_filters.update(|m| {
                                                    let list = m.entry(view).or_default();
                                                    if let Some(pos) = list.iter().position(|c| c == &label_click) {
                                                        list.remove(pos);
                                                    } else {
                                                        list.push(label_click.clone());
                                                    }
                                                });
                                            }
                                        >
                                            {ctx}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                })}
                <div class="py-2">
                    <Transition fallback=move || view! {
                        <div class="px-6 py-6 text-center text-slate-400">"Loading tasks..."</div>
                    }>
                        <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors/> }>
                            {move || {
                                let view = current_view.get();
                                let filters = active_filters.with(|m| m.get(&view).cloned().unwrap_or_default());
                                Suspend::new(async move {
                                    task_list.await.map(|data| {
                                        let data = apply_filter(data, &filters);
                                        let backlog_count = match &data {
                                            TaskListData::DeadlineGrouped(_, count) => *count,
                                            _ => 0,
                                        };
                                        let is_empty = match &data {
                                            TaskListData::Grouped(m) => m.is_empty(),
                                            TaskListData::EstimateGrouped(v) => v.is_empty(),
                                            TaskListData::DeadlineGrouped(v, _) => v.is_empty(),
                                            TaskListData::DayGrouped(v) => v.iter().all(|(_, t)| t.is_empty()),
                                        };
                                        if is_empty {
                                            Either::Left(view! {
                                                <div>
                                                    <p class="px-6 py-6 text-center text-slate-400">
                                                        {if view == View::Upcoming && backlog_count > 0 {
                                                            format!("{} {} tasks in backlog.", view.empty_message(), backlog_count)
                                                        } else {
                                                            view.empty_message().to_string()
                                                        }}
                                                    </p>
                                                    {if view == View::RecentlyChanged {
                                                        Either::Left(view! {
                                                            <button
                                                                type="button"
                                                                class="w-full py-3 text-sm text-slate-500 hover:text-slate-300 transition-colors"
                                                                on:click=move |_| extra_days.update(|n| *n += 2)
                                                            >
                                                                "▾ 2 more days"
                                                            </button>
                                                        })
                                                    } else {
                                                        Either::Right(())
                                                    }}
                                                </div>
                                            })
                                        } else {
                                            Either::Right(match data {
                                                TaskListData::Grouped(groups) => {
                                                    for cat in groups.keys() {
                                                        group_collapse.ensure(cat);
                                                    }
                                                    let groups: Vec<_> = groups.into_iter().collect();
                                                    Either::Left(view! {
                                                        <div>
                                                            {groups.into_iter().enumerate().map(|(i, (cat, tasks))| {
                                                                let collapsed = group_collapse.signal_for(&cat);
                                                                let tasks = StoredValue::new(tasks);
                                                                view! {
                                                                    <div class=if i == 0 { "" } else { "border-t border-slate-600 mt-1" }>
                                                                        <button
                                                                            type="button"
                                                                            class="w-full flex items-center gap-2 px-6 pt-3 pb-1 text-left select-none"
                                                                            on:click=move |_| collapsed.update(|c| *c = !*c)
                                                                        >
                                                                            <span class="text-sm font-semibold text-slate-400">
                                                                                {cat.to_string()}
                                                                            </span>
                                                                            <span class=move || format!(
                                                                                "text-slate-500 text-xs transition-transform {}",
                                                                                if collapsed.get() { "" } else { "rotate-90" }
                                                                            )>"›"</span>
                                                                        </button>
                                                                        <Show when=move || !collapsed.get()>
                                                                            <div>
                                                                                <For
                                                                                    each=move || tasks.get_value()
                                                                                    key=|task| *task.id()
                                                                                    children=move |task| {
                                                                                        view! {
                                                                                            <TaskItem task=task
                                                                                                expanded_task_id=expanded_task_id
                                                                                                set_expanded_task_id=set_expanded_task_id
                                                                                                set_completion_version=set_completion_version
                                                                                                strikethrough_when_done={view == View::AllOpen}
                                                                                                checkbox_checked_classes={view.checkbox_checked_classes()}
                                                                                                spinner_gradient={view.spinner_gradient()}
                                                                                                priority_a_border={view.priority_a_border()}
                                                                                            />
                                                                                        }
                                                                                    }
                                                                                />
                                                                            </div>
                                                                        </Show>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    })
                                                }
                                                TaskListData::EstimateGrouped(groups) => {
                                                    Either::Right(Either::Left(view! {
                                                        <div>
                                                            {groups.into_iter().enumerate().map(|(i, (est, tasks))| {
                                                                let label = est.to_string();
                                                                view! {
                                                                    <div class=if i == 0 { "" } else { "border-t border-slate-600 mt-1" }>
                                                                        <div class="px-6 pt-3 pb-1">
                                                                            <span class="text-sm font-semibold text-slate-400">
                                                                                {label}
                                                                            </span>
                                                                        </div>
                                                                        {tasks.into_iter().map(|task| {
                                                                            view! {
                                                                                <TaskItem task=task
                                                                                    expanded_task_id=expanded_task_id
                                                                                    set_expanded_task_id=set_expanded_task_id
                                                                                    set_completion_version=set_completion_version
                                                                                    strikethrough_when_done=false
                                                                                    checkbox_checked_classes={view.checkbox_checked_classes()}
                                                                                    spinner_gradient={view.spinner_gradient()}
                                                                                    priority_a_border={view.priority_a_border()}
                                                                                />
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }))
                                                }
                                                TaskListData::DeadlineGrouped(groups, backlog) => {
                                                    Either::Right(Either::Right(Either::Left(view! {
                                                        <div>
                                                            {groups.into_iter().enumerate().map(|(i, (dg, tasks))| {
                                                                let label = dg.label();
                                                                let header_class = if dg.is_overdue() {
                                                                    "text-sm font-semibold text-cyan-300"
                                                                } else {
                                                                    "text-sm font-semibold text-slate-400"
                                                                };
                                                                view! {
                                                                    <div class=if i == 0 { "" } else { "border-t border-slate-600 mt-1" }>
                                                                        <div class="px-6 pt-3 pb-1">
                                                                            <span class=header_class>
                                                                                {label}
                                                                            </span>
                                                                        </div>
                                                                        {tasks.into_iter().map(|task| {
                                                                            view! {
                                                                                <TaskItem task=task
                                                                                    expanded_task_id=expanded_task_id
                                                                                    set_expanded_task_id=set_expanded_task_id
                                                                                    set_completion_version=set_completion_version
                                                                                    strikethrough_when_done=false
                                                                                    checkbox_checked_classes={view.checkbox_checked_classes()}
                                                                                    spinner_gradient={view.spinner_gradient()}
                                                                                    priority_a_border={view.priority_a_border()}
                                                                                />
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                            {(backlog > 0).then(|| view! {
                                                                <p class="text-xs text-slate-500 text-center py-3 border-t border-slate-700">
                                                                    {format!("── {} tasks without deadline in backlog ──", backlog)}
                                                                </p>
                                                            })}
                                                        </div>
                                                    })))
                                                }
                                                TaskListData::DayGrouped(groups) => {
                                                    Either::Right(Either::Right(Either::Right(view! {
                                                        <div>
                                                            {groups.into_iter().enumerate().map(|(i, (day, tasks))| {
                                                                let label = day_label(day);
                                                                view! {
                                                                    <div class=if i == 0 { "" } else { "border-t border-slate-600 mt-1" }>
                                                                        <div class="px-6 pt-3 pb-1">
                                                                            <span class="text-sm font-semibold text-slate-400">
                                                                                {label}
                                                                            </span>
                                                                        </div>
                                                                        {tasks.into_iter().map(|rc| {
                                                                            let ai_last = rc.ai_last;
                                                                            let ai_involved = rc.ai_involved;
                                                                            let task = (rc.id, rc.info);
                                                                            view! {
                                                                                <div class=if ai_last { "border-l-4 border-amber-500" } else if ai_involved { "border-l-4 border-violet-500" } else { "" }>
                                                                                    <TaskItem task=task
                                                                                        expanded_task_id=expanded_task_id
                                                                                        set_expanded_task_id=set_expanded_task_id
                                                                                        set_completion_version=set_completion_version
                                                                                        strikethrough_when_done=false
                                                                                        checkbox_checked_classes={view.checkbox_checked_classes()}
                                                                                        spinner_gradient={view.spinner_gradient()}
                                                                                        priority_a_border={view.priority_a_border()}
                                                                                    />
                                                                                </div>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                            <button
                                                                type="button"
                                                                class="w-full py-3 text-sm text-slate-500 hover:text-slate-300 transition-colors"
                                                                on:click=move |_| extra_days.update(|n| *n += 2)
                                                            >
                                                                "▾ 2 more days"
                                                            </button>
                                                        </div>
                                                    })))
                                                }
                                            })
                                        }
                                    })
                                })
                            }}
                        </ErrorBoundary>
                    </Transition>
                </div>
                <Show when=move || edit_mode.get()>
                    <AddTaskForm on_add=move |summary: String| { add_task.dispatch(summary); }/>
                </Show>
                <footer class="py-4 text-center text-xs text-slate-600 select-none" title="keep it done">
                    {concat!("kid ", env!("CARGO_PKG_VERSION"))}
                </footer>
            </div>
            <Show when=move || edit_mode.get()>
                <div class="fixed top-0 left-0 right-0 h-[3px] bg-amber-400 z-50"></div>
                <div class="fixed bottom-0 left-0 right-0 h-[3px] bg-amber-400 z-50"></div>
            </Show>
        </div>
    }
}

#[component]
fn AddTaskForm(#[prop(into)] on_add: UnsyncCallback<String>) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let value = RwSignal::new(String::new());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let submit = move || {
        let v = value.get_untracked().trim().to_string();
        if !v.is_empty() {
            on_add.run(v);
            value.set(String::new());
        }
        expanded.set(false);
    };

    Effect::new(move || {
        if expanded.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    view! {
        <div class="border-t border-slate-700">
            {move || if expanded.get() {
                Either::Left(view! {
                    <input
                        node_ref=input_ref
                        type="text"
                        class="w-full bg-slate-700 text-slate-100 rounded-lg px-6 py-4 border border-amber-500 focus:outline-none placeholder-slate-400 text-base"
                        placeholder="New task…"
                        prop:value=move || value.get()
                        on:input=move |ev| value.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == key::ENTER {
                                ev.prevent_default();
                                submit();
                            } else if ev.key() == key::ESCAPE {
                                ev.prevent_default();
                                value.set(String::new());
                                expanded.set(false);
                            }
                        }
                        on:blur=move |_| submit()
                    />
                })
            } else {
                Either::Right(view! {
                    <button
                        type="button"
                        class="w-full flex items-center justify-center text-slate-400 hover:text-amber-400 transition-colors py-3"
                        on:click=move |_| expanded.set(true)
                    >
                        <span class="text-base">"Add Task"</span>
                    </button>
                })
            }}
        </div>
    }
}

#[component]
fn TaskItem<T: for<'a> TaskId<'a> + for<'a> TaskInfos<'a>>(
    task: T,
    expanded_task_id: ReadSignal<Option<Uuid>>,
    set_expanded_task_id: WriteSignal<Option<Uuid>>,
    set_completion_version: WriteSignal<u32>,
    strikethrough_when_done: bool,
    checkbox_checked_classes: &'static str,
    spinner_gradient: &'static str,
    priority_a_border: &'static str,
) -> impl IntoView {
    let id = *task.id();
    let task_ref = NodeRef::<leptos::html::Div>::new();

    // Scroll to this task when it was just created
    if let Some(scroll_to) = use_context::<ScrollToTaskId>() {
        Effect::new(move || {
            if scroll_to.get() == Some(id) {
                set_timeout(move || {
                    if let Some(el) = task_ref.get() {
                        el.scroll_into_view();
                        scroll_to.set(None);
                    }
                }, std::time::Duration::from_millis(100));
            }
        });
    }

    let (checked, set_checked) = signal(task.is_done());
    let complete_task = Action::new(move |(id, checked): &(_, _)| {
        let id = *id;
        let checked = *checked;
        async move {
            match server::complete_task(id, checked).await {
                Ok(()) => set_timeout(
                    move || set_completion_version.update(|v| *v += 1),
                    std::time::Duration::from_millis(600),
                ),
                Err(e) => {
                    tracing::error!("complete task failed: {e}");
                    set_checked.set(!checked);
                }
            }
        }
    });

    let summary = RwSignal::new(task.summary().to_string());
    let category = RwSignal::new(task.category().to_string());
    let contexts: RwSignal<Vec<String>> = RwSignal::new(task.contexts().iter().map(|c| c.to_string()).collect());
    let priority = RwSignal::new(task.priority().copied());
    let since = *task.since();

    let auto_expand = use_context::<AutoExpandFirst>();
    let is_expanded = move || {
        expanded_task_id.get() == Some(id)
            || auto_expand.is_some_and(|ae| ae.0.get() == Some(id))
    };

    // Handle task row click (except checkbox)
    let handle_task_click = move |_| {
        set_expanded_task_id.update(|current| {
            if *current == Some(id) {
                // Collapse if already expanded
                tracing::info!("collapse task");
                *current = None;
            } else {
                // Expand this task (collapses any other)
                tracing::info!("expand task {id}");
                *current = Some(id);
            }
        });
    };

    view! {
        <div
            node_ref=task_ref
            class=move || {
                let accent = match priority.get() {
                    Some(TaskPriority::A) => priority_a_border,
                    _ => "",
                };
                format!("transition-colors {accent}")
            }
            class:bg-slate-800=is_expanded
        >
            <div
                class="flex items-center px-6 py-4 hover:bg-slate-800 transition-colors cursor-pointer"
                on:click=handle_task_click
            >
                // Checkbox
                <input
                    type="checkbox"
                    class=format!("w-5 h-5 rounded-full border-2 border-slate-600 cursor-pointer appearance-none mr-4 flex-shrink-0 transition-all checked:bg-gradient-to-br {checkbox_checked_classes} relative")
                    prop:checked=move || checked.get()
                    prop:disabled=move || complete_task.pending().get()
                    on:click=move |ev| ev.stop_propagation()
                    on:change=move |ev| {
                        ev.stop_propagation();
                        let checked = event_target_checked(&ev);
                        set_checked.set(checked);
                        complete_task.dispatch((id, checked));
                    }
                />
                // Summary
                <div class="flex-1">
                    <span class=move || if checked.get() && strikethrough_when_done {
                        "text-slate-100 line-through opacity-50"
                    } else {
                        "text-slate-100"
                    }>
                        {move || summary.get()}
                    </span>
                </div>
                // Spinner
                <Show when=move || complete_task.pending().get()>
                    <div class="ml-4 flex-shrink-0">
                        <div class="relative w-5 h-5">
                            <div class=format!("absolute inset-0 rounded-full bg-gradient-to-br {spinner_gradient} opacity-75 animate-ping")></div>
                            <div class=format!("relative rounded-full bg-gradient-to-br {spinner_gradient} w-5 h-5 animate-spin border-2 border-slate-900 border-t-transparent")></div>
                        </div>
                    </div>
                </Show>
            </div>

            // Expanded detail section (Timeline-Style)
            <Show when=is_expanded>
                <TaskDetails task=id summary=summary category=category contexts=contexts priority=priority since=since/>
            </Show>
        </div>
    }
}

#[component]
fn EditableField(
    value: RwSignal<String>,
    #[prop(into)] on_save: UnsyncCallback<String>,
    #[prop(default = false)] multiline: bool,
    #[prop(default = "")] placeholder: &'static str,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let saved = StoredValue::new(value.get_untracked());
    let escape = RwSignal::new(false);
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let do_save = move || {
        let v = value.get_untracked();
        if v == saved.get_value() { return; }
        saved.set_value(v.clone());
        on_save.run(v);
    };
    let do_revert = move || value.set(saved.get_value());
    let do_blur = move || {
        if multiline {
            if let Some(el) = textarea_ref.get() { let _ = el.blur(); }
        } else {
            if let Some(el) = input_ref.get() { let _ = el.blur(); }
        }
    };

    if multiline {
        Either::Left(view! {
            <textarea
                node_ref=textarea_ref
                class=class
                placeholder=placeholder
                rows=4
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    if ev.key() == key::ESCAPE {
                        ev.prevent_default();
                        do_revert();
                        escape.set(true);
                        do_blur();
                    } else if ev.key() == key::ENTER && ev.ctrl_key() {
                        ev.prevent_default();
                        do_save();
                        escape.set(true);
                        do_blur();
                    }
                }
                on:blur=move |_| {
                    if escape.get_untracked() { escape.set(false); } else { do_save(); }
                }
            />
        })
    } else {
        Either::Right(view! {
            <input
                type="text"
                node_ref=input_ref
                class=class
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                on:keydown=move |ev| match ev.key().as_str() {
                    key::ENTER => {
                        ev.prevent_default();
                        do_save();
                        escape.set(true);
                        do_blur();
                    }
                    key::ESCAPE => {
                        ev.prevent_default();
                        do_revert();
                        escape.set(true);
                        do_blur();
                    }
                    _ => {}
                }
                on:blur=move |_| {
                    if escape.get_untracked() { escape.set(false); } else { do_save(); }
                }
            />
        })
    }
}

#[component]
fn TaskDetails<T: for<'a> TaskId<'a>>(task: T, summary: RwSignal<String>, category: RwSignal<String>, contexts: RwSignal<Vec<String>>, priority: RwSignal<Option<TaskPriority>>, since: DateTime<FixedOffset>) -> impl IntoView {
    let id = *task.id();
    let created = task.created();
    let show_since = (since - created.fixed_offset()).abs() >= TimeDelta::minutes(2);
    let created = created.to_relative_time();
    let since = since.to_relative_time();
    let details = Resource::new(move || (), move |_| async move {
        server::fetch_task_details(id).await.map(|(_, details, authors)| (details, authors))
    });
    let available_categories = *use_context::<AvailableCategoriesCtx>()
        .expect("available_categories context missing");
    // Shared context: stable across remounts, only ever grows.
    let available_contexts = *use_context::<AvailableContextsCtx>()
        .expect("available_contexts context missing");
    let edit_mode = use_context::<EditMode>().unwrap_or_default();
    let category_version = use_context::<RwSignal<u32>>().expect("category_version context missing");
    let scroll_to_task_id = use_context::<ScrollToTaskId>();
    let summary_last_saved = StoredValue::new(summary.get_untracked());
    let summary_error: RwSignal<Option<String>> = RwSignal::new(None);
    let rename_task = Action::new(move |value: &String| {
        let value = value.clone();
        summary_error.set(None);
        async move {
            match value.parse::<TaskSummary>() {
                Err(e) => {
                    summary.set(summary_last_saved.get_value());
                    summary_error.set(Some(e.to_string()));
                }
                Ok(s) => match server::rename_task(id, s).await {
                    Ok(()) => summary_last_saved.set_value(value),
                    Err(e) => {
                        tracing::error!("rename task failed: {e}");
                        summary.set(summary_last_saved.get_value());
                        summary_error.set(Some(e.to_string()));
                    }
                },
            }
        }
    });
    let update_time_estimate = Action::new(move |estimate: &Option<TaskTimeEstimate>| {
        let estimate = estimate.clone();
        async move {
            if let Err(e) = server::update_task_time_estimate(id, estimate).await {
                tracing::error!("update time estimate failed: {e}");
            }
        }
    });
    let update_priority = Action::new(move |new_priority: &Option<TaskPriority>| {
        let new_priority = *new_priority;
        async move {
            if let Err(e) = server::update_task_priority(id, new_priority).await {
                tracing::error!("update priority failed: {e}");
            }
        }
    });
    let update_notes = Action::new(move |value: &String| {
        let value = value.clone();
        async move {
            if let Err(e) = server::update_task_notes(id, value).await {
                tracing::error!("update notes failed: {e}");
            }
        }
    });
    let category_last_saved = StoredValue::new(category.get_untracked());
    let category_error: RwSignal<Option<String>> = RwSignal::new(None);
    let update_category = Action::new(move |value: &String| {
        let value = value.clone();
        category_error.set(None);
        async move {
            match value.parse::<TaskCategory>() {
                Err(e) => {
                    category.set(category_last_saved.get_value());
                    category_error.set(Some(e.to_string()));
                }
                Ok(cat) => match server::update_task_category(id, cat).await {
                    Ok(()) => {
                        category_last_saved.set_value(value);
                        category_version.update(|v| *v += 1);
                        if let Some(scroll_to) = scroll_to_task_id {
                            scroll_to.set(Some(id));
                        }
                    }
                    Err(e) => {
                        tracing::error!("update category failed: {e}");
                        category.set(category_last_saved.get_value());
                        category_error.set(Some(e.to_string()));
                    }
                },
            }
        }
    });
    let context_input = RwSignal::new(String::new());
    let failed_additions: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let failed_removals: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let last_saved_contexts: RwSignal<Vec<String>> = RwSignal::new(contexts.get_untracked());
    let replace_contexts = Action::new(move |new_contexts: &Vec<String>| {
        let new_contexts = new_contexts.clone();
        let last_saved = last_saved_contexts.get_untracked();
        let added: Vec<String> = new_contexts.iter()
            .filter(|c| !last_saved.contains(c))
            .cloned()
            .collect();
        let removed: Vec<String> = last_saved.iter()
            .filter(|c| !new_contexts.contains(c))
            .cloned()
            .collect();
        let parsed: Vec<_> = new_contexts.iter()
            .filter_map(|s| s.parse::<TaskContext>().ok())
            .collect();
        async move {
            if let Err(e) = server::replace_task_contexts(id, parsed).await {
                tracing::error!("replace contexts failed: {e}");
                contexts.set(last_saved);   // rollback
                failed_additions.set(added);
                failed_removals.set(removed);
            } else {
                last_saved_contexts.set(new_contexts);
                failed_additions.set(vec![]);
                failed_removals.set(vec![]);
            }
        }
    });
    let update_due_date = Action::new(move |date: &Option<TaskDate>| {
        let date = date.clone();
        async move {
            if let Err(e) = server::update_task_due_date(id, date).await {
                tracing::error!("update due date failed: {e}");
            }
        }
    });
    let update_start_date = Action::new(move |date: &Option<TaskDate>| {
        let date = date.clone();
        async move {
            if let Err(e) = server::update_task_start_date(id, date).await {
                tracing::error!("update start date failed: {e}");
            }
        }
    });

    view! {
        <div class="px-6 pb-4 pt-3 bg-gradient-to-b from-slate-900 via-slate-800 to-slate-900">
            // Summary (editable in edit mode)
            {move || edit_mode.get().then(|| view! {
                <EditableField
                    value=summary
                    on_save=move |v: String| { rename_task.dispatch(v); }
                    class="w-full bg-slate-700 text-slate-100 text-base font-medium rounded px-3 py-2 mb-3 border border-slate-600 focus:border-cyan-500 focus:outline-none"
                />
                {move || summary_error.get().map(|msg| view! {
                    <div class="text-xs text-red-400 mb-2">{msg}</div>
                })}
                <EditableField
                    value=category
                    on_save=move |v: String| { update_category.dispatch(v); }
                    class="w-full bg-slate-700 text-slate-200 text-sm rounded px-3 py-1.5 mb-3 border border-slate-600 focus:border-teal-500 focus:outline-none"
                    placeholder="Category…"
                />
                {move || category_error.get().map(|msg| view! {
                    <div class="text-xs text-red-400 mb-2">{msg}</div>
                })}
                <div class="flex flex-wrap gap-1.5 mb-3">
                    {move || available_categories.get().into_iter().map(|cat| {
                        let label = StoredValue::new(cat);
                        view! {
                            <button
                                type="button"
                                class=move || if category.get() == label.get_value() {
                                    "px-2.5 py-0.5 rounded-full text-xs font-medium bg-teal-600 text-white cursor-default"
                                } else {
                                    "px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors"
                                }
                                on:click=move |_| {
                                    let v = label.get_value();
                                    if category.get_untracked() != v {
                                        category.set(v.clone());
                                        update_category.dispatch(v);
                                    }
                                }
                            >
                                {move || label.get_value()}
                            </button>
                        }
                    }).collect_view()}
                </div>
            })}
            // Vertical timeline with connecting line
            <div class="relative pl-8 space-y-4">
                // Vertical line
                <div class="absolute left-3 top-0 -bottom-4 w-0.5 bg-gradient-to-b from-cyan-500 via-teal-500 to-cyan-500"></div>
                // Contexts
                {move || (!contexts.get().is_empty() || edit_mode.get()).then(|| view! {
                    <div class="relative">
                        <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-slate-700 border-4 border-slate-900 shadow flex items-center justify-center">
                            <span class="text-xs text-slate-300 font-bold">"@"</span>
                        </div>
                        <div class="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">"Contexts"</div>
                        // View mode: plain chips
                        {move || (!edit_mode.get()).then(|| view! {
                            <div class="flex flex-wrap gap-1.5">
                                {contexts.get().into_iter().map(|ctx| view! {
                                    <span class="px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-700 text-slate-300">
                                        {ctx}
                                    </span>
                                }).collect_view()}
                            </div>
                        })}
                        {move || edit_mode.get().then(|| {
                            let add_ctx = move |val: String| {
                                let mut val = val.trim().to_string();
                                if val.is_empty() { return; }
                                if !val.starts_with('@') { val = format!("@{val}"); }
                                contexts.update(|v| { if !v.contains(&val) { v.push(val.clone()); } });
                                available_contexts.update(|v| { if !v.contains(&val) { v.push(val); } });
                                replace_contexts.dispatch(contexts.get_untracked());
                                context_input.set(String::new());
                            };
                            view! {
                                // Suggestions: teal = active (click to remove), gray = inactive (click to add)
                                <div class="flex flex-wrap gap-1.5">
                                    {move || available_contexts.get().into_iter().map(|ctx| {
                                        let label_class = ctx.clone();
                                        let label_click = ctx.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class=move || match (
                                                    contexts.get().contains(&label_class),
                                                    failed_additions.get().contains(&label_class),
                                                    failed_removals.get().contains(&label_class),
                                                ) {
                                                    (false, true, _) => "px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-800 text-red-200 hover:bg-red-700 transition-colors",
                                                    (true, _, true)  => "px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-700 text-amber-100 hover:bg-amber-800 transition-colors",
                                                    (true, _, false) => "px-2.5 py-0.5 rounded-full text-xs font-medium bg-teal-700 text-teal-100 hover:bg-teal-800 transition-colors",
                                                    (false, false, _) => "px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-600 text-slate-300 hover:bg-slate-500 transition-colors",
                                                }
                                                on:click=move |_| {
                                                    if contexts.get_untracked().contains(&label_click) {
                                                        let l = label_click.clone();
                                                        contexts.update(|v| v.retain(|c| *c != l));
                                                    } else {
                                                        let v = label_click.clone();
                                                        contexts.update(|list| list.push(v));
                                                    }
                                                    replace_contexts.dispatch(contexts.get_untracked());
                                                }
                                            >
                                                {ctx}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                                // Free-text input for new contexts
                                <div class="flex items-center gap-2 mt-1.5">
                                    <input
                                        type="text"
                                        class="bg-slate-700 text-slate-200 text-sm rounded px-2 py-1 flex-1 min-w-0 border border-slate-600 focus:border-slate-500 focus:outline-none"
                                        placeholder="@context"
                                        prop:value=move || context_input.get()
                                        on:input=move |ev| context_input.set(event_target_value(&ev))
                                        on:keydown=move |ev| {
                                            if ev.key() == "Enter" {
                                                ev.prevent_default();
                                                add_ctx(context_input.get_untracked());
                                            }
                                        }
                                    />
                                    <button
                                        type="button"
                                        class="text-slate-400 hover:text-teal-400 px-2 py-1 text-sm leading-none"
                                        on:click=move |_| add_ctx(context_input.get_untracked())
                                    >"+"</button>
                                </div>
                            }
                        })}
                    </div>
                })}
                // Priority (from Infos — renders immediately, no fetch needed)
                {
                    let priority_initially_set = priority.get_untracked().is_some();
                    move || (priority_initially_set || edit_mode.get()).then(|| {
                        let marker_class = move || match priority.get() {
                            Some(TaskPriority::A) => "bg-red-500",
                            Some(TaskPriority::B) => "bg-amber-500",
                            Some(TaskPriority::C) | None => "bg-sky-400",
                        };
                        let label_class = move || match priority.get() {
                            Some(TaskPriority::A) => "text-red-400",
                            Some(TaskPriority::B) => "text-amber-500",
                            Some(TaskPriority::C) | None => "text-sky-400",
                        };
                        view! {
                            <div class="relative">
                                <div class=move || format!(
                                    "absolute -left-8 mt-0.5 w-6 h-6 rounded-full border-4 border-slate-900 shadow flex items-center justify-center {}",
                                    marker_class()
                                )>
                                    <span class=move || format!(
                                        "text-xs font-bold {}",
                                        match priority.get() {
                                            Some(TaskPriority::A) => "text-white",
                                            _ => "text-slate-900",
                                        }
                                    )>
                                        {move || priority.get().map(|p| p.to_string()).unwrap_or_default()}
                                    </span>
                                </div>
                                <div class=move || format!(
                                    "text-xs font-semibold uppercase tracking-wide mb-0.5 {}",
                                    label_class()
                                )>"Priority"</div>
                                {move || if edit_mode.get() {
                                    Either::Left(view! {
                                        <div class="flex gap-2">
                                            {TaskPriority::iter().map(|variant| {
                                                let active_class = match variant {
                                                    TaskPriority::A => "w-8 h-8 rounded-full bg-red-500 text-white text-xs font-bold shadow",
                                                    TaskPriority::B => "w-8 h-8 rounded-full bg-amber-500 text-slate-900 text-xs font-bold shadow",
                                                    TaskPriority::C => "w-8 h-8 rounded-full bg-sky-400 text-slate-900 text-xs font-bold shadow",
                                                };
                                                view! {
                                                    <button type="button"
                                                        class=move || if priority.get() == Some(variant) { active_class } else { "w-8 h-8 rounded-full bg-slate-700 text-slate-400 text-xs font-bold" }
                                                        on:click=move |_| {
                                                            let new = if priority.get_untracked() == Some(variant) { None } else { Some(variant) };
                                                            priority.set(new);
                                                            update_priority.dispatch(new);
                                                        }
                                                    >{variant.to_string()}</button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    })
                                } else {
                                    Either::Right(view! {
                                        <div class="text-sm text-slate-200">{move || match priority.get() {
                                            Some(TaskPriority::A) => "Critical",
                                            Some(TaskPriority::B) => "Important",
                                            Some(TaskPriority::C) => "Routine",
                                            None => "",
                                        }}</div>
                                    })
                                }}
                            </div>
                        }
                    })
                }
                <Suspense>
                    {move || {
                        Suspend::new(async move {
                            details.await.map(|(task, authors)| {
                                let due_date_value = RwSignal::new(task.due_date().cloned());
                                let due_date_initially_set = due_date_value.get_untracked().is_some();
                                let start_date_value = RwSignal::new(task.start_date().cloned());
                                let start_date_initially_set = start_date_value.get_untracked().is_some();
                                let time_estimate_value = RwSignal::new(task.time_estimate().cloned());
                                let time_estimate_initially_set = time_estimate_value.get_untracked().is_some();
                                let notes_value = RwSignal::new(task.notes().into_owned().unwrap_or_default());
                                let notes_initially_set = !notes_value.get_untracked().is_empty();
                                view! {
                                    {move || (due_date_initially_set || edit_mode.get()).then(|| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-teal-400 uppercase tracking-wide mb-0.5">
                                                {move || due_date_value.get().map(|d| if d.soft { "Due Date (approx.)" } else { "Due Date" }).unwrap_or("Due Date")}
                                            </div>
                                            {move || if edit_mode.get() {
                                                Either::Left(view! {
                                                    <div class="flex flex-col gap-1">
                                                        <div class="flex items-center gap-2">
                                                            <input
                                                                type="datetime-local"
                                                                class="bg-slate-700 text-slate-200 text-sm rounded px-2 py-1 flex-1 min-w-0"
                                                                prop:value=move || due_date_value.get().as_ref().map(|d| DateTimeLocal::from(d.date).to_string()).unwrap_or_default()
                                                                on:change=move |ev| {
                                                                    let s = event_target_value(&ev);
                                                                    let soft = due_date_value.get_untracked().map(|d| d.soft).unwrap_or(false);
                                                                    let new = s.parse::<DateTimeLocal>().ok().map(|dl| TaskDate { date: dl.into(), soft });
                                                                    due_date_value.set(new.clone());
                                                                    update_due_date.dispatch(new);
                                                                }
                                                            />
                                                            <button type="button"
                                                                class="text-slate-400 hover:text-red-400 px-1 py-1 text-sm leading-none"
                                                                on:click=move |_| {
                                                                    due_date_value.set(None);
                                                                    update_due_date.dispatch(None);
                                                                }
                                                            >"×"</button>
                                                        </div>
                                                        <label class="flex items-center gap-2 text-xs text-slate-400 select-none">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=move || due_date_value.get().map(|d| d.soft).unwrap_or(false)
                                                                on:change=move |ev| {
                                                                    let soft = event_target_checked(&ev);
                                                                    let new = due_date_value.get_untracked().map(|mut d| { d.soft = soft; d });
                                                                    due_date_value.set(new.clone());
                                                                    update_due_date.dispatch(new);
                                                                }
                                                            />
                                                            "approximate"
                                                        </label>
                                                    </div>
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <div class="text-sm text-slate-200">
                                                        {move || due_date_value.get().map(|d| d.date.to_relative_time()).unwrap_or_default()}
                                                    </div>
                                                })
                                            }}
                                        </div>
                                    })}
                                    {move || (start_date_initially_set || edit_mode.get()).then(|| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-sky-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-sky-400 uppercase tracking-wide mb-0.5">
                                                {move || start_date_value.get().map(|d| if d.soft { "Start Date (approx.)" } else { "Start Date" }).unwrap_or("Start Date")}
                                            </div>
                                            {move || if edit_mode.get() {
                                                Either::Left(view! {
                                                    <div class="flex flex-col gap-1">
                                                        <div class="flex items-center gap-2">
                                                            <input
                                                                type="datetime-local"
                                                                class="bg-slate-700 text-slate-200 text-sm rounded px-2 py-1 flex-1 min-w-0"
                                                                prop:value=move || start_date_value.get().as_ref().map(|d| DateTimeLocal::from(d.date).to_string()).unwrap_or_default()
                                                                on:change=move |ev| {
                                                                    let s = event_target_value(&ev);
                                                                    let soft = start_date_value.get_untracked().map(|d| d.soft).unwrap_or(false);
                                                                    let new = s.parse::<DateTimeLocal>().ok().map(|dl| TaskDate { date: dl.into(), soft });
                                                                    start_date_value.set(new.clone());
                                                                    update_start_date.dispatch(new);
                                                                }
                                                            />
                                                            <button type="button"
                                                                class="text-slate-400 hover:text-red-400 px-1 py-1 text-sm leading-none"
                                                                on:click=move |_| {
                                                                    start_date_value.set(None);
                                                                    update_start_date.dispatch(None);
                                                                }
                                                            >"×"</button>
                                                        </div>
                                                        <label class="flex items-center gap-2 text-xs text-slate-400 select-none">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=move || start_date_value.get().map(|d| d.soft).unwrap_or(false)
                                                                on:change=move |ev| {
                                                                    let soft = event_target_checked(&ev);
                                                                    let new = start_date_value.get_untracked().map(|mut d| { d.soft = soft; d });
                                                                    start_date_value.set(new.clone());
                                                                    update_start_date.dispatch(new);
                                                                }
                                                            />
                                                            "approximate"
                                                        </label>
                                                    </div>
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <div class="text-sm text-slate-200">
                                                        {move || start_date_value.get().map(|d| d.date.to_relative_time()).unwrap_or_default()}
                                                    </div>
                                                })
                                            }}
                                        </div>
                                    })}
                                    {move || (time_estimate_initially_set || edit_mode.get()).then(|| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-md bg-amber-500 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-slate-900" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M6 2a1 1 0 011-1h6a1 1 0 011 1v1a1 1 0 01-.293.707L12 5.414V7a1 1 0 01-.293.707L10 9.414l-1.707-1.707A1 1 0 018 7V5.414L6.293 3.707A1 1 0 016 3V2zm0 16a1 1 0 001 1h6a1 1 0 001-1v-1a1 1 0 00-.293-.707L12 14.586V13a1 1 0 00-.293-.707L10 10.586l-1.707 1.707A1 1 0 008 13v1.586l-1.707 1.707A1 1 0 006 17v1z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-amber-500 uppercase tracking-wide mb-0.5">"Estimate"</div>
                                            {move || if edit_mode.get() {
                                                Either::Left(view! {
                                                    <div class="flex flex-wrap gap-2">
                                                        {TaskTimeEstimate::iter().map(|variant| {
                                                            let label = variant.short_label();
                                                            view! {
                                                            <button type="button"
                                                                class=move || if time_estimate_value.get() == Some(variant) {
                                                                    "px-2.5 py-1 rounded bg-amber-500 text-slate-900 text-xs font-bold shadow"
                                                                } else {
                                                                    "px-2.5 py-1 rounded bg-slate-700 text-slate-400 text-xs font-bold"
                                                                }
                                                                on:click=move |_| {
                                                                    let new = if time_estimate_value.get_untracked() == Some(variant) { None } else { Some(variant) };
                                                                    time_estimate_value.set(new.clone());
                                                                    update_time_estimate.dispatch(new);
                                                                }
                                                            >{label}</button>
                                                        }}).collect_view()}
                                                    </div>
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <div class="text-sm text-slate-200">
                                                        {move || time_estimate_value.get().map(|t| t.to_string()).unwrap_or_default()}
                                                    </div>
                                                })
                                            }}
                                        </div>
                                    })}
                                    {move || (notes_initially_set || edit_mode.get()).then(|| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-slate-600 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M9 2a2 2 0 00-2 2v8a2 2 0 002 2h6a2 2 0 002-2V6.414A2 2 0 0016.414 5L14 2.586A2 2 0 0012.586 2H9z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-slate-300 uppercase tracking-wide mb-1">"Notes"</div>
                                            {move || if edit_mode.get() {
                                                Either::Left(view! {
                                                    <EditableField
                                                        multiline=true
                                                        value=notes_value
                                                        on_save=move |v: String| { update_notes.dispatch(v); }
                                                        class="w-full bg-slate-800 text-slate-300 text-sm leading-relaxed rounded-lg px-3 py-2 border border-slate-700 focus:border-slate-500 focus:outline-none resize-none"
                                                        placeholder="Add notes…"
                                                    />
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <div class="text-sm text-slate-300 leading-relaxed bg-slate-800 p-3 rounded-lg border border-slate-700 shadow-sm whitespace-pre-wrap">
                                                        {notes_value.get()}
                                                    </div>
                                                })
                                            }}
                                        </div>
                                    })}
                                    // Authors (fetched from server, shown in all views)
                                    {(!authors.is_empty()).then(|| {
                                        view! {
                                            <div class="relative">
                                                <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-violet-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                    <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                        <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
                                                    </svg>
                                                </div>
                                                <div class="text-xs font-semibold text-violet-400 uppercase tracking-wide mb-0.5">"Authors"</div>
                                                <div class="space-y-1">
                                                    {authors.iter().map(|(name, ts)| {
                                                        // Insert thin spaces (U+2009) around colons for readability
                                                        let display_name = name.replace(":", "\u{2009}:\u{2009}");
                                                        let display_ts = ts.to_relative_time();
                                                        view! {
                                                            <div class="flex items-baseline gap-2">
                                                                <span class="text-sm text-slate-200">{display_name}</span>
                                                                <span class="text-xs text-slate-500">{display_ts}</span>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }
                                    })}
                                }
                            })
                        })
                    }}
                </Suspense>
                // Since (only if different from created, minute-precise)
                {show_since.then(|| view! {
                    <div class="relative">
                        <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-slate-600 border-4 border-slate-900 shadow flex items-center justify-center">
                            <div class="w-2 h-2 rounded-full bg-white"></div>
                        </div>
                        <div class="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-0.5">"Last status change"</div>
                        <div class="text-sm text-slate-200">{since}</div>
                    </div>
                })}
                // Created
                <div class="relative">
                    <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-cyan-700 border-4 border-slate-900 shadow flex items-center justify-center">
                        <div class="w-2 h-2 rounded-full bg-white"></div>
                    </div>
                    <div class="text-xs font-semibold text-cyan-400 uppercase tracking-wide mb-0.5">"Created"</div>
                    <div class="text-sm text-slate-200">{created}</div>
                    <div class="text-xs text-slate-500 font-mono mt-0.5">{id.to_string()}</div>
                </div>
            </div>
        </div>
    }
}

struct DateTimeLocal(DateTime<FixedOffset>);

impl DateTimeLocal {
    fn to_display(&self) -> String {
        self.0.with_timezone(&Local).format("%d.%m.%Y %H:%M").to_string()
    }
}

impl Display for DateTimeLocal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.with_timezone(&Local).format("%Y-%m-%dT%H:%M"))
    }
}

impl From<DateTimeLocal> for DateTime<FixedOffset> {
    fn from(dl: DateTimeLocal) -> Self {
        dl.0
    }
}

impl From<DateTime<FixedOffset>> for DateTimeLocal {
    fn from(dt: DateTime<FixedOffset>) -> Self {
        Self(dt)
    }
}

impl FromStr for DateTimeLocal {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
            .ok()
            .and_then(|ndt| Local.from_local_datetime(&ndt).single())
            .map(|dt| Self(dt.fixed_offset()))
            .ok_or(())
    }
}

trait ToRelativeTime {
    fn to_relative_time(&self) -> String;
}

impl<Tz: TimeZone> ToRelativeTime for DateTime<Tz> {
    fn to_relative_time(&self) -> String {
        let plural = |num| if num == 1 { "" } else { "s" };
        let duration = Utc::now().signed_duration_since(self);
        if duration > TimeDelta::zero() {
            // self is in the past
            if duration < TimeDelta::seconds(60) {
                "Just now".to_string()
            } else if duration < TimeDelta::minutes(60) {
                let mins = duration.num_minutes();
                format!("{} minute{} ago", mins, plural(mins))
            } else if duration < TimeDelta::hours(24) {
                let hours = duration.num_hours();
                format!("{} hour{} ago", hours, plural(hours))
            } else if duration < TimeDelta::days(7) {
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                format!(
                    "{} day{} and {} hour{} ago",
                    days,
                    plural(days),
                    hours,
                    plural(hours)
                )
            } else {
                DateTimeLocal::from(self.with_timezone(&Local).fixed_offset()).to_display()
            }
        } else {
            let duration = duration.abs();
            if duration < TimeDelta::seconds(60) {
                "Just now".to_string()
            } else if duration < TimeDelta::minutes(60) {
                let mins = duration.num_minutes();
                format!("in {} minute{}", mins, plural(mins))
            } else if duration < TimeDelta::hours(24) {
                let hours = duration.num_hours();
                let mins = duration.num_minutes() % 60;
                format!(
                    "in {} hour{} and {} minute{}",
                    hours,
                    plural(hours),
                    mins,
                    plural(mins)
                )
            } else if duration < TimeDelta::days(7) {
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let mins = duration.num_minutes() % 60;
                format!(
                    "in {} day{}, {} hour{} and {} minute{}",
                    days,
                    plural(days),
                    hours,
                    plural(hours),
                    mins,
                    plural(mins)
                )
            } else {
                DateTimeLocal::from(self.with_timezone(&Local).fixed_offset()).to_display()
            }
        }
    }
}
