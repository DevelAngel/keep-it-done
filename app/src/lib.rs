pub mod cache;
mod error_template;
pub mod server;

use crate::error_template::ErrorTemplate;

use kid_types::{TaskDateEstimationRef, TaskDetails, TaskId, TaskInfos, TaskPriority, Uuid};

use chrono::prelude::*;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

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

#[component]
fn TaskList() -> impl IntoView {
    let (expanded_task_id, set_expanded_task_id) = signal(None::<Uuid>);

    let add_task = ServerMultiAction::<server::AddTask>::new();
    let delete_task = ServerAction::<server::DeleteTask>::new();

    let task_list = Resource::new(
        move || {
            (
                delete_task.version().get(),
                add_task.version().get(),
                delete_task.version().get(),
            )
        },
        move |_| server::fetch_task_list(),
    );

    view! {
        <MultiActionForm action=add_task>
            <label>"Add a Task" <input type="text" name="summary"/></label>
            <input type="submit" value="Add"/>
        </MultiActionForm>
        <div class="min-h-screen bg-gradient-to-br from-slate-950 to-slate-900">
            <div class="max-w-2xl mx-auto min-h-screen bg-slate-900 shadow-2xl">
                <header class="px-6 py-8 bg-gradient-to-br from-cyan-600 to-teal-700 text-white">
                    <h1 class="text-3xl font-semibold mb-1">"My Day"</h1>
                </header>
                <div class="py-2">
                    <Suspense fallback=move || view! {
                        <div class="px-6 py-6 text-center text-slate-400">"Loading tasks..."</div>
                    }>
                        <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors/> }>
                            {move || {
                                Suspend::new(async move {
                                    task_list.await.map(|task_list| {
                                        if task_list.is_empty() {
                                            Either::Left(view! { <p>"No tasks were found."</p> })
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
) -> impl IntoView {
    let id = *task.id();

    let (checked, set_checked) = signal(task.is_done());
    let complete_task = Action::new(move |(id, checked): &(_, _)| {
        let id = *id;
        let checked = *checked;
        async move {
            if let Err(e) = server::complete_task(id, checked).await {
                tracing::error!("complete task failed: {e}");
                set_checked.set(!checked);
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
                    class="w-5 h-5 rounded-full border-2 border-slate-600 cursor-pointer appearance-none mr-4 flex-shrink-0 transition-all checked:bg-gradient-to-br checked:from-cyan-500 checked:to-teal-600 checked:border-cyan-500 relative"
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
                    <span class=move || if checked.get() {
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
                            <div class="absolute inset-0 rounded-full bg-gradient-to-br from-cyan-500 to-teal-600 opacity-75 animate-ping"></div>
                            <div class="relative rounded-full bg-gradient-to-br from-cyan-500 to-teal-600 w-5 h-5 animate-spin border-2 border-slate-900 border-t-transparent"></div>
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
                                                    TaskPriority::A => "bg-red-700",
                                                    TaskPriority::B => "bg-amber-500",
                                                    TaskPriority::C => "bg-green-300",
                                                }
                                            )}>
                                                <span class="text-white text-xs font-bold">{priority.to_string()}</span>
                                            </div>
                                            <div class={format!(
                                                "text-xs font-semibold uppercase tracking-wide mb-0.5 {}",
                                                match priority {
                                                    TaskPriority::A => "text-red-500",
                                                    TaskPriority::B => "text-amber-500",
                                                    TaskPriority::C => "text-green-400",
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
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-500 border-4 border-slate-900 shadow flex items-center justify-center">
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
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-cyan-600 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-cyan-400 uppercase tracking-wide mb-0.5">"Start Date"</div>
                                            <div class="text-sm text-slate-200">{start_date}</div>
                                        </div>
                                    })}
                                    {time_estimate.map(|time_estimate| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-md bg-amber-500 border-4 border-slate-900 shadow flex items-center justify-center">
                                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                    <path d="M6 2a1 1 0 011-1h6a1 1 0 011 1v1a1 1 0 01-.293.707L12 5.414V7a1 1 0 01-.293.707L10 9.414l-1.707-1.707A1 1 0 018 7V5.414L6.293 3.707A1 1 0 016 3V2zm0 16a1 1 0 001 1h6a1 1 0 001-1v-1a1 1 0 00-.293-.707L12 14.586V13a1 1 0 00-.293-.707L10 10.586l-1.707 1.707A1 1 0 008 13v1.586l-1.707 1.707A1 1 0 006 17v1z"/>
                                                </svg>
                                            </div>
                                            <div class="text-xs font-semibold text-amber-500 uppercase tracking-wide mb-0.5">"Estimate"</div>
                                            <div class="text-sm text-slate-200">{time_estimate}</div>
                                        </div>
                                    })}
                                    {context.map(|context| view! {
                                        <div class="relative">
                                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-teal-500 border-4 border-slate-900 shadow flex items-center justify-center">
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
                                            <div class="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">"Notes"</div>
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
                    <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-cyan-500 border-4 border-slate-900 shadow flex items-center justify-center">
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
