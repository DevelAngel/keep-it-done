pub mod cache;
mod error_template;
pub mod server;

use crate::error_template::ErrorTemplate;

use kid_types::{TaskProperties, Uuid};

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
        <div class="min-h-screen bg-gradient-to-br from-indigo-500 to-purple-600">
            <div class="max-w-2xl mx-auto min-h-screen bg-white shadow-2xl">
                <header class="px-6 py-8 bg-gradient-to-br from-indigo-500 to-purple-600 text-white">
                    <h1 class="text-3xl font-semibold mb-1">"My Day"</h1>
                </header>
                <div class="py-2">
                    <Suspense fallback=move || view! {
                        <div class="px-6 py-6 text-center text-gray-500">"Loading tasks..."</div>
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
fn TaskItem<T: for<'a> TaskProperties<'a>>(
    task: T,
    expanded_task_id: ReadSignal<Option<Uuid>>,
    set_expanded_task_id: WriteSignal<Option<Uuid>>,
) -> impl IntoView {
    let (checked, set_checked) = signal(false);
    let complete_task = ServerMultiAction::<server::CompleteTask>::new();

    let id = *task.id();
    let created = task.created().to_relative_time();
    let summary = task.summary().to_string();

    // Mock additional properties until data model is extended
    let mock_priority = "A";
    let mock_estimate = "2 hours";
    let mock_context = "Kitchen";
    let mock_notes =
        Some("Contact references from neighbor Bob. Get at least 3 quotes for comparison.");

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
            class="border-b border-gray-100 transition-colors"
            class:bg-indigo-50=is_expanded
        >
            <div
                class="flex items-center px-6 py-4 hover:bg-gray-50 transition-colors cursor-pointer"
                on:click=handle_task_click
            >
                <input
                    type="checkbox"
                    class="w-5 h-5 rounded-full border-2 border-gray-300 cursor-pointer appearance-none mr-4 flex-shrink-0 transition-all checked:bg-gradient-to-br checked:from-indigo-500 checked:to-purple-600 checked:border-indigo-500 relative"
                    checked=move || checked.get()
                    on:change=move |_| set_checked.update(|c| *c = !*c)
                    on:click=|e| e.stop_propagation()  // Prevent row click when checking
                />
                <div class="flex-1">
                    <span class=move || if checked.get() {
                        "text-gray-900 line-through opacity-50"
                    } else {
                        "text-gray-900"
                    }>
                        {summary}
                    </span>
                </div>
            </div>

            // Expanded detail section (Timeline-Style)
            <Show when=is_expanded>
                <div class="px-6 pb-4 pt-3 bg-gradient-to-b from-white via-indigo-50 to-white">
                    // Vertical timeline with connecting line
                    <div class="relative pl-8 space-y-4">
                        // Vertical line
                        <div class="absolute left-3 top-0 bottom-0 w-0.5 bg-gradient-to-b from-indigo-300 via-purple-300 to-indigo-300"></div>

                        // Created
                        <div class="relative">
                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-indigo-500 border-4 border-white shadow flex items-center justify-center">
                                <div class="w-2 h-2 rounded-full bg-white"></div>
                            </div>
                            <div class="text-xs font-semibold text-indigo-600 uppercase tracking-wide mb-0.5">"Created"</div>
                            <div class="text-sm text-gray-900">{created.clone()}</div>
                        </div>

                        // Priority
                        <div class="relative">
                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-red-500 border-4 border-white shadow flex items-center justify-center">
                                <span class="text-white text-xs font-bold">{mock_priority}</span>
                            </div>
                            <div class="text-xs font-semibold text-red-600 uppercase tracking-wide mb-0.5">"Priority"</div>
                            <div class="text-sm text-gray-900">"High importance"</div>
                        </div>

                        // Time estimate
                        <div class="relative">
                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-blue-500 border-4 border-white shadow flex items-center justify-center">
                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <div class="text-xs font-semibold text-blue-600 uppercase tracking-wide mb-0.5">"Estimate"</div>
                            <div class="text-sm text-gray-900">{mock_estimate}</div>
                        </div>

                        // Context
                        <div class="relative">
                            <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-purple-500 border-4 border-white shadow flex items-center justify-center">
                                <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                                </svg>
                            </div>
                            <div class="text-xs font-semibold text-purple-600 uppercase tracking-wide mb-0.5">"Context"</div>
                            <div class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-purple-500 text-white shadow-sm">
                                {mock_context}
                            </div>
                        </div>

                        // Notes
                        {mock_notes.map(|notes| view! {
                            <div class="relative">
                                <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-gray-500 border-4 border-white shadow flex items-center justify-center">
                                    <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path d="M9 2a2 2 0 00-2 2v8a2 2 0 002 2h6a2 2 0 002-2V6.414A2 2 0 0016.414 5L14 2.586A2 2 0 0012.586 2H9z"/>
                                    </svg>
                                </div>
                                <div class="text-xs font-semibold text-gray-600 uppercase tracking-wide mb-1">"Notes"</div>
                                <div class="text-sm text-gray-700 leading-relaxed bg-white p-3 rounded-lg border border-gray-200 shadow-sm whitespace-pre-wrap">
                                    {notes}
                                </div>
                            </div>
                        })}
                    </div>
                </div>
            </Show>
        </div>
    }
}

trait ToRelativeTime {
    fn to_relative_time(&self) -> String;
}

impl ToRelativeTime for DateTime<Utc> {
    fn to_relative_time(&self) -> String {
        let duration = Utc::now().signed_duration_since(self);
        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else {
            let timestamp = self.with_timezone(&Local);
            timestamp.format("%x %T").to_string()
        }
    }
}
