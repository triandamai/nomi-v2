export interface PermissionResourceDef {
	resource: string;
	label: string;
	description: string;
}

// The `user` and `system_config` resources are the only ones the backend actually gates today
// (see admin_users.rs's require_user_permission and settings.rs's has_permission calls) — this
// list is the curated catalog shown in the admin "Update role" grant form. Anything else is
// still grantable via the "Other" custom-resource row, since the backend's resource string is
// free-form and other product areas may start enforcing new resources later.
export const ADMIN_PERMISSION_RESOURCES: PermissionResourceDef[] = [
	{ resource: 'user', label: 'Users', description: 'View and manage the admin user roster' },
	{ resource: 'system_config', label: 'System settings', description: 'LLM and embedding provider configuration' },
];

export const CUSTOM_PERMISSION_RESOURCE = 'other';
