<script lang="ts">
	// What the crew may see inside this project's app. Iris (the inspector) reads the app blind
	// until an admin lets her: she gets counts and shapes, the person sees the lines and rows.
	import { m } from '$lib/paraglide/messages';
	import Switch from '$lib/components/m3/Switch.svelte';
	import { projectApi } from '$lib/projectApi';

	let { projectId, isAdmin }: { projectId: string; isAdmin: boolean } = $props();

	interface AgentPermission {
		id: string;
		key: string;
		agent_type: string;
		permission: string;
		scope: 'allow' | 'denied';
		for_this_project: boolean;
		decided_by: string | null;
		updated_at: string;
	}

	const AGENT = 'inspector';
	const ROWS = [
		{ permission: 'read_logs', label: () => m.agent_access_logs(), hint: () => m.agent_access_logs_hint() },
		{ permission: 'read_data', label: () => m.agent_access_data(), hint: () => m.agent_access_data_hint() },
	];

	let grants = $state<AgentPermission[]>([]);
	let error = $state<string | null>(null);
	let busy = $state<string | null>(null);

	async function load() {
		try {
			grants = await projectApi<AgentPermission[]>(projectId, 'agent-permissions');
			error = null;
		} catch (e) {
			error = (e as Error).message;
		}
	}
	$effect(() => {
		void projectId;
		void load();
	});

	/** The grant that applies: this project's own, else the organization's. */
	function grantFor(permission: string): AgentPermission | undefined {
		const matching = grants.filter((g) => g.agent_type === AGENT && g.permission === permission);
		return matching.find((g) => g.for_this_project) ?? matching[0];
	}

	async function toggle(permission: string, on: boolean) {
		busy = permission;
		error = null;
		try {
			const own = grants.find((g) => g.agent_type === AGENT && g.permission === permission && g.for_this_project);
			if (on || !own) {
				await projectApi(projectId, 'agent-permissions', { method: 'PUT', body: { agent_type: AGENT, permission, scope: on ? 'allow' : 'denied' } });
			} else {
				await projectApi(projectId, `agent-permissions/${own.id}`, { method: 'DELETE' });
			}
			await load();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = null;
		}
	}
</script>

<section class="block">
	<h3 class="block__title">{m.agent_access_title()}</h3>
	<p class="block__hint">{m.agent_access_lede()}</p>
	<ul class="access">
		{#each ROWS as row (row.permission)}
			{@const grant = grantFor(row.permission)}
			<li class="access__row">
				<div class="access__text">
					<span class="access__label">{row.label()}</span>
					<span class="access__hint">
						{row.hint()}
						{#if grant && !grant.for_this_project}{' '}{m.agent_access_org_wide()}{/if}
					</span>
				</div>
				<Switch
					checked={grant?.scope === 'allow'}
					disabled={!isAdmin || busy !== null}
					aria-label={row.label()}
					onchange={(on) => toggle(row.permission, on)}
				/>
			</li>
		{/each}
	</ul>
	{#if !isAdmin}<p class="block__hint">{m.pj_admins_only()}</p>{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<style>
	.access {
		display: flex;
		flex-direction: column;
		margin: 4px 0 0;
		padding: 0;
		list-style: none;
	}
	.access__row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		min-height: 56px;
		padding: 8px 0;
	}
	.access__row + .access__row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.access__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.access__label {
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
	}
	.access__hint {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		line-height: 1.4;
	}
	.error {
		margin: 8px 0 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
</style>
