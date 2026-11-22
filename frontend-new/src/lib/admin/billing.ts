import { m } from '$lib/paraglide/messages';
import type { AdminCoupon, PaymentStatus } from './api';
import { endOfDayUtc, utcInstant } from './format';

// Billing as the admin console writes it (admin_billing.rs): a term and a coupon from their forms,
// checked as the backend checks them so a mistake is said before anything is sent, and how a
// payment's money and state read.

/** Nomi's prices are in rupiah. */
export const CURRENCY = 'IDR';

/** The most a term or a percent coupon can take off. */
export const MAX_PERCENT = 100;

export const PAYMENT_STATUSES: PaymentStatus[] = ['pending', 'paid', 'expired', 'failed', 'cancelled'];

const STATUS_LABELS: Record<PaymentStatus, () => string> = {
	pending: m.admin_payment_pending,
	paid: m.admin_payment_paid,
	expired: m.admin_payment_expired,
	failed: m.admin_payment_failed_status,
	cancelled: m.admin_payment_cancelled,
};

export const paymentStatusLabel = (status: PaymentStatus): string => (STATUS_LABELS[status] ?? STATUS_LABELS.pending)();

/** Currencies priced in whole units, whatever the platform's locale data says. */
const WHOLE_UNITS = new Set(['IDR', 'JPY', 'KRW', 'VND']);

/** `1500000` as "Rp 1.500.000"; an unknown currency as the amount with its code. */
export function money(amount: number, currency: string, language: string): string {
	try {
		const whole = WHOLE_UNITS.has(currency.toUpperCase()) ? { minimumFractionDigits: 0, maximumFractionDigits: 0 } : {};
		return new Intl.NumberFormat(language, { style: 'currency', currency, currencyDisplay: 'narrowSymbol', ...whole }).format(amount);
	} catch {
		return `${new Intl.NumberFormat(language).format(amount)} ${currency}`;
	}
}

const text = (form: FormData, name: string): string => {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

/** A whole number typed in a box; null when it's empty or isn't one. */
const whole = (raw: string): number | null => (/^-?\d+$/.test(raw) ? Number(raw) : null);

export type TermInput = { months: number; discount_percent: number; label: string; is_active: boolean; sort_order: number };

/** The term from its form, or what's wrong with it. */
export function termFromForm(form: FormData): { ok: true; input: TermInput } | { ok: false; message: string } {
	const months = whole(text(form, 'months'));
	if (months === null || months < 1) return { ok: false, message: m.admin_term_months_invalid() };
	const percent = whole(text(form, 'discount_percent') || '0');
	if (percent === null || percent < 0 || percent > MAX_PERCENT) return { ok: false, message: m.admin_term_percent_invalid() };
	const order = text(form, 'sort_order');
	if (order && whole(order) === null) return { ok: false, message: m.admin_plan_order_invalid() };
	return { ok: true, input: { months, discount_percent: percent, label: text(form, 'label'), is_active: form.get('is_active') === 'on', sort_order: order ? Number(order) : 0 } };
}

export type CouponInput = Omit<AdminCoupon, 'id' | 'redemptions'>;

/** The coupon from its form, or what's wrong with it. Days are UTC; a coupon ends at the close of its last day. */
export function couponFromForm(form: FormData): { ok: true; input: CouponInput } | { ok: false; message: string } {
	const code = text(form, 'code').toUpperCase();
	if (!code || /\s/.test(code)) return { ok: false, message: m.admin_coupon_code_invalid() };
	const kind = text(form, 'discount_kind') === 'amount' ? 'amount' : 'percent';
	const value = whole(text(form, 'discount_value').replace(/[.,\s]/g, ''));
	if (value === null || value < 1 || (kind === 'percent' && value > MAX_PERCENT)) return { ok: false, message: m.admin_coupon_value_invalid() };
	const capText = text(form, 'max_redemptions');
	const cap = capText ? whole(capText) : null;
	const perUser = whole(text(form, 'per_user_limit') || '1');
	const minMonths = whole(text(form, 'min_months') || '1');
	if ((capText && (cap === null || cap < 1)) || perUser === null || perUser < 1 || minMonths === null || minMonths < 1) return { ok: false, message: m.admin_coupon_limits_invalid() };
	const startText = text(form, 'starts_at');
	const endText = text(form, 'ends_at');
	const startsAt = startText ? utcInstant(startText, '00:00') : null;
	const endsAt = endText ? endOfDayUtc(endText) : null;
	if ((startText && !startsAt) || (endText && !endsAt) || (startsAt && endsAt && endsAt < startsAt)) return { ok: false, message: m.admin_coupon_dates_invalid() };
	return {
		ok: true,
		input: {
			code,
			description: text(form, 'description') || null,
			discount_kind: kind,
			discount_value: value,
			max_redemptions: cap,
			per_user_limit: perUser,
			starts_at: startsAt,
			ends_at: endsAt,
			plan_ids: form.getAll('plan_ids').filter((id): id is string => typeof id === 'string' && id.length > 0),
			min_months: minMonths,
			first_payment_only: form.get('first_payment_only') === 'on',
			is_active: form.get('is_active') === 'on',
		},
	};
}
