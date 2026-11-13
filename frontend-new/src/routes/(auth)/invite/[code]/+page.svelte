<script lang="ts">
	// An invite, in the sign-in layout (the pen draws no invite page; built from the Sign in or
	// create frame). Signed out: make an account or sign in, and come back here. Signed in: what the
	// invite is for and from whom, and one action for each state the backend gives: open (join),
	// member (already in), expired, used, or sent to another address.
	import { enhance } from '$app/forms';
	import { ArrowRight } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import { withRedirect } from '$lib/auth/redirects';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();
	let busy = $state(false);

	const back = $derived(`/invite/${data.code}`);
	const invite = $derived(data.invite);
	// An accept that failed moves the invite to the state it's really in.
	const shown = $derived(form?.problem && form.problem !== 'failed' ? form.problem : invite?.state);

	const ROLES = { owner: m.role_owner, admin: m.role_admin, member: m.role_member };
	const ACCESS = { build: m.invite_access_build, prompt: m.invite_access_prompt, view: m.invite_access_view };
</script>

<svelte:head><title>{m.invite_page_title()}</title></svelte:head>

{#snippet heading(title: string, line: string)}
	<div class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium text-text">{title}</h1>
		<p class="text-body leading-normal text-text-2">{line}</p>
	</div>
{/snippet}

{#if !data.signedIn}
	{@render heading(m.invite_signed_out_title(), m.invite_signed_out_sub())}
	<div class="flex flex-col gap-3">
		<AuthButton label={m.invite_create()} href={withRedirect('/register', back)} iconEnd={ArrowRight} />
		<AuthButton label={m.invite_sign_in()} href={withRedirect('/sign-in', back)} variant="secondary" />
	</div>
{:else if data.missing}
	{@render heading(m.invite_missing_title(), m.invite_missing_sub())}
	<AuthButton label={m.invite_go_home()} href="/" variant="secondary" />
{:else if data.failed || !invite}
	<ErrorCallout title={m.invite_load_failed()} actions={[{ label: m.invite_go_home(), href: '/' }]} />
{:else if shown === 'open'}
	{#if invite.project}
		{@render heading(
			m.invite_project_title({ project: invite.project.name }),
			m.invite_project_sub({ inviter: invite.inviter, project: invite.project.name, org: invite.org_name, access: ACCESS[invite.project.role]() }),
		)}
	{:else}
		{@render heading(m.invite_open_title({ org: invite.org_name }), m.invite_open_sub({ inviter: invite.inviter, org: invite.org_name, role: ROLES[invite.role]() }))}
	{/if}
	<form
		method="POST"
		action="?/accept"
		class="flex flex-col gap-3"
		use:enhance={() => {
			busy = true;
			return async ({ update }) => {
				await update();
				busy = false;
			};
		}}
	>
		{#if form?.problem === 'failed'}<ErrorCallout title={m.invite_failed()} />{/if}
		{#if invite.project}<input type="hidden" name="project" value={invite.project.id} />{/if}
		<AuthButton
			label={invite.project ? m.invite_accept_project({ project: invite.project.name }) : m.invite_accept({ org: invite.org_name })}
			iconEnd={ArrowRight}
			loading={busy}
		/>
		<AuthButton label={m.invite_not_now()} href="/" variant="secondary" />
	</form>
{:else if shown === 'member'}
	{@render heading(m.invite_member_title({ org: invite.org_name }), m.invite_member_sub())}
	{#if invite.project}
		<AuthButton label={m.invite_open_project({ project: invite.project.name })} href="/projects/{invite.project.id}" iconEnd={ArrowRight} />
	{:else}
		<AuthButton label={m.invite_go_home()} href="/" iconEnd={ArrowRight} />
	{/if}
{:else if shown === 'expired'}
	{@render heading(m.invite_expired_title(), m.invite_expired_sub({ inviter: invite.inviter }))}
	<AuthButton label={m.invite_go_home()} href="/" variant="secondary" />
{:else if shown === 'used'}
	{@render heading(m.invite_used_title(), m.invite_used_sub({ inviter: invite.inviter }))}
	<AuthButton label={m.invite_go_home()} href="/" variant="secondary" />
{:else}
	{@render heading(m.invite_other_title(), m.invite_other_sub({ email: invite.email ?? '' }))}
	<AuthButton label={m.invite_go_home()} href="/" variant="secondary" />
{/if}
