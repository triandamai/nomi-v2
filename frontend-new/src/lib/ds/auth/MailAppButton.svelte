<script lang="ts">
	import { ArrowUpRight, Inbox, Mail } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Auth / Enter code → Open Mail: a shortcut to the person's webmail, 40 high and bordered, the
	// label in regular weight with the app's icon and the ↗ of a link that leaves Nomi. Offered only
	// for an address that app serves; two side by side share the row.
	type Props = { app: 'gmail' | 'outlook' };
	let { app }: Props = $props();

	const APPS = {
		gmail: { href: 'https://mail.google.com/mail/u/0/#inbox', icon: Mail, label: () => m.mail_open_gmail() },
		outlook: { href: 'https://outlook.live.com/mail/0/', icon: Inbox, label: () => m.mail_open_outlook() },
	};
	const shown = $derived(APPS[app]);
</script>

<a
	href={shown.href}
	target="_blank"
	rel="noopener noreferrer"
	class="flex h-10 min-w-0 flex-1 items-center justify-center gap-2 rounded-sm border border-border-strong text-ui text-text outline-offset-0 transition-colors hover:bg-surface-2 focus-visible:border-transparent focus-visible:outline-3 max-sm:h-11"
>
	<span class="inline-flex text-text-2"><Icon icon={shown.icon} size={14} /></span>
	<span>{shown.label()}</span>
	<span class="inline-flex text-text-3"><Icon icon={ArrowUpRight} size={12} /></span>
</a>
