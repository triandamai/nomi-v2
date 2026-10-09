<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let title = $state('');
	let body = $state('');
	let link = $state('');
	let audience = $state('all');
	let sending = $state(false);

	const audiences = $derived([
		{ value: 'all', label: m.broadcast_everyone() },
		...data.plans.map((plan) => ({ value: plan.id, label: m.broadcast_on_plan({ plan: plan.name }) })),
	]);
	const audienceName = (value: string) => audiences.find((a) => a.value === value)?.label ?? value;
	const when = (at: string) => new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(at));
</script>

<PageHeader title={m.admin_notifications()} lede={m.broadcast_lede()} agent="reminders" />

<div class="layout">
	<form
		method="POST"
		action="?/send"
		class="compose"
		use:enhance={() => {
			sending = true;
			return async ({ result, update }) => {
				sending = false;
				await update({ reset: false });
				if (result.type === 'success') {
					title = '';
					body = '';
					link = '';
				}
			};
		}}
	>
		<h2 class="compose__title">{m.broadcast_new()}</h2>
		<TextField id="b-title" name="title" label={m.broadcast_title()} bind:value={title} required maxlength={120} />
		<label class="field">
			<span class="field__label">{m.broadcast_body()}</span>
			<textarea name="body" rows="5" bind:value={body} required maxlength="2000"></textarea>
		</label>
		<TextField id="b-link" name="link" label={m.broadcast_link()} bind:value={link} placeholder="/billing" supportingText={m.broadcast_link_hint()} />
		<Select name="audience" label={m.broadcast_audience()} options={audiences} bind:value={audience} />
		<p class="compose__note">{m.broadcast_email_note()}</p>
		{#if form && 'error' in form && form.error}<p class="compose__error" role="alert">{form.error}</p>{/if}
		{#if form && 'sent' in form}<p class="compose__ok" role="status">{m.broadcast_sent({ count: form.sent ?? 0 })}</p>{/if}
		<div>
			<Button type="submit" variant="filled" disabled={sending || !title.trim() || !body.trim()}>{m.broadcast_send({ audience: audienceName(audience) })}</Button>
		</div>
	</form>

	<aside class="preview" aria-label={m.broadcast_preview()}>
		<p class="field__label">{m.broadcast_preview()}</p>
		<div class="preview__item">
			<AgentShape agent="reminders" size={40} />
			<span class="preview__text">
				<span class="preview__kind">{m.notif_kind_promo()}</span>
				<span class="preview__title">{title || m.broadcast_title()}</span>
				<span class="preview__body">{body || m.broadcast_body()}</span>
			</span>
		</div>
	</aside>
</div>

<section class="sent" aria-labelledby="sent-title">
	<h2 id="sent-title" class="compose__title">{m.broadcast_history()}</h2>
	{#if data.sent.length === 0}
		<p class="sent__empty">{m.broadcast_none()}</p>
	{:else}
		<ul class="sent__list">
			{#each data.sent as item (item.id)}
				<li class="sent__item">
					<span class="sent__head">
						<span class="sent__title">{item.title}</span>
						<time class="sent__when" datetime={item.created_at}>{when(item.created_at)}</time>
					</span>
					<span class="sent__body">{item.body}</span>
					<span class="sent__meta">{audienceName(item.audience)} · {m.broadcast_read_of({ read: item.read, total: item.recipients })}</span>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 20px;
	}
	@media (min-width: 960px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) 320px;
		}
	}
	.compose,
	.sent {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.sent {
		margin-top: 20px;
	}
	.compose__title {
		margin: 0;
		font-family: var(--md-sys-typescale-title-medium-font);
		font-size: var(--md-sys-typescale-title-medium-size);
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field__label {
		margin: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.field textarea {
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.compose__note {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.compose__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.compose__ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.preview {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.preview__item {
		display: flex;
		gap: 14px;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.preview__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.preview__kind {
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--md-sys-color-on-surface-variant);
	}
	.preview__title {
		font-weight: 800;
	}
	.preview__body {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		white-space: pre-line;
		overflow-wrap: anywhere;
	}
	.sent__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sent__list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.sent__item {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.sent__head {
		display: flex;
		justify-content: space-between;
		gap: 12px;
	}
	.sent__title {
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.sent__when,
	.sent__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sent__body {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
		white-space: pre-line;
	}
</style>
