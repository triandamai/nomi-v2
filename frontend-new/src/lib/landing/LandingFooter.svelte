<script lang="ts" module>
	export type FooterColumn = { title: string; links: { href: string; label: string }[] };
	export type FooterLanguage = { locale: string; label: string; current: boolean };
</script>

<script lang="ts">
	import logo from './assets/logo-lockup.svg?url';

	// Figma, Landing v2 → Footer, from the page's render (its layers weren't read: the Figma MCP's
	// limit): 72 above, 64 at the sides, 32 below. The lockup (28 high) and the line under it, 12
	// apart; the columns 64 apart, each its title (14 / 18, 500) and its links (14 / 20), 12 apart; a
	// 1px rule 40 under the tallest column; then, 40 under it, the year and maker in mono and the
	// languages. Only links with a page behind them are here (docs/ux/work/landing): the design's
	// Harga, Template, Panduan, Status and the Nomi Labs column (Tentang kami, Privasi, Ketentuan)
	// wait for their pages. The languages switch this page's language.
	type Props = { home: string; homeHref: string; navLabel: string; tagline: string; columns: FooterColumn[]; legal: string; languageLabel: string; languages: FooterLanguage[]; onlanguage: (locale: string) => void };
	let { home, homeHref, navLabel, tagline, columns, legal, languageLabel, languages, onlanguage }: Props = $props();
</script>

<footer class="footer">
	<div class="top">
		<div class="brand">
			<a class="logo" href={homeHref} aria-label={home}><img src={logo} alt="" width="111" height="28" /></a>
			<p>{tagline}</p>
		</div>
		<nav class="columns" aria-label={navLabel}>
			{#each columns as column (column.title)}
				<div class="column">
					<h2>{column.title}</h2>
					<ul>
						{#each column.links as link (link.href)}<li><a href={link.href}>{link.label}</a></li>{/each}
					</ul>
				</div>
			{/each}
		</nav>
	</div>
	<div class="bottom">
		<span class="legal">{legal}</span>
		<span class="languages" role="group" aria-label={languageLabel}>
			{#each languages as language, index (language.locale)}
				{#if index > 0}<span aria-hidden="true">·</span>{/if}
				<button type="button" class="language" aria-pressed={language.current} onclick={() => onlanguage(language.locale)}>{language.label}</button>
			{/each}
		</span>
	</div>
</footer>

<style>
	.footer {
		max-width: 1440px;
		margin: 0 auto;
		padding: 72px 64px 32px;
	}

	.top {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 48px;
		padding-bottom: 40px;
		border-bottom: 1px solid var(--lp-border-subtle);
	}

	.brand {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.logo {
		display: flex;
		width: fit-content;
		border-radius: 4px;
	}

	.logo img {
		display: block;
		width: 110.83px;
		height: 28px;
	}

	.brand p {
		margin: 0;
		font-size: 16px;
		line-height: 24px;
		letter-spacing: -0.16px;
		color: var(--lp-text-2);
	}

	.columns {
		display: flex;
		gap: 64px;
	}

	.column {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	h2 {
		margin: 0;
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
	}

	ul {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	a {
		color: var(--lp-text-2);
		text-decoration: none;
	}

	.column a {
		font-size: 14px;
		line-height: 20px;
		letter-spacing: -0.14px;
	}

	.column a:hover,
	.language:hover {
		color: var(--lp-text);
	}

	a:focus-visible,
	.language:focus-visible {
		outline: 2px solid var(--lp-focus);
		outline-offset: 2px;
		border-radius: 4px;
	}

	.bottom {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding-top: 40px;
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-3);
	}

	.legal {
		text-wrap: pretty;
	}

	.languages {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.language {
		padding: 0;
		border: 0;
		background: none;
		font: inherit;
		color: var(--lp-text-3);
		cursor: pointer;
	}

	.language[aria-pressed='true'] {
		color: var(--lp-text-2);
	}

	@media (max-width: 767px) {
		.footer {
			padding: 56px 16px 32px;
		}

		.top,
		.bottom {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
