import { m } from '$lib/paraglide/messages';
import { endOfDayUtc } from './format';

// A plan as the admin form writes it, checked as the backend checks it (nomi_usage::plans::
// PlanInput::validate), so a mistake is said on the page before anything is sent.

/** The tones a plan's card can wear, in the backend's order (CARD_TONES). */
export const CARD_TONES = ['glow', 'ember', 'sky', 'tide', 'bloom', 'citrus', 'dusk', 'slate'] as const;

export type PlanInput = {
	slug: string;
	name: string;
	description: string;
	monthly_tokens: number;
	price_label: string;
	features: string[];
	card_tone: string;
	promo_label: string | null;
	promo_price_label: string | null;
	promo_ends_at: string | null;
	is_active: boolean;
	sort_order: number;
};

const text = (form: FormData, name: string): string => {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

/** The plan from the form, or what's wrong with it. */
export function planFromForm(form: FormData): { ok: true; input: PlanInput } | { ok: false; message: string } {
	const name = text(form, 'name');
	const slug = text(form, 'slug').toLowerCase();
	const tokens = text(form, 'monthly_tokens').replace(/[\s,._]/g, '');
	const tone = text(form, 'card_tone');
	const ends = text(form, 'promo_ends_at');
	const order = text(form, 'sort_order');
	if (!name) return { ok: false, message: m.admin_plan_name_required() };
	if (!/^[a-z0-9-]+$/.test(slug)) return { ok: false, message: m.admin_plan_slug_invalid() };
	if (!/^\d+$/.test(tokens) || Number(tokens) <= 0) return { ok: false, message: m.admin_plan_tokens_invalid() };
	if (!(CARD_TONES as readonly string[]).includes(tone)) return { ok: false, message: m.admin_plan_tone_invalid() };
	const promoEndsAt = ends ? endOfDayUtc(ends) : null;
	if (ends && !promoEndsAt) return { ok: false, message: m.admin_plan_promo_ends_invalid() };
	if (order && !/^-?\d+$/.test(order)) return { ok: false, message: m.admin_plan_order_invalid() };
	return {
		ok: true,
		input: {
			slug,
			name,
			description: text(form, 'description'),
			monthly_tokens: Number(tokens),
			price_label: text(form, 'price_label'),
			features: text(form, 'features')
				.split('\n')
				.map((line) => line.trim())
				.filter(Boolean),
			card_tone: tone,
			promo_label: text(form, 'promo_label') || null,
			promo_price_label: text(form, 'promo_price_label') || null,
			promo_ends_at: promoEndsAt,
			is_active: form.get('is_active') === 'on',
			sort_order: order ? Number(order) : 0,
		},
	};
}
