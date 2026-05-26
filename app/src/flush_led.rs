use kid_types::Uuid;
use leptos::prelude::*;

use std::ops::Deref;

/// Whether the flush error panel is currently open.
///
/// Provided via context so the edit-mode bottom bar can adjust
/// its position (move to top edge of panel or hide).
#[derive(Clone, Copy, Default)]
pub struct FlushPanelOpen(RwSignal<bool>);

impl Deref for FlushPanelOpen {
    type Target = RwSignal<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Bumped whenever a `TaskChanged` SSE event arrives.
///
/// Used as a Resource dependency to trigger silent refetch (Layer 1).
#[derive(Clone, Copy, Default)]
pub struct TaskChangeVersion(pub RwSignal<u32>);

impl Deref for TaskChangeVersion {
    type Target = RwSignal<u32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Set of task IDs that were recently changed externally.
///
/// Drives the inline row highlight (Layer 2) and the conflict
/// banner in the detail panel (Layer 3).
#[derive(Clone, Copy, Default)]
pub struct RecentlyChangedIds(pub RwSignal<Vec<Uuid>>);

impl Deref for RecentlyChangedIds {
    type Target = RwSignal<Vec<Uuid>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Flush status LED state, derived from the latest SSE event.
///
/// Variants are constructed only in the `hydrate` (WASM) build
/// via EventSource callbacks.
#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) enum LedState {
    /// No flush event received yet, or success auto-dismissed.
    Hidden,
    /// Flush succeeded — show green, auto-dismiss after 3 s.
    Success,
    /// Flush failed — show red, persist until next success.
    Error { message: String },
}

#[component]
pub fn FlushStatusLed() -> impl IntoView {
    let led_state = RwSignal::new(LedState::Hidden);
    let panel_open = FlushPanelOpen::default();
    provide_context(panel_open);
    // Brief pulse signal: toggled on retry while panel is open.
    let pulse = RwSignal::new(false);
    let edit_mode = use_context::<crate::EditMode>().unwrap_or_default();

    // --- EventSource setup (WASM only) ---
    #[cfg(feature = "hydrate")]
    {
        use crate::events::{FlushOutcome, ServerEvent};
        use wasm_bindgen::prelude::*;
        use web_sys::{EventSource, MessageEvent};

        let task_change_version = use_context::<TaskChangeVersion>().unwrap_or_default();
        let recently_changed = use_context::<RecentlyChangedIds>().unwrap_or_default();

        Effect::new(move |_| {
            let Ok(es) = EventSource::new("/api/events") else {
                tracing::warn!("failed to create EventSource");
                return;
            };
            let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                let Some(data) = event.data().as_string() else {
                    return;
                };
                let Ok(server_event) = serde_json::from_str::<ServerEvent>(&data) else {
                    return;
                };
                match server_event {
                    ServerEvent::Flush(outcome) => match outcome {
                        FlushOutcome::Success { .. } => {
                            // If error panel was open, close it.
                            panel_open.set(false);
                            led_state.set(LedState::Success);
                            // Auto-dismiss after 3 s.
                            set_timeout(
                                move || {
                                    if matches!(led_state.get_untracked(), LedState::Success) {
                                        led_state.set(LedState::Hidden);
                                    }
                                },
                                std::time::Duration::from_secs(3),
                            );
                        }
                        FlushOutcome::Error { message } => {
                            // If panel is open, pulse the LED briefly.
                            if panel_open.get_untracked() {
                                pulse.set(true);
                                set_timeout(
                                    move || pulse.set(false),
                                    std::time::Duration::from_millis(300),
                                );
                            }
                            led_state.set(LedState::Error { message });
                        }
                    },
                    ServerEvent::TaskChanged { id, .. } => {
                        // Layer 1: bump version → triggers Resource refetch.
                        task_change_version.update(|v| *v = v.wrapping_add(1));
                        // Layer 2 + 3: record changed ID for highlight / conflict.
                        recently_changed.update(|ids| {
                            if !ids.contains(&id) {
                                ids.push(id);
                            }
                        });
                        // Layer 2: auto-clear highlight after 4 s.
                        set_timeout(
                            move || {
                                recently_changed.update(|ids| ids.retain(|i| *i != id));
                            },
                            std::time::Duration::from_secs(4),
                        );
                    }
                }
            });
            es.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        });
    }

    let is_error = move || matches!(led_state.get(), LedState::Error { .. });

    let on_led_click = move |_| {
        if is_error() {
            panel_open.update(|open| *open = !*open);
        }
    };

    let led_class = move || {
        let base = "fixed bottom-4 right-4 z-50 w-2.5 h-2.5 rounded-full transition-opacity duration-200";
        match led_state.get() {
            LedState::Hidden => format!("{base} opacity-0 pointer-events-none"),
            LedState::Success => format!("{base} bg-green-500 pointer-events-none"),
            LedState::Error { .. } => {
                let dimmed = panel_open.get() && !pulse.get();
                if dimmed {
                    format!("{base} bg-red-500 cursor-pointer opacity-20")
                } else {
                    format!("{base} bg-red-500 cursor-pointer")
                }
            }
        }
    };

    let error_message = move || match led_state.get() {
        LedState::Error { message } => Some(message),
        _ => None,
    };

    let panel_border = move || {
        if edit_mode.get() {
            "border-t-[3px] border-amber-400"
        } else {
            "border-t border-slate-700"
        }
    };

    let led_testid = move || match led_state.get() {
        LedState::Hidden => "flush-led-hidden",
        LedState::Success => "flush-led-ok",
        LedState::Error { .. } => "flush-led-err",
    };

    view! {
        <div
            class=led_class
            role="status"
            aria-live="polite"
            on:click=on_led_click
            data-testid=led_testid
        >
            <span class="sr-only">
                {move || match led_state.get() {
                    LedState::Hidden => String::new(),
                    LedState::Success => "Tasks saved".to_string(),
                    LedState::Error { .. } => "Flush error \u{2014} tap for details".to_string(),
                }}
            </span>
        </div>
        <Show when=move || panel_open.get() && error_message().is_some()>
            <div
                class=move || format!(
                    "fixed bottom-0 left-0 right-0 z-50 bg-slate-900 px-4 py-3 text-sm text-red-300 {border}",
                    border = panel_border(),
                )
                role="alert"
                data-testid="flush-error-panel"
            >
                <div class="mx-auto max-w-xl">
                    "\u{26A0}\u{FE0F} Flush error: "
                    {error_message}
                </div>
            </div>
        </Show>
    }
}
