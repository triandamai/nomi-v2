// The configuration sections Admin → Configuration shows, in order, with their copy.
import { m } from '$lib/paraglide/messages';

export interface Field {
	name: string;
	/** The environment variable it falls back to; `null` when it's only ever set here. */
	env: string | null;
	kind: 'text' | 'number' | 'secret' | 'choice' | 'domain';
	choices?: string[];
	value: string | null;
	set: boolean;
	source: 'saved' | 'environment' | 'none';
}
export interface Section {
	name: string;
	fields: Field[];
}

export type Copy = { label: () => string; hint?: () => string };
export const SECTIONS: Record<string, { title: () => string; lede: () => string; note?: () => string; fields: Record<string, Copy> }> = {
	attachments: {
		title: m.config_attachments,
		lede: m.config_attachments_lede,
		fields: {
			max_file_mb: { label: m.config_attachments_max_file_mb },
			max_video_mb: { label: m.config_attachments_max_video_mb },
			max_message_mb: { label: m.config_attachments_max_message_mb },
			max_files: { label: m.config_attachments_max_files },
			quota_mb: { label: m.config_attachments_quota_mb, hint: m.config_attachments_quota_mb_hint },
		},
	},
	storage: {
		title: m.config_storage,
		lede: m.config_storage_lede,
		fields: {
			bucket: { label: m.config_storage_bucket },
			region: { label: m.config_storage_region, hint: m.config_storage_region_hint },
			access_key_id: { label: m.config_storage_access_key_id },
			secret_access_key: { label: m.config_storage_secret_access_key },
			endpoint_url: { label: m.config_storage_endpoint_url, hint: m.config_storage_endpoint_url_hint },
			public_url_base: { label: m.config_storage_public_url_base, hint: m.config_storage_public_url_base_hint },
		},
	},
	google: {
		title: m.config_google,
		lede: m.config_google_lede,
		fields: {
			client_id: { label: m.config_google_client_id },
			client_secret: { label: m.config_google_client_secret },
			redirect_uri: { label: m.config_google_redirect_uri, hint: m.config_google_redirect_uri_hint },
			signin_redirect_uri: { label: m.config_google_signin_redirect_uri, hint: m.config_google_signin_redirect_uri_hint },
		},
	},
	github: {
		title: m.config_github,
		lede: m.config_github_lede,
		fields: {
			client_id: { label: m.config_github_client_id },
			client_secret: { label: m.config_github_client_secret },
			redirect_uri: { label: m.config_github_redirect_uri, hint: m.config_github_redirect_uri_hint },
		},
	},
	smtp: {
		title: m.config_smtp,
		lede: m.config_smtp_lede,
		fields: {
			host: { label: m.config_smtp_host },
			port: { label: m.config_smtp_port, hint: m.config_smtp_port_hint },
			username: { label: m.config_smtp_username },
			password: { label: m.config_smtp_password },
			security: { label: m.config_smtp_security },
			from: { label: m.config_smtp_from, hint: m.config_smtp_from_hint },
		},
	},
	agent_loop: {
		title: m.config_agent_loop,
		lede: m.config_agent_loop_lede,
		fields: {
			turn_concurrency: { label: m.config_agent_loop_turn_concurrency, hint: m.config_agent_loop_turn_concurrency_hint },
			turn_time_limit_secs: { label: m.config_agent_loop_turn_time_limit_secs, hint: m.config_agent_loop_turn_time_limit_secs_hint },
			delegation_time_limit_secs: { label: m.config_agent_loop_delegation_time_limit_secs, hint: m.config_agent_loop_delegation_time_limit_secs_hint },
			issues_per_project: { label: m.config_agent_loop_issues_per_project, hint: m.config_agent_loop_issues_per_project_hint },
		},
	},
	relay: {
		title: m.config_relay,
		lede: m.config_relay_lede,
		fields: {
			relay_image: { label: m.config_relay_relay_image, hint: m.config_relay_relay_image_hint },
			functions_image: { label: m.config_relay_functions_image, hint: m.config_relay_functions_image_hint },
			postgrest_image: { label: m.config_relay_postgrest_image, hint: m.config_relay_postgrest_image_hint },
			version: { label: m.config_relay_version, hint: m.config_relay_version_hint },
		sdk_package: { label: m.config_relay_sdk_package, hint: m.config_relay_sdk_package_hint },
		sdk_version: { label: m.config_relay_sdk_version, hint: m.config_relay_sdk_version_hint },
		cli_package: { label: m.config_relay_cli_package, hint: m.config_relay_cli_package_hint },
		cli_version: { label: m.config_relay_cli_version, hint: m.config_relay_cli_version_hint },
		},
	},
	domains: {
		title: m.config_domains,
		lede: m.config_domains_lede,
		note: m.config_domains_note,
		fields: {
			apps_base_domain: { label: m.config_domains_apps_base_domain, hint: m.config_domains_apps_base_domain_hint },
		},
	},
	sandbox: {
		title: m.config_sandbox,
		lede: m.config_sandbox_lede,
		note: m.config_sandbox_note,
		fields: {
			enabled: { label: m.config_sandbox_enabled, hint: m.config_sandbox_enabled_hint },
			api_base: { label: m.config_sandbox_api_base },
			api_key: { label: m.config_sandbox_api_key },
			size: { label: m.config_sandbox_size },
			idle_stop_secs: { label: m.config_sandbox_idle_stop_secs },
			auto_stop_secs: { label: m.config_sandbox_auto_stop_secs, hint: m.config_sandbox_auto_stop_secs_hint },
			rollout: { label: m.config_sandbox_rollout, hint: m.config_sandbox_rollout_hint },
			rollout_allowlist: { label: m.config_sandbox_rollout_allowlist, hint: m.config_sandbox_rollout_allowlist_hint },
			rollout_percent: { label: m.config_sandbox_rollout_percent },
			starts_per_minute: { label: m.config_sandbox_starts_per_minute },
			starts_per_hour: { label: m.config_sandbox_starts_per_hour },
			monthly_minutes_per_user: { label: m.config_sandbox_monthly_minutes_per_user },
			reaper_mode: { label: m.config_sandbox_reaper_mode, hint: m.config_sandbox_reaper_mode_hint },
		},
	},
};
export const ORDER = ['attachments', 'storage', 'google', 'smtp', 'agent_loop', 'relay', 'domains', 'sandbox'];
