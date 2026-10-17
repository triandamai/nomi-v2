<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import MorphingShape from '$lib/components/m3/MorphingShape.svelte';
	import type { HomeData } from './homeData';

	// "Afternoon, Ana." with what the crew has been up to underneath. `hero` centres it under
	// Nomi's morphing mark, for layouts that lead with it.

	let { home, size = 'large' }: { home: HomeData; size?: 'large' | 'hero' } = $props();

	const name = $derived(home.profile.display_name?.split(' ')[0] ?? null);

	// Greeting follows the user's stored timezone, not the server's, so SSR and the browser agree.
	const greeting = $derived.by(() => {
		let hour = new Date().getHours();
		try {
			hour = Number(
				new Intl.DateTimeFormat('en-US', { hour: 'numeric', hourCycle: 'h23', timeZone: home.preferences.timezone }).format(new Date()),
			);
		} catch {
			// Unknown timezone string — fall back to the runtime's local hour.
		}
		if (hour < 5) return m.home_still_up();
		if (hour < 12) return m.home_morning();
		if (hour < 18) return m.home_afternoon();
		return m.home_evening();
	});

	const activeCount = $derived(home.recentSessions.filter((s) => s.agent_active).length);
	const movedCount = $derived(home.summary?.while_you_were_out.length ?? 0);
	const subtitle = $derived(
		movedCount > 0
			? movedCount === 1
				? m.home_moved_one()
				: m.home_moved_many({ count: movedCount })
			: activeCount > 0
				? activeCount === 1
					? m.home_working_one()
					: m.home_working_many({ count: activeCount })
				: m.home_what_next(),
	);
</script>

<div class="greeting greeting--{size}">
	{#if size === 'hero'}
		<MorphingShape size={76} face sequence={['cookie9', 'sunny12', 'flower5', 'clover4']} stepMs={1600} class="greeting__mark" />
	{/if}
	<h1 class="md-display-large greeting__title">
		{name ? m.home_greeting_named({ greeting, name }) : m.home_greeting({ greeting })}
		<span class="greeting__sub">{subtitle}</span>
	</h1>
</div>

<style>
	.greeting {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.greeting--hero {
		align-items: center;
		text-align: center;
		padding-top: clamp(8px, 6vh, 56px);
	}
	.greeting__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.greeting--hero .greeting__title {
		font-size: clamp(2.5rem, 7vw, 4rem);
		line-height: 1.02;
		letter-spacing: -0.035em;
	}
	.greeting__sub {
		display: block;
		margin-top: 0.15em;
		font-size: 0.5em;
		line-height: 1.1;
		letter-spacing: -0.02em;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}
	.greeting--hero .greeting__sub {
		font-size: 0.4em;
		margin-top: 0.35em;
	}
</style>
