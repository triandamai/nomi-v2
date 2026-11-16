<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { tick, untrack } from 'svelte';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import SoulEditor from '$lib/components/tools/SoulEditor.svelte';
	import ToolPicker from '$lib/components/tools/ToolPicker.svelte';
	import { rosterKey } from '$lib/crew';
	import { activityFailed, levelField, PERMISSION_LEVELS, SOUL_MAX_CHARS, toolLabel, TOOLS_SET_FIELD } from '$lib/crewSettings';
	import { timeAgo } from '$lib/i18n';
	import { addTool } from '$lib/toolPicker';
	import type { CrewActivityItem, CrewPermissionLevel } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const member = $derived(data.member);
	const key = $derived(rosterKey(member.agent_type));
	const LEVEL_LABELS: Record<CrewPermissionLevel, () => string> = {
		allow: m.crewmember_level_allow,
		ask: m.crewmember_level_ask,
		never: m.crewmember_level_never,
	};
	const levelOptions = PERMISSION_LEVELS.map((value) => ({ value, label: LEVEL_LABELS[value]() }));

	let editing = $state(false);
	let draft = $state('');
	let removeOpen = $state(false);
	let enabledForm: HTMLFormElement | undefined = $state();
	// The tools of a member the person made, as edited here (the saved ones come with the crew).
	let tools = $state<string[]>(untrack(() => [...(data.member.tools ?? [])]));
	let toolAdded = $state<string | null>(null);
	$effect(() => {
		tools = [...(member.tools ?? [])];
	});

	/** "@tool" in the soul gives a member the person made that tool (the field shows it at once). */
	function mentioned(tool: string) {
		if (!member.is_own || tools.includes(tool)) return;
		tools = addTool(tools, tool);
		toolAdded = tool;
	}

	function startEdit() {
		draft = data.soul.text;
		toolAdded = null;
		editing = true;
	}

	function activityLine(item: CrewActivityItem): string {
		switch (item.kind) {
			case 'task':
				return m.crewmember_activity_task({ task: item.text });
			case 'tool':
				return m.crewmember_activity_tool({ tool: toolLabel(item.text) });
			case 'started':
				return m.crewmember_activity_started();
			default:
				return item.text || m.crewmember_activity_finished();
		}
	}
</script>

<div class="member-page">
	<div class="member-page__inner">
		<a class="member-page__back" href="/crew">← {m.crewmember_back()}</a>
		<header class="member-page__head">
			<AgentShape agent={key} size={64} face />
			<div class="member-page__title">
				<h1 class="md-headline-medium">{member.name}</h1>
				<p class="member-page__role">{member.role}</p>
			</div>
		</header>

		<section class="panel" aria-labelledby="on-title">
			<div class="row">
				<div class="row__text">
					<h2 id="on-title" class="panel__title">{m.crewmember_on()}</h2>
					<p class="panel__lede">
						{member.is_default ? m.crewmember_always_on() : m.crewmember_on_hint({ name: member.name })}
					</p>
				</div>
				{#if !member.is_default}
					<form method="POST" action="?/enabled" bind:this={enabledForm} use:enhance>
						<Switch
							name="enabled"
							checked={member.enabled !== false}
							aria-label={m.crewmember_on()}
							onchange={() => tick().then(() => enabledForm?.requestSubmit())}
						/>
					</form>
				{/if}
			</div>
		</section>

		<section class="panel" aria-labelledby="perm-title">
			<h2 id="perm-title" class="panel__title">{m.crewmember_permissions()}</h2>
			<p class="panel__lede">{m.crewmember_permissions_lede({ name: member.name })}</p>
			{#if data.permissions.length === 0}
				<p class="panel__empty">{m.crewmember_permissions_none({ name: member.name })}</p>
			{:else}
				<form method="POST" action="?/permissions" use:enhance={() => async ({ update }) => update({ reset: false })}>
					<ul class="perms">
						{#each data.permissions as permission (permission.key)}
							<li class="perms__row">
								<div class="row__text">
									<span class="perms__label">{permission.label}</span>
									{#if permission.description}<span class="perms__desc">{permission.description}</span>{/if}
								</div>
								<Select label={permission.label} name={levelField(permission.key)} options={levelOptions} value={permission.level} class="perms__select" />
							</li>
						{/each}
					</ul>
					<div class="panel__actions">
						{#if form?.saved === 'permissions'}<span class="panel__note">{m.crewmember_saved()}</span>{/if}
						{#if form?.failed === 'permissions'}<span class="panel__error">{m.crewmember_failed()}</span>{/if}
						<Button variant="tonal" size="s" type="submit">{m.common_save()}</Button>
					</div>
				</form>
			{/if}
		</section>

		<section class="panel" aria-labelledby="soul-title">
			<div class="row">
				<h2 id="soul-title" class="panel__title">soul.md</h2>
				{#if !editing}<Button variant="text" size="s" onclick={startEdit}>{m.crewmember_edit()}</Button>{/if}
			</div>
			<p class="panel__lede">{m.crewmember_soul_lede({ name: member.name })}</p>
			{#if editing}
				<form
					method="POST"
					action="?/soul"
					use:enhance={() =>
						async ({ result, update }) => {
							await update({ reset: false });
							if (result.type === 'success') editing = false;
						}}
				>
					{#if member.is_own}<input type="hidden" name={TOOLS_SET_FIELD} value="1" />{#each tools as tool (tool)}<input type="hidden" name="tools" value={tool} />{/each}{/if}
					<SoulEditor bind:value={draft} name="text" label="soul.md" maxlength={SOUL_MAX_CHARS} rows={10} crew={data.crew} tools={data.catalog} ontoolmention={mentioned} />
					<div class="panel__actions">
						{#if toolAdded}<span class="panel__note" role="status">{m.crewmember_tool_added({ tool: toolAdded })}</span>{/if}
						{#if form?.failed === 'tools'}<span class="panel__error">{m.crewmember_failed()}</span>{/if}
						{#if form?.failed === 'soul'}<span class="panel__error">{m.crewmember_failed()}</span>{/if}
						<Button variant="text" size="s" onclick={() => (editing = false)}>{m.common_cancel()}</Button>
						<Button variant="tonal" size="s" type="submit">{m.common_save()}</Button>
					</div>
				</form>
			{:else if data.soul.text}
				<pre class="soul__text">{data.soul.text}</pre>
			{:else}
				<p class="panel__empty">{m.crewmember_soul_empty({ name: member.name })}</p>
			{/if}

			{#if data.versions.length > 0}
				<h3 class="panel__subtitle">{m.crewmember_versions()}</h3>
				<ul class="list">
					{#each data.versions as version (version.version)}
						<li class="list__row">
							<div class="row__text">
								<span>{m.crewmember_version({ version: version.version })}</span>
								<span class="list__meta">{timeAgo(version.created_at)}</span>
							</div>
							{#if version.is_current}
								<span class="list__meta">{m.crewmember_in_use()}</span>
							{:else}
								<form method="POST" action="?/restore" use:enhance>
									<input type="hidden" name="version" value={version.version} />
									<Button variant="text" size="xs" type="submit">{m.crewmember_restore()}</Button>
								</form>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		{#if member.is_own}
			<section class="panel" aria-labelledby="tools-title">
				<h2 id="tools-title" class="panel__title">{m.crewmember_tools()}</h2>
				<p class="panel__lede">{m.crewmember_tools_lede({ name: member.name })}</p>
				<form method="POST" action="?/tools" use:enhance={() => async ({ update }) => update({ reset: false })}>
					<ToolPicker label={m.crewmember_tools()} options={data.catalog} bind:selected={tools} name="tools" />
					<div class="panel__actions">
						{#if form?.saved === 'tools'}<span class="panel__note">{m.crewmember_saved()}</span>{/if}
						{#if form?.failed === 'tools'}<span class="panel__error">{m.crewmember_failed()}</span>{/if}
						<Button variant="tonal" size="s" type="submit">{m.common_save()}</Button>
					</div>
				</form>
			</section>
		{/if}

		<section class="panel" aria-labelledby="activity-title">
			<h2 id="activity-title" class="panel__title">{m.crewmember_activity({ count: data.activity.length })}</h2>
			{#if data.activity.length === 0}
				<p class="panel__empty">{m.crewmember_activity_empty()}</p>
			{:else}
				<ul class="list">
					{#each data.activity as item, index (index)}
						<li class="list__row" data-failed={activityFailed(item)}>
							<div class="row__text">
								<span class="list__line">{activityLine(item)}</span>
								<span class="list__meta">
									{#if item.session_id}<a href="/chat/{item.session_id}">{item.chat_title ?? m.common_open()}</a> ·{/if}
									{timeAgo(item.at)}
								</span>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		{#if member.is_own}
			<div>
				<Button variant="text" size="s" onclick={() => (removeOpen = true)}>{m.crewmember_remove()}</Button>
			</div>
			<Dialog bind:open={removeOpen} headline={m.crewmember_remove()}>
				<p>{m.crewmember_remove_confirm({ name: member.name })}</p>
				{#snippet actions()}
					<Button variant="text" onclick={() => (removeOpen = false)}>{m.common_cancel()}</Button>
					<form method="POST" action="?/remove" use:enhance>
						<Button variant="filled" type="submit">{m.common_delete()}</Button>
					</form>
				{/snippet}
			</Dialog>
		{/if}
	</div>
</div>

<style>
	.member-page {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(16px, 4vw, 44px) 44px;
		box-sizing: border-box;
		background: var(--nomi-color-ground);
	}
	.member-page__inner {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 760px;
		margin: 0 auto;
	}
	.member-page__back {
		align-self: flex-start;
		min-height: 44px;
		display: inline-flex;
		align-items: center;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
	}
	.member-page__back:hover,
	.member-page__back:focus-visible {
		color: var(--md-sys-color-primary);
	}
	.member-page__head {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	.member-page__title h1 {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.member-page__role {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 18px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 650;
	}
	.panel__subtitle {
		margin: 8px 0 0;
		font-size: 0.9375rem;
		font-weight: 650;
	}
	.panel__lede,
	.panel__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.45;
	}
	.panel__actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.panel__note {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.panel__error {
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.row__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.perms,
	.list {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.perms__row,
	.list__row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 12px;
		padding: 10px 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.perms__label {
		font-weight: 600;
	}
	.perms__desc,
	.list__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.list__meta a {
		color: inherit;
	}
	.list__meta a:hover,
	.list__meta a:focus-visible {
		color: var(--md-sys-color-primary);
	}
	.list__row[data-failed='true'] .list__line {
		color: var(--md-sys-color-error);
	}
	.perms :global(.perms__select) {
		min-width: 150px;
	}
	.soul__text {
		margin: 0;
		padding: 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		line-height: 1.5;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
</style>
