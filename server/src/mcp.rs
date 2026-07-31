//! MCP (Model Context Protocol) server exposing task operations to AI assistants.
//! See `docs/adr/rmcp-mcp-server.md` for the design decisions
//! behind the tool/resource split, the assistant-identity handling,
//! and the separate-port transport choice.

use crate::oauth::{self, McpOAuthStore};
use crate::{SharedTaskCache, SharedTimeOffset};

use kid_app::{DeadlineGroup, server::group_quick_wins, server::group_upcoming};
use kid_types::task::Details as TaskDetails;
use kid_types::task::DetailsPatch as TaskDetailsPatch;
use kid_types::{
    Task, TaskCategory, TaskContext, TaskInfos, TaskPriority, TaskSummary, TaskTimeEstimate, Uuid,
};

use chrono::{Datelike, Weekday};

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{
    CacheScope, ContentBlock, ErrorData as McpError, Implementation, ListResourcesResult,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult,
    Resource, ResourceContents, ResultType, ServerCapabilities, ServerInfo,
};
use rmcp::schemars::{self, JsonSchema};
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{RoleServer, ServerHandler, model::CallToolResult, tool, tool_handler, tool_router};

use indexmap::IndexSet;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use url::Url;

use std::collections::BTreeSet;
use std::sync::Arc;

/// Standalone MCP server, kept on its own port.
/// See `docs/adr/rmcp-mcp-server.md` — "Transport and port".
pub struct McpServer;

/// MCP server exposing task tools and read-only reference resources.
#[derive(Clone)]
pub struct McpService {
    task_cache: SharedTaskCache,
    time_offset: SharedTimeOffset,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListInput {
    /// Fuzzy search query, matched against summary/category/contexts
    #[serde(default)]
    search: Option<String>,
    /// Filter by status (default: open)
    #[serde(default)]
    status: StatusFilter,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ListOutput {
    entries: Vec<TaskEntry>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum StatusFilter {
    #[default]
    Open,
    Done,
    All,
}

#[derive(Debug, Serialize, JsonSchema)]
struct TaskEntry {
    id: Uuid,
    task: Task,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddInput {
    /// Short task title
    summary: String,
    /// Life-domain category, e.g. "Household", "Finance" — see kid://categories
    category: String,
    /// GTD-style contexts, each starting with '@' — see kid://contexts
    #[serde(default)]
    contexts: Vec<String>,
    /// Optional task details (due_date, start_date, time_estimate, notes, availability)
    #[serde(default)]
    details: Option<TaskDetails>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RenameInput {
    id: Uuid,
    /// New summary — this is the task's identity, use `update` for other changes
    summary: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaceInput {
    id: Uuid,
    /// Full replacement details — omitted fields become unset
    details: TaskDetails,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateInput {
    id: Uuid,
    /// Patch — omitted fields stay unchanged, explicit null clears a field
    details: TaskDetailsPatch,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CompleteInput {
    id: Uuid,
    /// Reopen the task instead of completing it
    #[serde(default)]
    reopen: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RecategorizeInput {
    id: Uuid,
    category: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContextsInput {
    id: Uuid,
    /// Contexts, each starting with '@'
    #[serde(default)]
    contexts: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetPriorityInput {
    id: Uuid,
    /// "A", "B", or "C" — omit or null to clear
    #[serde(default)]
    priority: Option<String>,
}

impl McpServer {
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
        time_offset: SharedTimeOffset,
        base_url: Url,
        allowed_origins: Vec<Url>,
        clients: crate::oauth::McpClientsConfig,
    ) -> Result<()> {
        tracing::info!(
            "MCP server listening on: http://{}",
            listener.local_addr().unwrap()
        );

        // The server's own base URL must be allowed too, since it issues
        // requests to itself (e.g. OAuth metadata, self-registration).
        let all_origins: Vec<Url> = std::iter::once(base_url.clone())
            .chain(allowed_origins)
            .collect();
        let allowed_hosts: Vec<String> = all_origins.iter().filter_map(host_header).collect();
        let allowed_origins: Vec<String> = all_origins
            .iter()
            .map(|url| url.origin().ascii_serialization())
            .collect();

        let mcp_service = StreamableHttpService::new(
            move || Ok(McpService::new(task_cache.clone(), time_offset.clone())),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_allowed_origins(allowed_origins)
                .with_allowed_hosts(allowed_hosts)
                .with_cancellation_token(shutdown.child_token()),
        );

        let oauth_store = Arc::new(McpOAuthStore::new(base_url, clients));
        tokio::spawn({
            let oauth_store = oauth_store.clone();
            let shutdown = shutdown.child_token();
            async move { oauth_store.background_cleanup(shutdown).await }
        });

        let protected_mcp_router =
            Router::new()
                .nest_service("/mcp", mcp_service)
                .layer(middleware::from_fn_with_state(
                    oauth_store.clone(),
                    oauth::validate_access_token,
                ));

        let oauth_server_router = Router::new()
            .route(
                "/.well-known/oauth-authorization-server",
                get(oauth::auth_server).options(oauth::auth_server),
            )
            .route(
                "/.well-known/oauth-protected-resource",
                get(oauth::protected_resource).options(oauth::protected_resource),
            )
            .route(
                "/.well-known/oauth-protected-resource/mcp",
                get(oauth::protected_resource).options(oauth::protected_resource),
            )
            .route("/authorize", get(oauth::authorize))
            .route("/oauth/approve", post(oauth::approve))
            .route(
                "/token",
                post(oauth::gen_access_token).options(oauth::gen_access_token),
            )
            .with_state(oauth_store.clone());

        // In deployment, static assets live under LEPTOS_SITE_ROOT; for
        // local dev (where that's unset) public/ is a sibling of server/
        // (this crate's manifest dir) in the workspace.
        let favicons_dir = std::env::var("LEPTOS_SITE_ROOT")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../public").to_owned());
        let favicons = ServeDir::new(favicons_dir);

        let app = Router::new()
            .merge(protected_mcp_router)
            .merge(oauth_server_router)
            .fallback_service(favicons)
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
                tracing::info!("MCP server shutting down");
            })
            .await
            .into_diagnostic()?;
        Ok(())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("kid-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Family task manager. Every task needs a category and should \
             have at least one context (GTD-style, starting with '@'). \
             Fetch kid://categories and kid://contexts to reuse existing \
             values before inventing new ones.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        const TTL_MS: u64 = 10 * 60 * 1000;
        Ok(ListResourcesResult {
            resources: vec![
                Resource::new(self::categories::URI, self::categories::NAME)
                    .with_description("Categories currently in use")
                    .with_mime_type("application/json"),
                Resource::new(self::contexts::URI, self::contexts::NAME)
                    .with_description("Contexts currently in use")
                    .with_mime_type("application/json"),
                Resource::new(self::daily_report::URI, self::daily_report::NAME)
                    .with_description(
                        "Markdown summary of open tasks for today: grouped into \
                         Overdue, Today, Tomorrow, This Week, Next Week, Later, and \
                         Ready to Start. Use this to tell a human what's on their \
                         plate.",
                    )
                    .with_mime_type("text/markdown"),
                Resource::new(self::backlog::URI, self::backlog::NAME)
                    .with_description(
                        "Markdown list of open tasks that have no due date and \
                         aren't ready to start yet.",
                    )
                    .with_mime_type("text/markdown"),
                Resource::new(self::quick_wins::URI, self::quick_wins::NAME)
                    .with_description(
                        "Markdown list of open tasks that have a time estimate, \
                         grouped from shortest to longest. Use this to suggest \
                         something to knock out with whatever time is available.",
                    )
                    .with_mime_type("text/markdown"),
            ],
            next_cursor: None,
            meta: None,
            // since 2026-07-28 spec
            result_type: Some(ResultType::COMPLETE),
            ttl_ms: Some(TTL_MS),
            cache_scope: Some(CacheScope::Private),
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri, .. }: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let result = match uri.as_str() {
            self::categories::URI => {
                let cache = self.task_cache.read().await;
                let categories: BTreeSet<TaskCategory> = cache
                    .iter()
                    .map(|(_, task)| task.category().parse::<TaskCategory>().unwrap())
                    .collect();
                let json = serde_json::to_string(&categories).unwrap_or_default();
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(json, &uri).with_mime_type("application/json"),
                ]))
            }
            self::contexts::URI => {
                let cache = self.task_cache.read().await;
                let contexts: BTreeSet<TaskContext> = cache
                    .iter()
                    .flat_map(|(_, task)| task.info().contexts().iter().cloned())
                    .collect();
                let json = serde_json::to_string(&contexts).unwrap_or_default();
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(json, &uri).with_mime_type("application/json"),
                ]))
            }
            self::daily_report::URI => {
                let cache = self.task_cache.read().await;
                let today = kid_app::time::today_at_offset(self.time_offset.get());
                let (groups, _backlog) = group_upcoming(cache.iter(), today);
                let markdown = render_daily_report(groups);
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            self::backlog::URI => {
                let cache = self.task_cache.read().await;
                let today = kid_app::time::today_at_offset(self.time_offset.get());
                let (_groups, backlog) = group_upcoming(cache.iter(), today);
                let markdown = render_backlog(backlog);
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            self::quick_wins::URI => {
                let cache = self.task_cache.read().await;
                let groups = group_quick_wins(cache.iter());
                let markdown = render_quick_wins(groups);
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            _ => Err(McpError::resource_not_found(
                format!("unknown resource: {uri}"),
                None,
            )),
        }?;
        Ok(ReadResourceResponse::Complete(result))
    }
}

#[tool_router]
impl McpService {
    #[tool(description = "List tasks, optionally filtered by status and/or fuzzy search")]
    async fn list(
        &self,
        Parameters(ListInput { search, status }): Parameters<ListInput>,
    ) -> Json<ListOutput> {
        let cache = self.task_cache.read().await;
        let entries: Vec<_> = cache
            .iter()
            .filter(|(_, task)| match status {
                StatusFilter::All => true,
                StatusFilter::Open => !task.is_done(),
                StatusFilter::Done => task.is_done(),
            })
            .collect();

        let entries: Vec<_> = match search.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
            Some(query) => {
                let words: Vec<&str> = query.split_whitespace().collect();
                entries
                    .into_iter()
                    .filter(|(_, task)| {
                        let haystack = format!(
                            "{} {} {}",
                            task.summary(),
                            task.category(),
                            task.contexts()
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                        words
                            .iter()
                            .all(|w| sublime_fuzzy::best_match(w, &haystack).is_some())
                    })
                    .collect()
            }
            None => entries,
        };

        let entries = entries
            .into_iter()
            .map(|(id, task)| TaskEntry {
                id: *id,
                task: task.clone(),
            })
            .collect();

        Json(ListOutput { entries })
    }

    #[tool(description = "Add a new task. Returns the new task's id")]
    async fn add(
        &self,
        Parameters(AddInput {
            summary,
            category,
            contexts,
            details,
        }): Parameters<AddInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let summary: TaskSummary = summary.parse().map_err(parse_err)?;
        let category: TaskCategory = category.parse().map_err(parse_err)?;
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let mut task = Task::new(summary)
            .with_category(category)
            .with_contexts(contexts);
        if let Some(details) = details {
            task = task.with_details(details);
        }
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        let id = cache.add(task, actor);
        Ok(CallToolResult::structured(serde_json::json!({ "id": id })))
    }

    #[tool(description = "Rename a task's summary — its identity. Use `update` for other changes")]
    async fn rename(
        &self,
        Parameters(RenameInput { id, summary }): Parameters<RenameInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let summary: TaskSummary = summary.parse().map_err(parse_err)?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.rename(summary);
                Ok(CallToolResult::success(vec![ContentBlock::text("renamed")]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Replace all task details (PUT semantics — omitted fields cleared)")]
    async fn replace(
        &self,
        Parameters(ReplaceInput { id, details }): Parameters<ReplaceInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_details(details);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "replaced",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Patch task details (PATCH semantics — omitted fields unchanged)")]
    async fn update(
        &self,
        Parameters(UpdateInput { id, details }): Parameters<UpdateInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.patch_details(details);
                Ok(CallToolResult::success(vec![ContentBlock::text("updated")]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Complete a task, or reopen it with reopen=true")]
    async fn complete(
        &self,
        Parameters(CompleteInput { id, reopen }): Parameters<CompleteInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                if reopen {
                    task.mark_todo();
                } else {
                    task.mark_done();
                }
                let msg = if reopen { "reopened" } else { "completed" };
                Ok(CallToolResult::success(vec![ContentBlock::text(msg)]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Change a task's category")]
    async fn recategorize(
        &self,
        Parameters(RecategorizeInput { id, category }): Parameters<RecategorizeInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let category: TaskCategory = category.parse().map_err(parse_err)?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_category(category);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "recategorized",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Add contexts to a task, keeping existing ones")]
    async fn add_contexts(
        &self,
        Parameters(ContextsInput { id, contexts }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.extend_contexts(contexts);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "contexts added",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Replace all of a task's contexts (empty list clears them)")]
    async fn replace_contexts(
        &self,
        Parameters(ContextsInput { id, contexts }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_contexts(contexts);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "contexts replaced",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(description = "Set or clear a task's priority")]
    async fn set_priority(
        &self,
        Parameters(SetPriorityInput { id, priority }): Parameters<SetPriorityInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let priority: Option<TaskPriority> =
            priority.map(|s| s.parse().map_err(parse_err)).transpose()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                match priority {
                    Some(p) => task.set_priority(p),
                    None => task.clear_priority(),
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "priority set",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }
}

impl McpService {
    pub fn new(task_cache: SharedTaskCache, time_offset: SharedTimeOffset) -> Self {
        Self {
            task_cache,
            time_offset,
            tool_router: Self::tool_router(),
        }
    }

    /// Derives the actor string `<client_id>:<on_behalf_of>` entirely from
    /// the `--mcp-clients-file` entry of the OAuth client that authenticated
    /// this request - neither piece is claimed by the client itself.
    ///
    /// The client id is not the MCP handshake's self-reported
    /// `clientInfo.name`, since that's freely claimed by the client and
    /// thus unverified. It carries whatever prefix distinguishes it (e.g.
    /// `ai:claude-desktop` vs. `matrix-relay`), since not every client is an
    /// AI assistant. Likewise, `on_behalf_of` is fixed per client in that
    /// same file, not a tool parameter - a client claiming this for itself
    /// couldn't be trusted any more than it claiming its own identity.
    ///
    /// A client whose entry omits `on-behalf-of` is read-only: there's no
    /// human to attribute a change to, so this returns an error instead of
    /// making one up.
    fn actor(context: &RequestContext<RoleServer>) -> Result<String, McpError> {
        let extensions = context
            .extensions
            .get::<axum::http::request::Parts>()
            .map(|parts| &parts.extensions);
        let client_id = extensions
            .and_then(|ext| ext.get::<oauth::ClientId>())
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                tracing::warn!("no authenticated client id in request extensions");
                "unknown".to_string()
            });
        let prefix = extensions
            .and_then(|ext| ext.get::<oauth::Prefix>())
            .map(|prefix| format!("{prefix}:"))
            .unwrap_or_default();
        let Some(on_behalf_of) = extensions.and_then(|ext| ext.get::<oauth::OnBehalfOf>()) else {
            return Err(McpError::invalid_request(
                format!("client {prefix}{client_id} is read-only (no on-behalf-of configured)"),
                None,
            ));
        };
        Ok(format!("{prefix}{client_id}:{on_behalf_of}"))
    }

    fn not_found(id: Uuid) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(format!("task {id} not found"))])
    }
}

fn parse_err(e: &str) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

pub(super) mod categories {
    pub const NAME: &str = "Categories";
    pub const URI: &str = "kid://categories";
}
pub(super) mod contexts {
    pub const NAME: &str = "Contexts";
    pub const URI: &str = "kid://contexts";
}
pub(super) mod daily_report {
    pub const NAME: &str = "Daily Report";
    pub const URI: &str = "kid://report/daily";
}
pub(super) mod backlog {
    pub const NAME: &str = "Backlog";
    pub const URI: &str = "kid://report/backlog";
}
pub(super) mod quick_wins {
    pub const NAME: &str = "Quick Wins";
    pub const URI: &str = "kid://report/quick_wins";
}

/// Picks one of `sentences` at random.
fn random_intro(sentences: &'static [&'static str]) -> &'static str {
    use rand::prelude::IndexedRandom;
    sentences.choose(&mut rand::rng()).unwrap()
}

const DAILY_REPORT_INTROS: [&str; 10] = [
    "Hey! Listen! 🧚 New quests have appeared - time to set out!",
    "Wake up, young hero. 🧚 The tasks of this day await your courage. 🌳",
    "Hey! Hey! 🧚 Today's trials are ready for you!",
    "The path ahead is clear, adventurer. 🧙 Let's clear these quests! 🗡️",
    "Fairy's honor: today's objectives won't complete themselves. Onward! ✨",
    "Hey! 🧚 Over here! Your quest log has refreshed for today!",
    "It is time, hero. 🧝 Destiny - or at least today's to-do list - awaits.",
    "Hyah! 🐲 A new day dawns over Hyrule, and with it, new quests. ☀️",
    "The Great Deku Tree has watched over these tasks. Now they're yours. 🌳",
    "Hey! Listen! Don't let the day slip by like a Skulltula in the dark. ✨",
];

const BACKLOG_INTROS: [&str; 10] = [
    "Deep in the quest log, these tasks slumber - no deadline binds them yet. 🌳",
    "Side quests, patiently waiting in the shadow of the Great Tree. 🍃",
    "Hey! 🧚 These ones don't have a due date, but they haven't been forgotten!",
    "The undated scrolls of your journey rest here, hero. 🧙",
    "No urgency, no timer - just quests waiting for a worthy moment. 🧙",
    "Hey! Listen! 🧚 These are the quests that time forgot - for now.",
    "The Great Deku Tree has seen many ages pass, and these tasks with them. 🌳",
    "A hero's journey is long. 🧝 These quests will keep for when you're ready.",
    "Hey! 🧚 No rush on these ones - the forest keeps its secrets patiently. 🍃",
    "Old quests, older courage. 🧙 They'll be here when you return. ✨",
];

const QUICK_WINS_INTROS: [&str; 10] = [
    "Hey! Listen! 🧚 These quests are quick - a true hero clears them in a flash! ✨",
    "Small trials, swiftly won. Even a Kokiri could finish these. 🍃",
    "Hey! 🧚 No need for the Master Sword here - just a few minutes!",
    "Short quests, sorted by how fast courage can conquer them. 🗡️",
    "The Great Deku Tree smiles upon these easy victories. 🌳",
    "Hey! Listen! 🧚 Quick as a Deku Nut - these won't take long!",
    "Small deeds, hero, but every Triforce piece starts somewhere. ✨",
    "Hey! 🧚 Over here! Easy quests, ripe for the taking!",
    "Not every hero's journey needs an Epona 🐎 - these are a short walk. 🍃",
    "Fast quests for a fast hero. Go get 'em! 🗡️",
];

/// Renders `groups` (the dated-or-ready part of [`group_upcoming`]'s result)
/// as Markdown: one heading per [`DeadlineGroup`], with the same "This/Next
/// Weekend" relabeling and weekday/weekend divider as the Upcoming view when
/// every task in a `ThisWeek`/`NextWeek` group falls on a weekend.
fn render_daily_report(groups: kid_app::server::UpcomingGroups) -> String {
    if groups.is_empty() {
        return format!("*Daily Report*\n\nNo open tasks due, or ready to start, today.\n");
    }

    let mut markdown = format!("*Daily Report*\n\n{}\n", random_intro(&DAILY_REPORT_INTROS));

    for (group, tasks) in &groups {
        let all_weekend = matches!(group, DeadlineGroup::ThisWeek | DeadlineGroup::NextWeek)
            && tasks
                .iter()
                .all(|(_, _, _, date)| matches!(date.weekday(), Weekday::Sat | Weekday::Sun));
        let separator_idx = if matches!(group, DeadlineGroup::ThisWeek | DeadlineGroup::NextWeek) {
            tasks
                .iter()
                .position(|(_, _, _, date)| matches!(date.weekday(), Weekday::Sat | Weekday::Sun))
                .filter(|&idx| idx > 0)
        } else {
            None
        };

        markdown.push_str(&format!("\n*{}*\n\n", group.display_label(all_weekend)));
        for (idx, (_, info, ..)) in tasks.iter().enumerate() {
            if separator_idx == Some(idx) {
                markdown.push_str("---\n\n");
            }
            markdown.push_str(&task_line(info));
        }
    }

    markdown
}

/// Renders `backlog` (the dateless part of [`group_upcoming`]'s result) as
/// Markdown.
fn render_backlog(backlog: kid_app::server::UpcomingBacklog) -> String {
    let mut markdown = format!("*Backlog*\n\n{}\n\n", random_intro(&BACKLOG_INTROS));

    if backlog.is_empty() {
        markdown.push_str("No open tasks without a date.\n");
        return markdown;
    }

    for (_, info) in &backlog {
        markdown.push_str(&task_line(info));
    }

    markdown
}

/// Renders `groups` ([`group_quick_wins`]'s result) as Markdown: one
/// heading per time estimate, shortest first.
fn render_quick_wins(
    groups: Vec<(TaskTimeEstimate, Vec<(Uuid, kid_types::task::Infos)>)>,
) -> String {
    let mut markdown = format!("*Quick Wins*\n\n{}\n", random_intro(&QUICK_WINS_INTROS));

    if groups.is_empty() {
        markdown.push_str("\nNo open tasks have a time estimate.\n");
        return markdown;
    }

    for (estimate, tasks) in &groups {
        markdown.push_str(&format!("\n*{estimate}*\n\n"));
        for (_, info) in tasks {
            markdown.push_str(&task_line(info));
        }
    }

    markdown
}

/// Renders one task as a Markdown checkbox line, e.g.
/// `- [ ] Buy groceries _Household_`. Always unchecked: `group_upcoming`
/// only ever returns open tasks.
fn task_line(info: &kid_types::task::Infos) -> String {
    let summary = info.summary();
    let category = info.category();
    let category = if category.is_empty() {
        "⚠ no category"
    } else {
        category
    };
    if info.is_done() {
        format!("- ☑ {summary} _{category}_\n")
    } else {
        format!("- ☐ {summary} _{category}_\n")
    }
}

/// Derives a `Host` header value (`host` or `host:port`) from a configured
/// allowed origin, for `StreamableHttpServerConfig::with_allowed_hosts`.
fn host_header(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}
