<!-- Needs you: an unread notification with its actions, and a reminder that went off. -->
<script lang="ts">
	import { AlarmClockPlus, ArrowRight, Check } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import InboxCard from '$lib/ds/inbox/InboxCard.svelte';
</script>

<div class="flex w-full max-w-215 flex-col gap-2.5">
	<InboxCard time="18 Oct" agent="nomi" title="Trian invited you to Trian’s space" tag="account" caption="Work on its 4 projects together with the crew. Your own chats and memory stay private." note="expires in 4 days" unread>
		{#snippet actions()}
			<Button label="Open" variant="inverse" iconEnd={ArrowRight} />
			<Button label="Mark read" variant="ghost" iconStart={Check} />
		{/snippet}
	</InboxCard>
	<InboxCard time="07:40" agent="kala" title="Netflix renews tomorrow · Rp 186.000" tag="reminder · once" caption="From BCA ending 4417.">
		{#snippet actions()}
			<Button label="Done" variant="inverse" iconStart={Check} />
			<Button label="Snooze 1 hour" variant="ghost" iconStart={AlarmClockPlus} />
		{/snippet}
	</InboxCard>
</div>
