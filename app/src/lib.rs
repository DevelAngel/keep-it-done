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
        task_cache.into_vec()
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
    let task_list = move || {
        Suspend::new(async move {
            task_list.await.map(|task_list| {
                if task_list.is_empty() {
                    Either::Left(view! { <p>"No tasks were found."</p> })
                } else {
                    Either::Right(
                        task_list
                            .iter()
                            .map(move |task: &Task| {
                                let summary = task.summary().to_string();
                                let id = task.id().as_simple().to_string();
                                view! {
                                    <li>
                                        {summary} - {id.clone()}
                                        <ActionForm action=delete_task>
                                            <input type="hidden" name="id" value=id/>
                                            <input type="submit" value="X"/>
                                        </ActionForm>
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            })
        })
    };

    view! {
        <MultiActionForm action=add_task>
            <label>"Add a Task" <input type="text" name="summary"/></label>
            <input type="submit" value="Add"/>
        </MultiActionForm>
        <div>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| view! { <ErrorTemplate errors/> }>
                    <ul>
                        {task_list}
                        {move || {
                            add_task.submissions()
                                .get()
                                .into_iter()
                                .filter(|submission| submission.pending().get())
                                .map(|submission| {
                                    view! {
                                        <li class="pending">
                                            {move || submission.input().get().map(|data| data.summary)}
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}

                    </ul>
                </ErrorBoundary>
            </Transition>
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
