//! MCP (Model Context Protocol) server exposing task operations to AI assistants.
//! See `docs/adr/rmcp-mcp-server.md` for the design decisions
//! behind the tool/resource split, the assistant-identity handling,
//! and the separate-port transport choice.

use crate::SharedTaskCache;
use crate::oauth::{self, McpOAuthStore};

use kid_types::task::Details as TaskDetails;
use kid_types::task::DetailsPatch as TaskDetailsPatch;
use kid_types::{Task, TaskCategory, TaskContext, TaskInfos, TaskPriority, TaskSummary, Uuid};

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{
    ContentBlock, ErrorData as McpError, Implementation, ListResourcesResult,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerInfo,
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
    /// Human on whose behalf this task is created
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RenameInput {
    id: Uuid,
    /// New summary — this is the task's identity, use `update` for other changes
    summary: String,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaceInput {
    id: Uuid,
    /// Full replacement details — omitted fields become unset
    details: TaskDetails,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateInput {
    id: Uuid,
    /// Patch — omitted fields stay unchanged, explicit null clears a field
    details: TaskDetailsPatch,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CompleteInput {
    id: Uuid,
    /// Reopen the task instead of completing it
    #[serde(default)]
    reopen: bool,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RecategorizeInput {
    id: Uuid,
    category: String,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContextsInput {
    id: Uuid,
    /// Contexts, each starting with '@'
    #[serde(default)]
    contexts: Vec<String>,
    on_behalf_of: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetPriorityInput {
    id: Uuid,
    /// "A", "B", or "C" — omit or null to clear
    #[serde(default)]
    priority: Option<String>,
    on_behalf_of: String,
}

impl McpServer {
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
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
            move || Ok(McpService::new(task_cache.clone())),
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

        let app = Router::new()
            .merge(protected_mcp_router)
            .merge(oauth_server_router)
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
             values before inventing new ones. Mutating tools take \
             `on_behalf_of` (the human who requested the change); the \
             assistant's own name is derived from this MCP session, not \
             a tool parameter.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                Resource::new(self::categories::URI, self::categories::NAME)
                    .with_description("Categories currently in use")
                    .with_mime_type("application/json"),
                Resource::new(self::contexts::URI, self::contexts::NAME)
                    .with_description("Contexts currently in use")
                    .with_mime_type("application/json"),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri, .. }: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
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
            _ => Err(McpError::resource_not_found(
                format!("unknown resource: {uri}"),
                None,
            )),
        }
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
            on_behalf_of,
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
        let actor = Self::actor(&context, &on_behalf_of);
        let mut cache = self.task_cache.write().await;
        let id = cache.add(task, actor);
        Ok(CallToolResult::structured(serde_json::json!({ "id": id })))
    }

    #[tool(description = "Rename a task's summary — its identity. Use `update` for other changes")]
    async fn rename(
        &self,
        Parameters(RenameInput {
            id,
            summary,
            on_behalf_of,
        }): Parameters<RenameInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let summary: TaskSummary = summary.parse().map_err(parse_err)?;
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(ReplaceInput {
            id,
            details,
            on_behalf_of,
        }): Parameters<ReplaceInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(UpdateInput {
            id,
            details,
            on_behalf_of,
        }): Parameters<UpdateInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(CompleteInput {
            id,
            reopen,
            on_behalf_of,
        }): Parameters<CompleteInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(RecategorizeInput {
            id,
            category,
            on_behalf_of,
        }): Parameters<RecategorizeInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let category: TaskCategory = category.parse().map_err(parse_err)?;
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(ContextsInput {
            id,
            contexts,
            on_behalf_of,
        }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(ContextsInput {
            id,
            contexts,
            on_behalf_of,
        }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context, &on_behalf_of);
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
        Parameters(SetPriorityInput {
            id,
            priority,
            on_behalf_of,
        }): Parameters<SetPriorityInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let priority: Option<TaskPriority> =
            priority.map(|s| s.parse().map_err(parse_err)).transpose()?;
        let actor = Self::actor(&context, &on_behalf_of);
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
    pub fn new(task_cache: SharedTaskCache) -> Self {
        Self {
            task_cache,
            tool_router: Self::tool_router(),
        }
    }

    /// Derives the actor string `ai:<assistant>:<human>`
    /// from the MCP handshake (`clientInfo.name`) and
    /// the tool's `on_behalf_of` parameter.
    fn actor(context: &RequestContext<RoleServer>, on_behalf_of: &str) -> String {
        let assistant = context
            .peer
            .peer_info()
            .map(|info| info.client_info.name.clone())
            .unwrap_or_else(|| "unknown".to_string());
        format!("ai:{assistant}:{on_behalf_of}")
    }

    fn not_found(id: Uuid) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(format!("task {id} not found"))])
    }
}

fn parse_err(e: &str) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

pub(super) mod categories {
    pub const NAME: &str = "categories";
    pub const URI: &str = "kid://categories";
}
pub(super) mod contexts {
    pub const NAME: &str = "contexts";
    pub const URI: &str = "kid://contexts";
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
