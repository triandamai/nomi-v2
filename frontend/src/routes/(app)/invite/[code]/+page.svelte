<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { OrgRole } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const ROLE: Record<OrgRole, () => string> = {
		owner: m.org_role_owner,
		admin: m.org_role_admin,
		member: m.org_role_member,
	};
	let busy = $state(false);
</script>

<div class="invite">
	<section class="invite__card">
		<AgentShape agent="nomi" face size={56} />
		{#if !data.invite}
			<p class="invite__lede">{m.invite_not_found()}</p>
			<Button href="/projects" variant="tonal" size="m">{m.invite_open_projects()}</Button>
		{:else}
			{@const invite = data.invite}
			<h1 class="invite__title">{m.invite_title({ org: invite.org_name })}</h1>
			{#if invite.state === 'open'}
				<p class="invite__lede">{m.invite_lede({ inviter: invite.inviter, org: invite.org_name, role: ROLE[invite.role]().toLowerCase() })}</p>
				{#if form?.error}<p class="invite__error" role="alert">{form.error}</p>{/if}
				<form
					method="POST"
					action="?/accept"
					class="invite__actions"
					use:enhance={() => {
						busy = true;
						return async ({ update }) => {
							await update();
							busy = false;
						};
					}}
				>
					<Button type="submit" variant="filled" size="m" disabled={busy}>{m.invite_accept({ org: invite.org_name })}</Button>
					<Button href="/" variant="text" size="m">{m.invite_decline()}</Button>
				</form>
			{:else}
				<p class="invite__lede">
					{#if invite.state === 'member'}{m.invite_state_member({ org: invite.org_name })}
					{:else if invite.state === 'used'}{m.invite_state_used({ inviter: invite.inviter })}
					{:else if invite.state === 'expired'}{m.invite_state_expired({ inviter: invite.inviter })}
					{:else}{m.invite_state_other_address({ email: invite.email ?? '' })}{/if}
				</p>
				<Button href="/projects" variant="tonal" size="m">{m.invite_open_projects()}</Button>
			{/if}
		{/if}
	</section>
</div>

<style>
	.invite {
		display: grid;
		place-items: center;
		height: 100%;
		overflow-y: auto;
		padding: 32px 16px;
		box-sizing: border-box;
	}
	.invite__card {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 16px;
		width: 100%;
		max-width: 480px;
		padding: 28px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
	}
	.invite__title {
		margin: 0;
		font-family: var(--md-sys-typescale-headline-small-font);
		font-size: var(--md-sys-typescale-headline-small-size);
		line-height: 1.2;
		color: var(--md-sys-color-on-surface);
	}
	.invite__lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.invite__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.invite__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
</style>
