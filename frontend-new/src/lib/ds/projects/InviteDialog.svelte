<script lang="ts" module>
	export type InviteRole = 'member' | 'admin';
</script>

<script lang="ts">
	import { Link, Send, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / Invite Dialog (RtsT2): inviting people to a workspace.
	// 520 wide. The addresses as mono chips (24 tall, surface-2, a × to remove) in one field on the
	// page colour (38 tall, 8 in; accent once it holds an address or has focus), then the two roles
	// side by side as a radio group (58 tall cards: the chosen one filled with a text-2 stroke and a
	// solid dot). Enter, a comma, a space or leaving the field turns what's typed into a chip;
	// Backspace in an empty field takes the last one back. Copy invite link is ghost, Send the
	// inverse main action, counting who it goes to.
	type Props = {
		open?: boolean;
		inline?: boolean;
		workspace: string;
		emails?: string[];
		role?: InviteRole;
		oninvite: (emails: string[], role: InviteRole) => void;
		oncopylink?: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), inline = false, workspace, emails = $bindable([]), role = $bindable('member'), oninvite, oncopylink, onclose }: Props = $props();

	let draft = $state('');
	let field: HTMLInputElement | undefined = $state();
	let radios: HTMLButtonElement[] = $state([]);

	const roles = $derived([
		{ value: 'member' as const, label: m.workspace_role_member(), body: m.projects_role_member_body() },
		{ value: 'admin' as const, label: m.workspace_role_admin(), body: m.projects_role_admin_body() },
	]);

	function commit() {
		const found = draft
			.split(/[\s,;]+/)
			.map((part) => part.trim())
			.filter((part) => /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(part) && !emails.includes(part));
		if (found.length) emails = [...emails, ...found];
		draft = draft
			.split(/[\s,;]+/)
			.filter((part) => part && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(part))
			.join(' ');
	}

	function oninput() {
		if (/[\s,;]$/.test(draft)) commit();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.isComposing) return;
		if (event.key === 'Enter') {
			event.preventDefault();
			commit();
		} else if (event.key === 'Backspace' && !draft && emails.length) {
			emails = emails.slice(0, -1);
		}
	}

	function remove(email: string) {
		emails = emails.filter((item) => item !== email);
		field?.focus();
	}

	function onrolekey(event: KeyboardEvent) {
		const step = event.key === 'ArrowRight' || event.key === 'ArrowDown' ? 1 : event.key === 'ArrowLeft' || event.key === 'ArrowUp' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const at = roles.findIndex((item) => item.value === role);
		const next = (at + step + roles.length) % roles.length;
		role = roles[next].value;
		radios[next]?.focus();
	}

	function send() {
		commit();
		if (!emails.length) return;
		open = false;
		oninvite(emails, role);
	}
</script>

<ProjectDialog bind:open {inline} {onclose} width={520} title={m.projects_invite_title({ workspace })} description={m.projects_invite_body()}>
	<div class="flex flex-col gap-3">
		<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
		<div
			onclick={() => field?.focus()}
			class={[
				'flex min-h-9.5 cursor-text flex-wrap items-center gap-1.5 rounded-sm border bg-bg px-1.75 py-1.5 focus-within:border-accent',
				emails.length ? 'border-accent' : 'border-border-strong',
			]}
		>
			{#each emails as email (email)}
				<span class="inline-flex h-6 items-center gap-1 rounded-xs bg-surface-2 pr-1 pl-2">
					<span class="font-mono text-label text-text">{email}</span>
					<button type="button" aria-label={m.projects_invite_remove({ email })} onclick={() => remove(email)} class="inline-flex cursor-pointer rounded-xs text-text-3 hover:text-text">
						<Icon icon={X} size={12} />
					</button>
				</span>
			{/each}
			<input
				bind:this={field}
				bind:value={draft}
				{oninput}
				{onkeydown}
				onblur={commit}
				type="text"
				inputmode="email"
				autocomplete="off"
				aria-label={m.projects_invite_emails()}
				placeholder={m.projects_invite_more()}
				class="min-w-24 flex-1 bg-transparent text-ui text-text caret-accent outline-none placeholder:text-text-3"
			/>
		</div>
		<div role="radiogroup" aria-label={m.projects_invite_role()} tabindex="-1" onkeydown={onrolekey} class="flex gap-2.5">
			{#each roles as item, index (item.value)}
				{@const checked = item.value === role}
				<button
					bind:this={radios[index]}
					type="button"
					role="radio"
					aria-checked={checked}
					tabindex={checked ? 0 : -1}
					onclick={() => (role = item.value)}
					class={[
						'flex h-14.5 min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-sm border px-2.75 py-2.25 text-left outline-offset-1',
						checked ? 'border-text-2 bg-surface-2' : 'border-border hover:bg-surface-2',
					]}
				>
					<span aria-hidden="true" class={['size-3.5 shrink-0 rounded-full', checked ? 'bg-text' : 'border border-border-strong bg-bg']}></span>
					<span class="flex min-w-0 flex-1 flex-col gap-px">
						<span class="truncate text-ui font-medium text-text">{item.label}</span>
						<span class="truncate text-caption text-text-3">{item.body}</span>
					</span>
				</button>
			{/each}
		</div>
	</div>
	{#snippet actions()}
		{#if oncopylink}<Button variant="ghost" label={m.projects_copy_link()} iconStart={Link} onclick={oncopylink} />{/if}
		<Button
			variant="inverse"
			label={emails.length > 1 ? m.projects_send_invites({ count: emails.length }) : m.projects_send_invite()}
			iconEnd={Send}
			disabled={!emails.length && !draft.trim()}
			onclick={send}
		/>
	{/snippet}
</ProjectDialog>
