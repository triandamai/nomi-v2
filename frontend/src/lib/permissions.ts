import { m } from '$lib/paraglide/messages';

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
	{
		resource: 'user',
		get label() {
			return m.admin_users();
		},
		get description() {
			return m.perm_users_desc();
		},
	},
	{
		resource: 'system_config',
		get label() {
			return m.perm_system();
		},
		get description() {
			return m.perm_system_desc();
		},
	},
];

export const CUSTOM_PERMISSION_RESOURCE = 'other';
