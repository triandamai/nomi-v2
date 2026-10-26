import type { TransferOrgSummary } from '$lib/types';

const UUID = '[0-9a-f-]{36}';

/**
 * Allowed proxy route paths for project API proxy.
 * Includes transfer and transfer/orgs for project ownership migration.
 */
export const ALLOWED_PROJECT_API_PATHS = new RegExp(
	'^(' +
		[
			'',
			'runtime',
			'runtime/check',
			`runtime/runs/${UUID}`,
			'snapshot',
			'manifest',
			'files/.+',
			'issues',
			'issues/\\d+',
			'deploy',
			'deploy/publish',
			`deploy/publishes/${UUID}`,
			`deploy/publishes/${UUID}/rollback`,
			'deploy/deployments/[A-Za-z0-9-]+',
			'deploy/logs',
			'deploy/usage',
			'deploy/events',
			'deploy/environments/[a-z0-9-]+',
			'members',
			`members/${UUID}`,
			'access',
			'database',
			'database/credentials',
			'database/tables',
			'database/query',
			'showcase',
			'domain',
			'domain/check',
			'transfer',
			'transfer/orgs',
			'import/github',
			'import/files',
			'relay/preview',
			// Secret names as the backend takes them (project_secrets::check_name).
			'secrets',
			'secrets/[A-Z_][A-Z0-9_]{0,63}',
			'agent-permissions',
			`agent-permissions/${UUID}`,
		].join('|') +
		')$',
);

/**
 * Filters a list of transfer candidate organizations by search query (case-insensitive).
 */
export function filterTransferOrgs(orgs: TransferOrgSummary[], query: string): TransferOrgSummary[] {
	const trimmed = query.trim().toLowerCase();
	if (!trimmed) return orgs;
	return orgs.filter((org) => org.name.toLowerCase().includes(trimmed));
}

/**
 * Checks if a user has sufficient permissions to transfer a project.
 * Only the project owner or an organization admin/owner can initiate a project transfer.
 */
export function canTransferProject(params: { isProjectOwner: boolean; orgRole?: string | null }): boolean {
	if (params.isProjectOwner) return true;
	return params.orgRole === 'owner' || params.orgRole === 'admin';
}

/**
 * Formats parameters for the transfer confirmation notice message.
 */
export function formatTransferNoticeParams(selectedOrg: TransferOrgSummary): { org: string; owner: string } {
	return {
		org: selectedOrg.name,
		owner: selectedOrg.owner_name?.trim() || '—',
	};
}
