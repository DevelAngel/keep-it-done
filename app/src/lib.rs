pub mod cache;
mod error_template;

use crate::error_template::ErrorTemplate;

use kid_types::{Task, TaskProperties, Uuid};

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
                                                    key=|task| task.id().clone()
                                                    children=move |task| {
                                                        view! { <TaskItem task=task/> }
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
fn TaskItem<T: for<'a> TaskProperties<'a>>(task: T) -> impl IntoView {
    let (checked, set_checked) = signal(false);
    let summary = task.summary().to_string();

    view! {
        <div class="flex items-center px-6 py-4 border-b border-gray-100 hover:bg-gray-50 transition-colors">
            <input
                type="checkbox"
                class="w-5 h-5 rounded-full border-2 border-gray-300 cursor-pointer appearance-none mr-4 flex-shrink-0 transition-all checked:bg-gradient-to-br checked:from-indigo-500 checked:to-purple-600 checked:border-indigo-500 relative"
                checked=move || checked.get()
                on:change=move |_| set_checked.update(|c| *c = !*c)
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
