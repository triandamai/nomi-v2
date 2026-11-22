<!-- A board: Backlog, Ready to dev with a card Koda is working on and one that got stuck, and Done. -->
<script lang="ts">
	import IssueBoard from '$lib/ds/projects/IssueBoard.svelte';

	const labels = { board: 'Issue board', empty: 'No issues here', moveTo: (number: number) => `Move #${number} to`, retry: 'Try again', openChat: 'Open its chat' };
	const columns = [
		{ id: 'backlog', name: 'Backlog', cards: [{ number: 14, title: 'Show pickup slots after 11:00 on weekends', tags: 'Bug · High · 3 pts', status: { text: 'Overdue since 18 Nov', tone: 'late' as const } }] },
		{
			id: 'ready',
			name: 'Ready to dev',
			hint: 'The crew picks these up in order.',
			cards: [
				{ number: 12, title: 'Add QRIS receipt upload to checkout', tags: 'Task · Medium', status: { text: 'Koda is working on it', crew: 'koda' as const }, chatHref: '#' },
				{ number: 11, title: 'Send the booking ticket on WhatsApp', tags: 'Enhancement · Low', status: { text: 'Koda couldn’t finish it', tone: 'failed' as const, crew: 'koda' as const }, retry: true },
			],
		},
		{ id: 'done', name: 'Done', hint: 'Checked and finished. Their chats are read-only.', cards: [] },
	];
</script>

<IssueBoard {columns} {labels} onmove={() => {}} onretry={() => {}} />
