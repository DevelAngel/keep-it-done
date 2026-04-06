pub mod cache;
mod error_template;
pub mod server;

use crate::error_template::ErrorTemplate;

use kid_types::{TaskCategory, TaskDate, TaskDetails, TaskId, TaskInfos, TaskPriority, TaskTimeEstimate, Uuid};
use strum::IntoEnumIterator;

use chrono::prelude::*;
use chrono::TimeDelta;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
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

impl Deref for EditMode {
    type Target = RwSignal<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, EnumCount, FromRepr)]
enum View {
    MyDay,
    WhatIFinished,
    QuickWins,
    RecentlyChanged,
}

impl View {
    fn title(self) -> &'static str {
        match self {
            View::MyDay => "My Day",
            View::WhatIFinished => "What I Finished",
            View::QuickWins => "Quick Wins",
            View::RecentlyChanged => "Recent Changes",
        }
    }

    fn header_gradient(self) -> &'static str {
        match self {
            View::MyDay => "from-cyan-600 to-teal-700",
            View::WhatIFinished => "from-teal-600 to-emerald-700",
            View::QuickWins => "from-amber-500 to-amber-700",
            View::RecentlyChanged => "from-sky-500 to-sky-700",
        }
    }

    fn dot_active_color(self) -> &'static str {
        match self {
            View::MyDay => "bg-cyan-200",
            View::WhatIFinished => "bg-emerald-200",
            View::QuickWins => "bg-amber-200",
            View::RecentlyChanged => "bg-sky-200",
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
            View::MyDay          => "Open tasks · by creation date",
            View::WhatIFinished  => "Completed tasks · most recent first",
            View::QuickWins      => "Open tasks with estimate · shortest first",
            View::RecentlyChanged => "Changed within 24 h · most recent first",
        }
    }

    fn empty_message(self) -> &'static str {
        match self {
            View::MyDay => "Nothing left for today.",
            View::WhatIFinished => "Nothing finished yet.",
            View::QuickWins => "No estimated tasks.",
            View::RecentlyChanged => "No changes in the last 24 hours.",
        }
    }

    fn checkbox_checked_classes(self) -> &'static str {
        match self {
            View::MyDay => "checked:from-cyan-500 checked:to-teal-600 checked:border-cyan-500",
            View::WhatIFinished => {
                "checked:from-teal-500 checked:to-emerald-600 checked:border-teal-500"
            }
            View::QuickWins => {
                "checked:from-amber-400 checked:to-amber-600 checked:border-amber-400"
            }
            View::RecentlyChanged => {
                "checked:from-sky-400 checked:to-sky-600 checked:border-sky-400"
            }
        }
    }

    fn spinner_gradient(self) -> &'static str {
        match self {
            View::MyDay => "from-cyan-500 to-teal-600",
            View::WhatIFinished => "from-teal-500 to-emerald-600",
            View::QuickWins => "from-amber-400 to-amber-600",
            View::RecentlyChanged => "from-sky-400 to-sky-600",
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

#[component]
fn TaskList() -> impl IntoView {
    let (expanded_task_id, set_expanded_task_id) = signal(None::<Uuid>);

    let add_task = Action::new(move |summary: &String| {
        let summary = summary.clone();
        async move { server::add_task(summary).await }
    });
    let delete_task = ServerAction::<server::DeleteTask>::new();
    let (completion_version, set_completion_version) = signal(0u32);

    let current_view = RwSignal::new(View::MyDay);
    let switch_count = RwSignal::new(0u32);
    let edit_mode = EditMode::default();
    provide_context(edit_mode);

    window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == key::ESCAPE && !ev.default_prevented() && edit_mode.get() {
            edit_mode.update(|m| *m = false);
        }
    });

    let task_list = Resource::new(
        move || (add_task.version().get(), delete_task.version().get(), completion_version.get(), current_view.get()),
        move |_| async move {
            match current_view.get_untracked() {
                View::MyDay          => server::fetch_my_day().await,
                View::WhatIFinished  => server::fetch_what_i_finished().await,
                View::QuickWins      => server::fetch_quick_wins().await,
                View::RecentlyChanged => server::fetch_recently_changed().await,
            }
        },
    );

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
                    "px-6 pt-4 pb-5 bg-gradient-to-br {} text-white select-none",
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
                                format!("absolute left-0 top-[58%] -translate-y-1/2 w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
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
                                format!("absolute right-0 top-[58%] -translate-y-1/2 w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
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
                    // Page indicator dots
                    <div class="flex justify-center items-center gap-2 mt-3">
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
                    </div>
                </header>
                <button
                    type="button"
                    class="w-full flex items-center justify-center py-3 select-none"
                    on:click=move |_| edit_mode.update(|m| *m = !*m)
                    aria-pressed=move || edit_mode.get()
                    aria-label="Toggle edit mode"
                >
                    {move || if edit_mode.get() {
                        Either::Left(view! {
                            <span class="w-full bg-amber-400 text-slate-900 text-sm font-semibold text-center py-1 transition-all">
                                "Edit Mode"
                            </span>
                        })
                    } else {
                        Either::Right(view! {
                            <span class="w-16 h-1 rounded-full bg-slate-600 transition-all"></span>
                        })
                    }}
                </button>
                <div class="py-2">
                    <Suspense fallback=move || view! {
                        <div class="px-6 py-6 text-center text-slate-400">"Loading tasks..."</div>
                    }>
                        <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors/> }>
                            {move || {
                                let view = current_view.get();
                                Suspend::new(async move {
                                    task_list.await.map(|task_list| {
                                        if task_list.is_empty() {
                                            Either::Left(view! {
                                                <p class="px-6 py-6 text-center text-slate-400">
                                                    {view.empty_message()}
                                                </p>
                                            })
                                        } else {
                                            Either::Right(view! {
                                                <For
                                                    each=move || task_list.clone()
                                                    key=|task| *task.id()
                                                    children=move |task| {
                                                        view! {
                                                            <TaskItem task=task
                                                                expanded_task_id=expanded_task_id
                                                                set_expanded_task_id=set_expanded_task_id
                                                                set_completion_version=set_completion_version
                                                                strikethrough_when_done={view == View::MyDay}
                                                                checkbox_checked_classes={view.checkbox_checked_classes()}
                                                                spinner_gradient={view.spinner_gradient()}
                                                            />
                                                        }
                                                    }
                                                />
                                            })
                                        }
                                    })
                                })
                            }}
                        </ErrorBoundary>
                    </Suspense>
                </div>
                <Show when=move || edit_mode.get()>
                    <AddTaskForm on_add=move |summary: String| { add_task.dispatch(summary); }/>
                </Show>
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
) -> impl IntoView {
    let id = *task.id();

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
    let since = *task.since();

    let is_expanded = move || expanded_task_id.get() == Some(id);

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
            class="border-b border-slate-700 transition-colors"
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
                <TaskDetails task=id summary=summary category=category since=since/>
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
fn TaskDetails<T: for<'a> TaskId<'a>>(task: T, summary: RwSignal<String>, category: RwSignal<String>, since: DateTime<FixedOffset>) -> impl IntoView {
    let id = *task.id();
    let created = task.created();
    let show_since = (since - created.fixed_offset()).abs() >= TimeDelta::minutes(2);
    let created = created.to_relative_time();
    let since = since.to_relative_time();
    let details = Resource::new(move || (), move |_| server::fetch_task_details(id));
    let edit_mode = use_context::<EditMode>().unwrap_or_default();
    let rename_task = Action::new(move |value: &String| {
        let value = value.clone();
        async move {
            if let Err(e) = server::rename_task(id, value).await {
                tracing::error!("rename task failed: {e}");
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
    let update_priority = Action::new(move |priority: &Option<TaskPriority>| {
        let priority = priority.clone();
        async move {
            if let Err(e) = server::update_task_priority(id, priority).await {
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
                    Ok(()) => category_last_saved.set_value(value),
                    Err(e) => {
                        tracing::error!("update category failed: {e}");
                        category.set(category_last_saved.get_value());
                        category_error.set(Some(e.to_string()));
                    }
                },
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
            })}
            // Vertical timeline with connecting line
            <div class="relative pl-8 space-y-4">
                // Vertical line
                <div class="absolute left-3 top-0 -bottom-4 w-0.5 bg-gradient-to-b from-cyan-500 via-teal-500 to-cyan-500"></div>
                <Suspense>
                    {move || {
                        Suspend::new(async move {
                            details.await.map(|task| {
                                let priority_value = RwSignal::new(task.priority().cloned());
                                let priority_initially_set = priority_value.get_untracked().is_some();
                                let due_date_value = RwSignal::new(task.due_date().cloned());
                                let due_date_initially_set = due_date_value.get_untracked().is_some();
                                let start_date_value = RwSignal::new(task.start_date().cloned());
                                let start_date_initially_set = start_date_value.get_untracked().is_some();
                                let time_estimate_value = RwSignal::new(task.time_estimate().cloned());
                                let time_estimate_initially_set = time_estimate_value.get_untracked().is_some();
                                let category_initially_set = !category.get_untracked().is_empty();
                                let notes_value = RwSignal::new(task.notes().into_owned().unwrap_or_default());
                                let notes_initially_set = !notes_value.get_untracked().is_empty();
                                view! {
                                    {move || (priority_initially_set || edit_mode.get()).then(|| {
                                        let marker_class = move || match priority_value.get() {
                                            Some(TaskPriority::A) => "bg-red-500",
                                            Some(TaskPriority::B) => "bg-amber-500",
                                            Some(TaskPriority::C) | None => "bg-sky-400",
                                        };
                                        let label_class = move || match priority_value.get() {
                                            Some(TaskPriority::A) => "text-red-400",
                                            Some(TaskPriority::B) => "text-amber-500",
                                            Some(TaskPriority::C) | None => "text-sky-400",
                                        };
                                        view! {
                                            // Priority badge with color coding
                                            <div class="relative">
                                                <div class=move || format!(
                                                    "absolute -left-8 mt-0.5 w-6 h-6 rounded-full border-4 border-slate-900 shadow flex items-center justify-center {}",
                                                    marker_class()
                                                )>
                                                    <span class=move || format!(
                                                        "text-xs font-bold {}",
                                                        match priority_value.get() {
                                                            Some(TaskPriority::A) => "text-white",
                                                            _ => "text-slate-900",
                                                        }
                                                    )>
                                                        {move || priority_value.get().map(|p| p.to_string()).unwrap_or_default()}
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
                                                                        class=move || if priority_value.get() == Some(variant) { active_class } else { "w-8 h-8 rounded-full bg-slate-700 text-slate-400 text-xs font-bold" }
                                                                        on:click=move |_| {
                                                                            let new = if priority_value.get_untracked() == Some(variant) { None } else { Some(variant) };
                                                                            priority_value.set(new.clone());
                                                                            update_priority.dispatch(new);
                                                                        }
                                                                    >{variant.to_string()}</button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    })
                                                } else {
                                                    Either::Right(view! {
                                                        <div class="text-sm text-slate-200">{move || match priority_value.get() {
                                                            Some(TaskPriority::A) => "Critical",
                                                            Some(TaskPriority::B) => "Important",
                                                            Some(TaskPriority::C) => "Routine",
                                                            None => "",
                                                        }}</div>
                                                    })
                                                }}
                                            </div>
                                        }
                                    })}
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
                                    {move || (category_initially_set || edit_mode.get()).then(|| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-teal-400 uppercase tracking-wide mb-0.5">"Category"</div>
                                            {move || if edit_mode.get() {
                                                Either::Left(view! {
                                                    <EditableField
                                                        value=category
                                                        on_save=move |v: String| { update_category.dispatch(v); }
                                                        class="w-full bg-slate-700 text-slate-200 text-sm rounded px-2 py-1 border border-slate-600 focus:border-teal-500 focus:outline-none"
                                                        placeholder="Add category…"
                                                    />
                                                    {move || category_error.get().map(|msg| view! {
                                                        <div class="text-xs text-red-400 mt-1">{msg}</div>
                                                    })}
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <div class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-teal-500 text-white shadow-sm">
                                                        {category.get()}
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
