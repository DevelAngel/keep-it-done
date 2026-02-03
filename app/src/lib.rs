pub mod cache;
mod error_template;

use crate::error_template::ErrorTemplate;

use kid_types::{Task, TaskProperties, Uuid};

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

#[cfg(feature = "ssr")]
pub mod ssr {
    use kid_types::server::{TaskCache, TaskList};
    use kid_types::{Task, Uuid};

    use leptos::context::use_context;
    use tokio::sync::RwLock;

    use std::sync::Arc;

    pub type SharedTaskCache = Arc<RwLock<TaskCache>>;

    pub(crate) async fn fetch_task_list() -> Vec<Task> {
        tracing::info!("fetch task list");
        let Some(task_cache) = use_context::<SharedTaskCache>() else {
            unreachable!("task cache missing")
        };
        let task_cache = task_cache.read().await;
        task_cache.to_vec()
    }

    pub(crate) async fn add_task(task: Task) -> bool {
        tracing::info!("add task {task:?}");
        let Some(task_cache) = use_context::<SharedTaskCache>() else {
            unreachable!("task cache missing")
        };
        let mut task_cache = task_cache.write().await;
        task_cache.add(task)
    }

    pub(crate) async fn delete_task(id: Uuid) -> bool {
        tracing::info!("delete task with id {id}");
        let Some(task_cache) = use_context::<SharedTaskCache>() else {
            unreachable!("task cache missing")
        };
        let mut task_cache = task_cache.write().await;
        task_cache.remove(id)
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

/// Renders the home page of your application.
#[component]
fn TaskList() -> impl IntoView {
    let (expanded_task_id, set_expanded_task_id) = signal(None::<Uuid>);

    let add_task = ServerMultiAction::<AddTask>::new();
    let delete_task = ServerAction::<DeleteTask>::new();

    let task_list = Resource::new(
        move || {
            (
                delete_task.version().get(),
                add_task.version().get(),
                delete_task.version().get(),
            )
        },
        move |_| fetch_task_list(),
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

            // Expanded detail section
            <Show when=is_expanded>
                <div class="px-6 pb-4 pt-2 space-y-2 text-sm">
                    // Created timestamp
                    <div class="text-gray-600">
                        <span class="font-medium">"Created: "</span>
                        <span>{created.clone()}</span>
                    </div>
                    // Priority
                    <div class="text-gray-600">
                        <span class="font-medium">"Priority: "</span>
                        <span class="inline-flex items-center justify-center w-6 h-6 rounded-full bg-gradient-to-br from-red-500 to-orange-500 text-white text-xs font-bold">
                            {mock_priority}
                        </span>
                    </div>
                    // Time estimate
                    <div class="text-gray-600">
                        <span class="font-medium">"Time estimate: "</span>
                        <span>{mock_estimate}</span>
                    </div>
                    // Context/Category
                    <div class="text-gray-600">
                        <span class="font-medium">"Context: "</span>
                        <span>{mock_context}</span>
                    </div>
                    // Notes (if present)
                    {mock_notes.map(|notes| view! {
                        <div class="text-gray-600 pt-2 border-t border-gray-200">
                            <div class="font-medium mb-1">"Notes:"</div>
                            <div class="text-gray-700 whitespace-pre-wrap">{notes}</div>
                        </div>
                    })}
                </div>
            </Show>
        </div>
    }
}

#[server]
pub async fn fetch_task_list() -> Result<Vec<Task>, ServerFnError> {
    let list = ssr::fetch_task_list().await;
    Ok(list)
}

#[server]
#[allow(unused_variables)]
pub async fn add_task(summary: String) -> Result<(), ServerFnError> {
    let task = Task::new(summary);
    let _added = ssr::add_task(task).await;
    assert!(_added);
    Ok(())
}

#[server]
pub async fn delete_task(id: Uuid) -> Result<(), ServerFnError> {
    let _deleted = ssr::delete_task(id).await;
    assert!(_deleted);
    Ok(())
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
