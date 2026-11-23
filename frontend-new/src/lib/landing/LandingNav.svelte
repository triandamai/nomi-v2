<script lang="ts">
	import logo from './assets/logo-lockup.svg?url';

	// Figma, Landing v2 → Nav (170:2767): the lockup, the page's sections, Sign in (ghost) and Try it
	// free (primary), 48 at the sides and 20 above and below. Labels are the design's: uppercase
	// buttons are the marketing system's (Label/Button), not the app's.
	// Waitlist and FAQ ("Tanya-tanya") wait for their sections, so they aren't linked yet.
	type Link = { href: string; label: string };
	type Props = { home: string; homeHref: string; label: string; links: Link[]; signIn: Link; tryFree: Link };
	let { home, homeHref, label, links, signIn, tryFree }: Props = $props();
</script>

<header class="nav">
	<a class="logo" href={homeHref} aria-label={home}><img src={logo} alt="" width="103" height="26" /></a>
	<nav aria-label={label} class="links">
		{#each links as link (link.href)}<a href={link.href}>{link.label}</a>{/each}
	</nav>
	<div class="actions">
		<a class="button ghost" href={signIn.href}>{signIn.label}</a>
		<a class="button primary" href={tryFree.href}>{tryFree.label}</a>
	</div>
</header>

<style>
	.nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 20px 48px;
	}

	.logo {
		display: flex;
		border-radius: 4px;
	}

	.logo img {
		display: block;
		width: 102.915px;
		height: 26px;
	}

	.links {
		display: flex;
		gap: 32px;
	}

	.links a {
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
		color: var(--lp-text-2);
		text-decoration: none;
		border-radius: 4px;
	}

	.links a:hover {
		color: var(--lp-text);
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		height: 40px;
		padding: 0 16px;
		border-radius: var(--lp-radius-sm);
		font-size: 14px;
		line-height: 16px;
		font-weight: 500;
		letter-spacing: 0.28px;
		text-transform: uppercase;
		text-decoration: none;
		white-space: nowrap;
	}

	.ghost {
		color: var(--lp-text);
	}

	.ghost:hover {
		background: var(--lp-bg-muted);
	}

	.primary {
		background: var(--lp-primary);
		color: var(--lp-primary-fg);
	}

	.primary:hover {
		opacity: 0.88;
	}

	a:focus-visible {
		outline: 2px solid var(--lp-focus);
		outline-offset: 2px;
	}

	@media (max-width: 899px) {
		.links {
			display: none;
		}
	}

	@media (max-width: 599px) {
		.nav {
			padding: 16px;
		}

		.ghost {
			display: none;
		}
	}
</style>
