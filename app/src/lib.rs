pub mod cache;
mod error_template;
pub mod server;

use crate::error_template::ErrorTemplate;

use kid_types::{TaskDateEstimationRef, TaskDetails, TaskFilter, TaskId, TaskInfos, TaskPriority, Uuid};

use chrono::prelude::*;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use strum::{EnumCount, FromRepr};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
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

    fn task_filter(self) -> TaskFilter {
        match self {
            View::MyDay => TaskFilter::Todo,
            View::WhatIFinished => TaskFilter::Done,
            View::QuickWins => TaskFilter::HasTimeEstimate,
            View::RecentlyChanged => TaskFilter::RecentlyChanged,
        }
    }

    fn sort_tasks<T: for<'a> TaskId<'a> + for<'a> TaskInfos<'a>>(self, tasks: &mut Vec<T>) {
        let created_asc = |a: &T, b: &T| a.id().cmp(b.id());        
        let since_asc   = |a: &T, b: &T| a.since().cmp(b.since());
        let since_desc  = |a: &T, b: &T| b.since().cmp(a.since());
        match self {
            View::MyDay          => tasks.sort_by(created_asc),
            View::WhatIFinished  => tasks.sort_by(since_desc),
            View::QuickWins      => tasks.sort_by(since_asc),
            View::RecentlyChanged => tasks.sort_by(since_desc),
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

    let add_task = ServerMultiAction::<server::AddTask>::new();
    let delete_task = ServerAction::<server::DeleteTask>::new();
    let (completion_version, set_completion_version) = signal(0u32);

    let current_view = RwSignal::new(View::MyDay);
    let switch_count = RwSignal::new(0u32);
    let edit_mode = RwSignal::new(false);
    provide_context(edit_mode);

    let task_list = Resource::new(
        move || {
            (
                delete_task.version().get(),
                add_task.version().get(),
                delete_task.version().get(),
                completion_version.get(),
                current_view.get(),
            )
        },
        move |_| server::fetch_task_list(current_view.get_untracked().task_filter()),
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
        <MultiActionForm action=add_task>
            <label>"Add a Task" <input type="text" name="summary"/></label>
            <input type="submit" value="Add"/>
        </MultiActionForm>
        <div class="min-h-screen bg-gradient-to-br from-slate-950 to-slate-900">
            <div class="max-w-2xl mx-auto min-h-screen bg-slate-900 shadow-2xl">
                <header class=move || format!(
                    "px-6 pt-6 pb-5 bg-gradient-to-br {} text-white select-none",
                    current_view.get().header_gradient()
                )>
                    <div class="flex items-center gap-2">
                        // Spacer (balances edit icon on the right)
                        <div class="w-8 h-8 flex-shrink-0"></div>
                        // Left arrow
                        <button
                            type="button"
                            class=move || {
                                let opacity = match current_view.get().prev() {
                                    Some(_) => arrow_opacity_class(switch_count.get()),
                                    None => "opacity-0 pointer-events-none",
                                };
                                format!("w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
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
                        <h1
                            aria-live="polite"
                            class="flex-1 text-center text-3xl font-semibold"
                        >
                            {move || current_view.get().title()}
                        </h1>
                        // Right arrow
                        <button
                            type="button"
                            class=move || {
                                let opacity = match current_view.get().next() {
                                    Some(_) => arrow_opacity_class(switch_count.get()),
                                    None => "opacity-0 pointer-events-none",
                                };
                                format!("w-8 h-8 flex items-center justify-center rounded-full transition-opacity {opacity}")
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
                        // Edit mode toggle icon
                        <button
                            type="button"
                            class=move || if edit_mode.get() {
                                "w-8 h-8 flex-shrink-0 flex items-center justify-center rounded-full bg-amber-400 text-slate-900 transition-colors"
                            } else {
                                "w-8 h-8 flex-shrink-0 flex items-center justify-center rounded-full text-white/50 hover:text-white/80 hover:bg-white/10 transition-colors"
                            }
                            on:click=move |_| edit_mode.update(|m| *m = !*m)
                            aria-pressed=move || edit_mode.get()
                            aria-label="Toggle edit mode"
                        >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"/>
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
                <Show when=move || edit_mode.get()>
                    <div class="bg-amber-400 text-slate-900 text-sm font-semibold text-center py-1.5 select-none">
                        "Edit Mode"
                    </div>
                </Show>
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
                                                    each=move || {
                                                        let mut list = task_list.clone();
                                                        view.sort_tasks(&mut list);
                                                        list
                                                    }
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
            </div>
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

    let summary = task.summary().to_string();

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
                        {summary}
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
                <TaskDetails task=id/>
            </Show>
        </div>
    }
}

#[component]
fn TaskDetails<T: for<'a> TaskId<'a>>(task: T) -> impl IntoView {
    let id = *task.id();
    let created = task.created().to_relative_time();
    let details = Resource::new(move || (), move |_| server::fetch_task_details(id));

    view! {
        <div class="px-6 pb-4 pt-3 bg-gradient-to-b from-slate-900 via-slate-800 to-slate-900">
            // Vertical timeline with connecting line
            <div class="relative pl-8 space-y-4">
                // Vertical line
                <div class="absolute left-3 top-0 bottom-0 w-0.5 bg-gradient-to-b from-cyan-500 via-teal-500 to-cyan-500"></div>
                <Suspense>
                    {move || {
                        Suspend::new(async move {
                            details.await.map(|task| {
                                let priority = task.priority();
                                let due_date = task.due_date(&Local).map(|t| t.to_relative_time());
                                let start_date = task.start_date(&Local).map(|t| t.to_relative_time());
                                let time_estimate = task.time_estimate().map(|t| t.to_string());
                                let context = task.context().into_owned();
                                let notes = task.notes().into_owned();
                                view! {
                                    {priority.map(|priority| view! {
                                        // Priority badge with color coding
                                        <div class="relative">
                                            <div class={format!(
                                                "absolute -left-8 mt-0.5 w-6 h-6 rounded-full border-4 border-slate-900 shadow flex items-center justify-center {}",
                                                match priority {
                                                    TaskPriority::A => "bg-red-500",
                                                    TaskPriority::B => "bg-amber-500",
                                                    TaskPriority::C => "bg-sky-400",
                                                }
                                            )}>
                                                <span class={format!(
                                                    "text-xs font-bold {}",
                                                    match priority {
                                                        TaskPriority::A => "text-white",
                                                        TaskPriority::B => "text-slate-900",
                                                        TaskPriority::C => "text-slate-900",
                                                    }
                                                )}>{priority.to_string()}</span>
                                            </div>
                                            <div class={format!(
                                                "text-xs font-semibold uppercase tracking-wide mb-0.5 {}",
                                                match priority {
                                                    TaskPriority::A => "text-red-400",
                                                    TaskPriority::B => "text-amber-500",
                                                    TaskPriority::C => "text-sky-400",
                                                }
                                            )}>"Priority"</div>
                                            <div class="text-sm text-slate-200">{
                                                match priority {
                                                    TaskPriority::A => "Critical",
                                                    TaskPriority::B => "Important",
                                                    TaskPriority::C => "Routine",
                                                }
                                            }</div>
                                        </div>
                                    })}
                                    {due_date.map(|due_date| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-teal-400 uppercase tracking-wide mb-0.5">"Due Date"</div>
                                            <div class="text-sm text-slate-200">{due_date}</div>
                                        </div>
                                    })}
                                    {start_date.map(|start_date| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-sky-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-sky-400 uppercase tracking-wide mb-0.5">"Start Date"</div>
                                            <div class="text-sm text-slate-200">{start_date}</div>
                                        </div>
                                    })}
                                    {time_estimate.map(|time_estimate| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-md bg-amber-500 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-slate-900" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M6 2a1 1 0 011-1h6a1 1 0 011 1v1a1 1 0 01-.293.707L12 5.414V7a1 1 0 01-.293.707L10 9.414l-1.707-1.707A1 1 0 018 7V5.414L6.293 3.707A1 1 0 016 3V2zm0 16a1 1 0 001 1h6a1 1 0 001-1v-1a1 1 0 00-.293-.707L12 14.586V13a1 1 0 00-.293-.707L10 10.586l-1.707 1.707A1 1 0 008 13v1.586l-1.707 1.707A1 1 0 006 17v1z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-amber-500 uppercase tracking-wide mb-0.5">"Estimate"</div>
                                            <div class="text-sm text-slate-200">{time_estimate}</div>
                                        </div>
                                    })}
                                    {context.map(|context| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-700 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-teal-400 uppercase tracking-wide mb-0.5">"Context"</div>
                                            <div class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-teal-500 text-white shadow-sm">
                                                {context}
                                            </div>
                                        </div>
                                    })}
                                    {notes.map(|notes| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-slate-600 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M9 2a2 2 0 00-2 2v8a2 2 0 002 2h6a2 2 0 002-2V6.414A2 2 0 0016.414 5L14 2.586A2 2 0 0012.586 2H9z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-slate-300 uppercase tracking-wide mb-1">"Notes"</div>
                                            <div class="text-sm text-slate-300 leading-relaxed bg-slate-800 p-3 rounded-lg border border-slate-700 shadow-sm whitespace-pre-wrap">
                                                {notes}
                                            </div>
                                        </div>
                                    })}
                                }
                            })
                        })
                    }}
                </Suspense>
                // Created
                <div class="relative">
                    <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-cyan-700 border-4 border-slate-900 shadow flex items-center justify-center">
                        <div class="w-2 h-2 rounded-full bg-white"></div>
                    </div>
                    <div class="text-xs font-semibold text-cyan-400 uppercase tracking-wide mb-0.5">"Created"</div>
                    <div class="text-sm text-slate-200">{created}</div>
                </div>
            </div>
        </div>
    }
}

trait ToRelativeTime {
    fn to_relative_time(&self) -> String;
}

impl<Tz: TimeZone> ToRelativeTime for TaskDateEstimationRef<'_, Tz> {
    fn to_relative_time(&self) -> String {
        match self {
            Self::Guess(s) => s.to_string(),
            Self::Precise(date) => date.to_relative_time(),
        }
    }
}

impl<Tz: TimeZone> ToRelativeTime for DateTime<Tz> {
    fn to_relative_time(&self) -> String {
        use chrono::TimeDelta;
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
                self.with_timezone(&Local).to_rfc2822()
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
                self.with_timezone(&Local).to_rfc2822()
            }
        }
    }
}
