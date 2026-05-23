use leptos::either::Either;
use leptos::prelude::*;

/// Check whether any error looks like an auth-proxy 401
/// (Tinyauth returns `{"message":"Unauthorized","status":401}`
/// which Leptos cannot deserialize as a server-function result).
fn is_unauthorized(errors: &[leptos::prelude::Error]) -> bool {
    errors.iter().any(|e| {
        let msg = e.to_string();
        msg.contains("Unauthorized") || msg.contains("\"status\":401")
    })
}

#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional, into)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };

    let errors = move || errors.get().into_iter().map(|(_, v)| v).collect::<Vec<_>>();

    move || {
        let errs = errors();
        if is_unauthorized(&errs) {
            Either::Left(view! {
                <div class="px-6 py-12 text-center">
                    <p class="text-lg text-slate-300 mb-4">"Sitzung abgelaufen — bitte neu anmelden."</p>
                    <a href="/" class="text-sky-400 underline hover:text-sky-300">"Neu laden"</a>
                </div>
            })
        } else {
            Either::Right(view! {
                <div class="px-6 py-12 text-center">
                    <h1 class="text-lg font-semibold text-red-400 mb-4">"Fehler"</h1>
                    {errs
                        .into_iter()
                        .map(|error| {
                            view! { <p class="text-slate-300 text-sm mb-2">{error.to_string()}</p> }
                        })
                        .collect::<Vec<_>>()}
                </div>
            })
        }
    }
}
