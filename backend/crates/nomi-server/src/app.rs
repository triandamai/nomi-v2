use axum::{routing::{delete, get, patch, post, put}, Router};
use sqlx::PgPool;
use axum::http::HeaderName;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use nomi_auth::extractor::AuthClaims;
use crate::routes::admin_dashboard as admin_dashboard_routes;
use crate::routes::admin_users as admin_users_routes;
use crate::routes::attachments as attachments_routes;
use crate::routes::auth as auth_routes;
use crate::routes::dynamic_agents as dynamic_agents_routes;
use crate::routes::llm_models as llm_models_routes;
use crate::routes::provider_connections as provider_connections_routes;
use crate::routes::agents as agents_routes;
use crate::routes::home as home_routes;
use crate::routes::money as money_routes;
use crate::routes::notifications as notifications_routes;
use crate::routes::usage as usage_routes;
use crate::routes::connections as connections_routes;
use crate::routes::google_auth as google_auth_routes;
use crate::routes::reminders as reminders_routes;
use crate::routes::memory as memory_routes;
use crate::routes::personality as personality_routes;
use crate::routes::plans as plans_routes;
use crate::routes::projects as projects_routes;
use crate::routes::profile as profile_routes;
use crate::routes::sessions as sessions_routes;
use crate::routes::settings as settings_routes;
use crate::routes::tools as tools_routes;
use nomi_storage::{ProjectStore, S3Config};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
    /// Avatars (attachments and project code share the bucket through their own stores).
    pub s3: Option<S3Config>,
    pub project_storage: ProjectStore,
    pub tool_catalog: std::sync::Arc<nomi_agent_core::ToolCatalog>,
    /// Whether password sign-in waits for an emailed code, and the mailer that sends it.
    pub email_codes: crate::sign_in_codes::EmailCodes,
    /// Where projects are published; None when Shipyard isn't set up.
    pub shipyard: Option<nomi_shipyard::ShipyardConfig>,
    /// Pushes to phones (FCM, APNs), or `NoopPush` when neither is set up.
    pub push: std::sync::Arc<dyn crate::push::PushSender>,
}

impl nomi_auth::extractor::HasJwtSecret for AppState {
    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login_handler))
        .route("/api/auth/refresh", post(auth_routes::refresh_handler))
        .route("/api/auth/logout", post(auth_routes::logout_handler))
        .route("/api/auth/verify", post(auth_routes::verify_handler))
        .route("/api/auth/resend", post(auth_routes::resend_handler))
        .route("/api/auth/challenge/:id", get(auth_routes::challenge_handler))
        .route("/api/auth/google", get(google_auth_routes::get_methods).delete(google_auth_routes::unlink))
        .route("/api/auth/google/available", get(google_auth_routes::available))
        .route("/api/auth/google/start", post(google_auth_routes::start))
        .route("/api/auth/google/link", post(google_auth_routes::start_link))
        .route("/api/auth/google/callback", post(google_auth_routes::callback))
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .route("/api/orgs", get(crate::routes::orgs::list_orgs).post(crate::routes::orgs::create_org))
        .route("/api/orgs/:org_id", get(crate::routes::orgs::get_org).patch(crate::routes::orgs::rename_org))
        .route("/api/orgs/:org_id/invites", post(crate::routes::orgs::create_invite))
        .route("/api/orgs/:org_id/invites/:code", delete(crate::routes::orgs::revoke_invite))
        .route(
            "/api/orgs/:org_id/members/:user_id",
            delete(crate::routes::orgs::remove_member).patch(crate::routes::orgs::set_member_role),
        )
        .route("/api/invites/:code", get(crate::routes::orgs::get_invite))
        .route("/api/invites/:code/accept", post(crate::routes::orgs::accept_invite))
        .route("/api/preferences/project-org", put(crate::routes::orgs::set_project_org))
        .route(
            "/api/sessions",
            post(sessions_routes::create_session).get(sessions_routes::list_sessions),
        )
        .route("/api/sessions/:id", delete(sessions_routes::delete_session).patch(sessions_routes::rename_session))
        .route(
            "/api/sessions/:id/messages",
            get(sessions_routes::list_messages).post(sessions_routes::send_message),
        )
        .route(
            "/api/sessions/:id/messages/:message_id",
            get(sessions_routes::get_message)
                .delete(crate::routes::message_delete::delete_message)
                .patch(crate::routes::message_rerun::edit),
        )
        .route("/api/sessions/:id/messages/:message_id/delete", get(crate::routes::message_delete::preview_delete))
        .route("/api/sessions/:id/messages/:message_id/retry", post(crate::routes::message_retry::retry_failed_run))
        .route("/api/sessions/:id/messages/:message_id/regenerate", post(crate::routes::message_rerun::regenerate))
        .route("/api/sessions/:id/messages/:message_id/versions", get(crate::routes::message_rerun::versions))
        .route(
            "/api/sessions/:id/agent-plans/:agent_session_id",
            get(sessions_routes::list_agent_plans),
        )
        .route(
            "/api/sessions/:id/agent-plans/:agent_session_id/:plan_id/items/:index",
            put(sessions_routes::set_agent_plan_item),
        )
        .route("/api/sessions/:id/ws", get(sessions_routes::session_stream))
        .route("/api/realtime/ws", get(crate::routes::realtime::realtime_stream))
        .route("/api/devices", post(crate::routes::devices::register))
        .route("/api/devices/:id", delete(crate::routes::devices::remove))
        .route("/api/me/activity/ws", get(sessions_routes::user_activity_stream))
        .route("/api/sessions/:id/agent-activity", get(sessions_routes::list_agent_activity))
        .route("/api/sessions/:id/agent-status", get(sessions_routes::get_agent_status))
        .route(
            "/api/sessions/:id/thinking",
            get(sessions_routes::get_thinking_level).put(sessions_routes::set_thinking_level),
        )
        .route(
            "/api/sessions/:id/messages/:message_id/feedback",
            put(sessions_routes::put_message_feedback).delete(sessions_routes::delete_message_feedback),
        )
        .route("/api/sessions/:id/messages/:message_id/memories", get(sessions_routes::list_message_memories))
        .route(
            "/api/sessions/:id/messages/:message_id/approval",
            put(sessions_routes::resolve_approval),
        )
        .route(
            "/api/admin/settings/llm/models",
            get(llm_models_routes::list_admin_models).post(llm_models_routes::create_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id",
            put(llm_models_routes::update_admin_model).delete(llm_models_routes::delete_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id/enabled",
            put(llm_models_routes::set_admin_model_enabled),
        )
        .route(
            "/api/admin/settings/llm/models/:id/default",
            put(llm_models_routes::set_default_admin_model),
        )
        .route("/api/admin/settings/llm/files-model", put(llm_models_routes::set_files_admin_model))
        .route("/api/admin/settings/llm/coding-model", put(llm_models_routes::set_coding_admin_model))
        .route(
            "/api/admin/settings/llm/models/fetch-models",
            post(llm_models_routes::fetch_provider_models),
        )
        .route("/api/llm/models", get(llm_models_routes::get_user_models))
        .route("/api/llm/selection", put(llm_models_routes::put_user_selection))
        .route("/api/llm/coding-selection", put(llm_models_routes::put_user_coding_selection))
        .route("/api/llm/fetch-models", post(llm_models_routes::fetch_user_models))
        .route("/api/llm/model-options", get(crate::routes::model_options::list_options).put(crate::routes::model_options::set_visibility))
        .route("/api/sessions/:id/model", get(crate::routes::model_options::get_chat_model).put(crate::routes::model_options::set_chat_model))
        .route("/api/llm/default-model", put(crate::routes::model_options::set_default_model))
        .route("/api/llm/providers", get(provider_connections_routes::list_providers))
        .route("/api/llm/providers/:id/models", get(provider_connections_routes::list_provider_catalog_models))
        .route("/api/llm/connections", get(provider_connections_routes::list_own).post(provider_connections_routes::connect_own))
        .route(
            "/api/llm/connections/:id",
            put(provider_connections_routes::update_own).delete(provider_connections_routes::disconnect_own),
        )
        .route("/api/llm/connections/:id/models", post(provider_connections_routes::own_models))
        .route(
            "/api/admin/settings/llm/connections",
            get(provider_connections_routes::list_shared).post(provider_connections_routes::connect_shared),
        )
        .route(
            "/api/admin/settings/llm/connections/:id",
            put(provider_connections_routes::update_shared).delete(provider_connections_routes::disconnect_shared),
        )
        .route("/api/admin/settings/llm/connections/:id/models", post(provider_connections_routes::shared_models))
        .route("/api/agents", get(agents_routes::list_crew))
        .route("/api/home", get(home_routes::home_summary))
        .route("/api/home/updates", get(home_routes::updates_page))
        .route("/api/home/today", get(home_routes::today_page))
        .route("/api/home/plans", get(home_routes::plans_page))
        .route("/api/usage", get(usage_routes::usage_month))
        .route("/api/usage/brief", get(usage_routes::usage_brief))
        .route("/api/money", get(money_routes::money_summary))
        .route("/api/money/transactions", post(money_routes::add_transaction))
        .route("/api/money/budgets", put(money_routes::set_budget))
        .route("/api/money/period", put(money_routes::set_period))
        .route("/api/money/budgets/:category", delete(money_routes::delete_budget))
        .route("/api/connections", get(connections_routes::overview))
        .route("/api/connections/hevy", get(connections_routes::get_hevy).put(connections_routes::connect_hevy).delete(connections_routes::disconnect_hevy))
        .route("/api/connections/google", get(connections_routes::get_google).delete(connections_routes::disconnect_google))
        .route("/api/connections/google/start", post(connections_routes::start_google))
        .route("/api/connections/google/callback", post(connections_routes::complete_google))
        .route("/api/reminders", get(reminders_routes::list_reminders).post(reminders_routes::create_reminder))
        .route("/api/reminders/:id", post(reminders_routes::reminder_action))
        .route("/api/scheduled-tasks/:id/cancel", post(reminders_routes::cancel_scheduled_task))
        .route("/api/memory", get(memory_routes::list_my_memories))
        .route("/api/memory/:memory_id", delete(memory_routes::delete_my_memory).put(memory_routes::edit_my_memory))
        .route("/api/memory/:memory_id/confirm", post(memory_routes::confirm_my_memory))
        .route(
            "/api/personality/history",
            get(personality_routes::get_personality_history),
        )
        .route("/api/personality/rollback", post(personality_routes::rollback_personality))
        .route(
            "/api/sessions/:id/personality",
            get(personality_routes::get_chat_personality)
                .put(personality_routes::put_chat_personality)
                .delete(personality_routes::reset_chat_personality),
        )
        .route("/api/admin/dashboard", get(admin_dashboard_routes::get_dashboard))
        .route("/api/admin/usage", get(crate::routes::admin_usage::get_usage))
        .route("/api/maintenance", get(crate::routes::maintenance::get))
        .route("/api/showcase", get(crate::routes::showcase::list))
        .route("/api/sessions/:id/shares", post(crate::routes::shares::create))
        .route("/api/sessions/:id/shares/:share_id", put(crate::routes::shares::update).delete(crate::routes::shares::revoke))
        .route("/api/shared/:token", get(crate::routes::shares::read))
        .route("/api/admin/maintenance", get(crate::routes::maintenance::admin_get).put(crate::routes::maintenance::put))
        .route("/api/support/v1/ask", post(crate::routes::support::ask))
        .route("/api/admin/support/articles", get(crate::routes::support::list_articles).post(crate::routes::support::create_article))
        .route(
            "/api/admin/support/articles/:id",
            put(crate::routes::support::update_article).delete(crate::routes::support::delete_article),
        )
        .route("/api/admin/support/drafts", get(crate::routes::support::list_drafts))
        .route("/api/admin/support/drafts/:id/publish", post(crate::routes::support::publish_draft))
        .route("/api/admin/support/drafts/:id/dismiss", post(crate::routes::support::dismiss_draft))
        .route("/api/admin/support/keys", get(crate::routes::support::list_keys).post(crate::routes::support::create_key))
        .route("/api/admin/support/keys/:id", delete(crate::routes::support::revoke_key))
        .route("/api/admin/support/questions", get(crate::routes::support::list_questions))
        .route("/api/admin/configuration", get(crate::routes::admin_configuration::get))
        .route("/api/admin/configuration/:section", axum::routing::put(crate::routes::admin_configuration::save))
        .route("/api/admin/shipyard", get(crate::routes::deploy::admin_get))
        .route("/api/admin/shipyard/apps", get(crate::routes::admin_shipyard::list_apps))
        .route("/api/admin/shipyard/apps/link", post(crate::routes::admin_shipyard::link_app))
        .route("/api/admin/shipyard/webhooks", get(crate::routes::admin_shipyard::list_webhooks).post(crate::routes::admin_shipyard::create_webhook))
        .route(
            "/api/admin/shipyard/webhooks/:id",
            patch(crate::routes::admin_shipyard::update_webhook).delete(crate::routes::admin_shipyard::delete_webhook),
        )
        .route("/api/admin/shipyard/webhooks/:id/rotate", post(crate::routes::admin_shipyard::rotate_webhook))
        .route("/api/admin/shipyard/webhooks/:id/test", post(crate::routes::admin_shipyard::test_webhook))
        .route("/api/admin/shipyard/webhooks/:id/deliveries", get(crate::routes::admin_shipyard::webhook_deliveries))
        .route("/api/admin/shipyard/deliveries/:id/redeliver", post(crate::routes::admin_shipyard::redeliver))
        .route("/api/admin/shipyard/orgs", get(crate::routes::admin_shipyard::list_orgs))
        .route("/api/admin/shipyard/orgs/:id", delete(crate::routes::admin_shipyard::remove_org))
        .route("/api/admin/shipyard/orgs/:id/sync", post(crate::routes::admin_shipyard::sync_org))
        .route(
            "/api/admin/shipyard/orgs/:org/apps/:app",
            get(crate::routes::admin_shipyard::get_app).delete(crate::routes::admin_shipyard::delete_app),
        )
        .route("/api/admin/shipyard/orgs/:org/apps/:app/envs/:env", delete(crate::routes::admin_shipyard::delete_env))
        .route("/api/admin/shipyard/orgs/:org/apps/:app/envs/:env/duplicate", post(crate::routes::admin_shipyard::duplicate_env))
        .route("/api/admin/shipyard/orgs/:org/apps/:app/secrets", get(crate::routes::admin_shipyard::list_secrets))
        .route(
            "/api/admin/shipyard/orgs/:org/apps/:app/secrets/:name",
            put(crate::routes::admin_shipyard::set_secret).delete(crate::routes::admin_shipyard::delete_secret),
        )
        .route("/api/admin/shipyard/events", get(crate::routes::admin_shipyard::list_events))
        .route("/api/admin/shipyard/events/catch-up", post(crate::routes::admin_shipyard::catch_up))
        .route("/api/admin/shipyard/webhook", post(crate::routes::deploy::admin_register_webhook))
        .route("/api/admin/agents", get(admin_dashboard_routes::get_agents))
        .route("/api/admin/agents/ws", get(admin_dashboard_routes::admin_agents_stream))
        .route("/api/admin/agent-events", get(admin_dashboard_routes::list_agent_events))
        .route(
            "/api/admin/dynamic-agents",
            get(dynamic_agents_routes::list_dynamic_agents).post(dynamic_agents_routes::create_dynamic_agent),
        )
        .route("/api/admin/dynamic-agents/:id", put(dynamic_agents_routes::update_dynamic_agent))
        .route("/api/admin/dynamic-agents/:id/toggle-active", post(dynamic_agents_routes::toggle_active_dynamic_agent))
        .route(
            "/api/admin/users",
            get(admin_users_routes::list_users),
        )
        .route(
            "/api/admin/users/:id",
            get(admin_users_routes::get_user_detail),
        )
        .route(
            "/api/admin/users/:id/profile",
            put(admin_users_routes::update_user_profile),
        )
        .route(
            "/api/admin/users/:id/permissions",
            post(admin_users_routes::grant_user_permission),
        )
        .route("/api/admin/users/:id/admin-access", put(admin_users_routes::put_admin_access))
        .route("/api/admin/permissions/catalog", get(crate::routes::admin_access::catalog))
        .route(
            "/api/admin/users/:id/permissions/:permission_id",
            delete(admin_users_routes::revoke_user_permission),
        )
        .route(
            "/api/admin/users/:id/memberships",
            post(admin_users_routes::assign_user_to_org),
        )
        .route(
            "/api/admin/users/:id/memberships/:org_id",
            delete(admin_users_routes::remove_user_from_org),
        )
        .route("/api/admin/tools", get(tools_routes::list_tools))
        .route("/api/admin/tools/web_search/test", post(tools_routes::test_web_search))
        .route("/api/admin/tools/speak/voices", get(tools_routes::voice_options))
        .route("/api/admin/tools/speak/test", post(tools_routes::test_voice))
        .route("/api/voice/:user_id/:file", get(crate::routes::voice::get_recording))
        .route("/api/admin/tools/:name", put(tools_routes::set_tool_enabled))
        .route("/api/admin/tools/:name/settings", put(tools_routes::save_tool_settings))
        .route("/api/admin/orgs", get(admin_users_routes::list_orgs))
        .route(
            "/api/admin/settings/embedding",
            get(settings_routes::get_embedding_settings).put(settings_routes::put_embedding_settings),
        )
        .route(
            "/api/admin/settings/embedding/fetch-models",
            post(settings_routes::fetch_embedding_provider_models),
        )
        .route(
            "/api/profile",
            get(profile_routes::get_profile).put(profile_routes::put_profile),
        )
        .route(
            "/api/attachments",
            // Uploads are streamed and limited by the handler itself (nomi_attachments::Limits).
            post(attachments_routes::upload).layer(axum::extract::DefaultBodyLimit::disable()),
        )
        .route("/api/attachments/limits", get(attachments_routes::limits))
        .route("/api/attachments/:id", get(attachments_routes::get_attachment).delete(attachments_routes::delete_attachment))
        .route("/api/attachments/:id/content", get(attachments_routes::get_content))
        .route("/api/attachments/:id/preview", get(attachments_routes::get_preview))
        .route("/api/plans", get(plans_routes::list_for_user))
        .route("/api/admin/plans", get(plans_routes::admin_list).post(plans_routes::admin_create))
        .route("/api/admin/plans/:id", put(plans_routes::admin_update).delete(plans_routes::admin_delete))
        .route("/api/admin/plans/:id/default", put(plans_routes::admin_set_default))
        .route("/api/admin/users/:id/subscription", get(plans_routes::admin_get_subscription).put(plans_routes::admin_put_subscription))
        .route("/api/notifications", get(notifications_routes::list))
        .route("/api/notifications/unread", get(notifications_routes::unread))
        .route("/api/notifications/read-all", post(notifications_routes::mark_all_read))
        .route("/api/notifications/preferences", get(notifications_routes::get_preferences).put(notifications_routes::put_preferences))
        .route("/api/notifications/:id/read", post(notifications_routes::mark_read))
        .route("/api/admin/notifications/broadcast", post(notifications_routes::broadcast))
        .route("/api/admin/notifications/broadcasts", get(notifications_routes::list_broadcasts))
        .route("/api/profile/avatar/upload-url", post(profile_routes::request_avatar_upload_url))
        .route("/api/avatars/:user_id/:file", get(profile_routes::get_avatar))
        .route(
            "/api/profile/avatar",
            post(profile_routes::upload_avatar)
                .layer(axum::extract::DefaultBodyLimit::max(profile_routes::MAX_AVATAR_BYTES + 1024)),
        )
        .route(
            "/api/preferences",
            get(profile_routes::get_preferences).put(profile_routes::put_preferences),
        )
        .route(
            "/api/projects",
            get(projects_routes::list_projects).post(projects_routes::create_project_session),
        )
        .route("/api/projects/by-session/:sessionId", get(projects_routes::get_project_by_session))
        .route("/api/projects/:id", get(projects_routes::get_project).delete(crate::routes::deploy::delete_project))
        .route("/api/projects/:id/transfer/orgs", get(projects_routes::search_transfer_orgs))
        .route("/api/projects/:id/transfer", post(projects_routes::transfer_project))
        .route(
            "/api/projects/:id/agent-permissions",
            get(crate::routes::agent_permissions::list).put(crate::routes::agent_permissions::set),
        )
        .route("/api/projects/:id/agent-permissions/:permission_id", delete(crate::routes::agent_permissions::remove))
        .route("/api/projects/:id/deploy", get(crate::routes::deploy::overview))
        .route("/api/projects/:id/showcase", get(crate::routes::showcase::get).put(crate::routes::showcase::put))
        .route("/api/projects/:id/deploy/publish", post(crate::routes::deploy::publish))
        .route("/api/projects/:id/ws", get(crate::routes::deploy::project_stream))
        .route("/api/projects/:id/domain", get(crate::routes::domain::get).put(crate::routes::domain::change))
        .route("/api/projects/:id/domain/check", get(crate::routes::domain::check))
        .route("/api/projects/:id/deploy/publishes/:publish_id", get(crate::routes::deploy::get_publish))
        .route("/api/projects/:id/deploy/publishes/:publish_id/rollback", post(crate::routes::deploy::rollback))
        .route("/api/projects/:id/deploy/deployments/:deployment_id", get(crate::routes::deploy::get_deployment))
        .route("/api/projects/:id/deploy/logs", get(crate::routes::deploy::runtime_logs))
        .route("/api/projects/:id/deploy/usage", get(crate::routes::deploy::usage))
        .route("/api/projects/:id/deploy/metrics", get(crate::routes::deploy::metrics))
        .route("/api/projects/:id/deploy/events", get(crate::routes::deploy::events))
        .route("/api/projects/:id/deploy/environments/:env", delete(crate::routes::deploy::take_offline))
        .route(
            "/api/projects/:id/import",
            post(crate::routes::project_import::import_archive)
                .layer(axum::extract::DefaultBodyLimit::max(crate::routes::project_import::MAX_ARCHIVE_BYTES)),
        )
        .route("/api/projects/:id/import/github", post(crate::routes::project_import::import_github))
        .route(
            "/api/projects/:id/import/files",
            post(crate::routes::project_import::import_batch).layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024)),
        )
        .route("/api/projects/:id/members", get(crate::routes::project_members::list))
        .route(
            "/api/projects/:id/presence",
            get(crate::routes::project_presence::list)
                .post(crate::routes::project_presence::heartbeat)
                .delete(crate::routes::project_presence::leave),
        )
        .route("/api/projects/:id/typing", post(crate::routes::project_presence::typing))
        .route("/api/projects/:id/activity", get(crate::routes::project_activity::list))
        .route("/api/projects/:id/access", put(crate::routes::project_members::set_access))
        .route(
            "/api/projects/:id/members/:user_id",
            put(crate::routes::project_members::put_member).delete(crate::routes::project_members::remove_member),
        )
        .route("/api/projects/:id/database", get(crate::routes::project_database::info))
        .route("/api/projects/:id/database/credentials", post(crate::routes::project_database::credentials))
        .route("/api/projects/:id/database/tables", get(crate::routes::project_database::tables))
        .route("/api/projects/:id/database/query", post(crate::routes::project_database::run_query))
        .route("/api/projects/:id/relay/preview", get(crate::routes::project_relay::preview))
        .route("/api/projects/:id/relay", get(crate::routes::project_relay::overview))
        .route("/api/projects/:id/relay/query", post(crate::routes::project_relay::run_query))
        .route("/api/projects/:id/relay/retry", post(crate::routes::project_relay::retry))
        .route("/api/projects/:id/secrets", get(crate::routes::project_secrets::list))
        .route(
            "/api/projects/:id/secrets/:name",
            put(crate::routes::project_secrets::set).delete(crate::routes::project_secrets::remove),
        )
        .route("/api/hooks/shipyard", post(crate::routes::deploy::webhook))
        .route(
            "/api/projects/:id/issues",
            get(crate::routes::project_issues::list_issues).post(crate::routes::project_issues::create_issue),
        )
        .route(
            "/api/projects/:id/issues/:number",
            get(crate::routes::project_issues::get_issue)
                .patch(crate::routes::project_issues::update_issue)
                .delete(crate::routes::project_issues::delete_issue),
        )
        .route(
            "/api/projects/:id/files/*path",
            get(projects_routes::get_project_file).put(projects_routes::put_project_file).delete(projects_routes::delete_project_file),
        )
        .route("/api/projects/:id/runtime", get(crate::routes::project_runtime::check_in))
        .route("/api/projects/:id/runtime/runs/:run_id", post(crate::routes::project_runtime::finish_run))
        .route("/api/projects/:id/runtime/check", post(crate::routes::project_runtime::record_check))
        .route("/api/projects/:id/manifest", get(crate::routes::project_runtime::manifest))
        .route("/api/projects/:id/snapshot", get(crate::routes::project_runtime::snapshot))
        .route("/api/projects/:id/preview", get(projects_routes::preview_project_index))
        .route("/api/projects/:id/preview/*path", get(projects_routes::preview_project_file))
        // Outermost last: every request gets an id (kept when the caller sent one), every
        // response carries it back, and errors become problem details for clients that ask
        // (see crate::problem).
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static(crate::problem::REQUEST_ID_HEADER)))
        .layer(SetRequestIdLayer::new(HeaderName::from_static(crate::problem::REQUEST_ID_HEADER), MakeRequestUuid))
        .layer(axum::middleware::from_fn(crate::problem::problem_details))
        .with_state(state)
}
