<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { NAME_MAX_CHARS, ROLE_MAX_CHARS, SOUL_MAX_CHARS } from '$lib/crewSettings';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const errorText = $derived.by(() => {
		if (form && 'invalid' in form) {
			return form.invalid === 'name' ? m.crewnew_invalid_name() : form.invalid === 'role' ? m.crewnew_invalid_role() : m.crewnew_invalid_soul();
		}
		if (form && 'failed' in form) return form.failed === 'taken' ? m.crewnew_taken() : m.crewnew_failed();
		return null;
	});
</script>

<div class="new-page">
	<div class="new-page__inner">
		<a class="new-page__back" href="/crew">← {m.crewmember_back()}</a>
		<header>
			<h1 class="md-headline-medium new-page__title">{m.crewnew_title()}</h1>
			<p class="new-page__lede">{m.crewnew_lede()}</p>
		</header>
		{#if errorText}<p class="new-page__error" role="alert">{errorText}</p>{/if}

		{#if data.templates.length > 0}
			<section aria-labelledby="templates-title">
				<h2 id="templates-title" class="new-page__section">{m.crewnew_templates()}</h2>
				<ul class="templates">
					{#each data.templates as template (template.key)}
						<li class="template">
							<AgentShape agent={template.key} size={40} face />
							<div class="template__text">
								<span class="template__name">{template.name}</span>
								<span class="template__role">{template.role}</span>
							</div>
							<form method="POST" use:enhance>
								<input type="hidden" name="template" value={template.key} />
								<Button variant="tonal" size="s" type="submit">{m.crewnew_use_template()}</Button>
							</form>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		<section class="blank" aria-labelledby="blank-title">
			<h2 id="blank-title" class="new-page__section">{m.crewnew_blank()}</h2>
			<form method="POST" use:enhance class="blank__form">
				<TextField label={m.crewnew_name()} name="name" value={form?.name ?? ''} maxlength={NAME_MAX_CHARS} required error={!!form && 'invalid' in form && form.invalid === 'name'} />
				<TextField label={m.crewnew_role()} name="role" value={form?.role ?? ''} maxlength={ROLE_MAX_CHARS} required error={!!form && 'invalid' in form && form.invalid === 'role'} />
				<label class="blank__soul">
					<span>{m.crewnew_soul()}</span>
					<textarea name="soul" rows="6" maxlength={SOUL_MAX_CHARS} value={form?.soul ?? ''}></textarea>
				</label>
				<div class="blank__actions">
					<Button variant="filled" size="s" type="submit">{m.crewnew_create()}</Button>
				</div>
			</form>
		</section>
	</div>
</div>

<style>
	.new-page {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(16px, 4vw, 44px) 44px;
		box-sizing: border-box;
		background: var(--nomi-color-ground);
	}
	.new-page__inner {
		display: flex;
		flex-direction: column;
		gap: 18px;
		max-width: 760px;
		margin: 0 auto;
	}
	.new-page__back {
		align-self: flex-start;
		min-height: 44px;
		display: inline-flex;
		align-items: center;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
	}
	.new-page__back:hover,
	.new-page__back:focus-visible {
		color: var(--md-sys-color-primary);
	}
	.new-page__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.new-page__lede {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.new-page__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.new-page__section {
		margin: 0 0 10px;
		font-size: 1.0625rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.templates {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
	.template {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
	}
	.template + .template {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.template__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 160px;
	}
	.template__name {
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.template__role {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.blank__form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 18px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.blank__soul {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.blank__soul textarea {
		box-sizing: border-box;
		width: 100%;
		padding: 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		resize: vertical;
	}
	.blank__soul textarea:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.blank__actions {
		display: flex;
		justify-content: flex-end;
	}
</style>
