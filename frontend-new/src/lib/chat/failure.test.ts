import { describe, expect, it } from 'vitest';
import { failureKind, failureOf } from './failure';

// The backend explains a failed turn in a sentence (nomi-i18n/locales/{en,id}.yml, error.*) and
// nothing else: these are those sentences, word for word.
const EN = {
	no_credits: "I couldn't reply because your DeepSeek account is out of credits for this request. Add credits or raise the API key's limit with DeepSeek, then send your message again.",
	bad_key: "I couldn't reply because Anthropic rejected the API key. Check the key in Settings, then try again.",
	rate_limited: 'DeepSeek is limiting how many requests I can make right now. Wait a minute, then try again.',
	too_long: 'This chat has grown too long for the model. Start a new chat to keep going.',
	no_model: "DeepSeek couldn't find the selected model. Pick another model in Settings.",
	provider_down: 'DeepSeek is having trouble right now. Try again in a moment.',
	unreachable: "I couldn't reach the AI provider. Check the server's connection, then send your message again.",
	unreadable: "The AI provider sent back a reply I couldn't read. Send your message again; if it keeps happening, try another model in Settings.",
	timed_out: 'That took too long, so I stopped. Please try again.',
	quota: "You've used this month's allowance on Nomi's models. Add your own API key in Models to keep going, or ask for a bigger plan in Billing & usage.",
	other: 'Something went wrong talking to the AI provider. Send your message again.',
};
const ID = {
	no_credits: 'Maaf, aku belum bisa membalas karena kredit akun DeepSeek-mu habis untuk permintaan ini. Silakan tambah kredit atau naikkan batas API key di DeepSeek, lalu kirim pesanmu lagi.',
	bad_key: 'Maaf, aku belum bisa membalas karena Anthropic menolak API key-nya. Silakan periksa key-nya di Pengaturan, lalu coba lagi.',
	rate_limited: 'DeepSeek sedang membatasi jumlah permintaanku. Tunggu sebentar, lalu coba lagi, ya.',
	too_long: 'Obrolan ini sudah terlalu panjang untuk modelnya. Yuk, lanjutkan di obrolan baru.',
	no_model: 'DeepSeek tidak menemukan model yang dipilih. Silakan pilih model lain di Pengaturan.',
	provider_down: 'DeepSeek sedang mengalami gangguan. Silakan coba lagi sebentar lagi, ya.',
	unreachable: 'Maaf, aku belum bisa menghubungi penyedia AI. Silakan periksa koneksi server, lalu kirim pesanmu lagi.',
	timed_out: 'Yang tadi kelamaan, jadi aku hentikan dulu. Coba kirim lagi, ya.',
	quota: 'Jatah bulan ini untuk model Nomi sudah habis. Tambahkan API key milikmu di Model agar bisa lanjut, atau minta paket yang lebih besar di Tagihan & pemakaian.',
	other: 'Maaf, ada kendala saat menghubungi penyedia AI. Silakan kirim pesanmu lagi.',
};

describe('failureKind', () => {
	it.each(Object.entries(EN))('reads %s from the English explanation', (kind, text) => {
		expect(failureKind(text)).toBe(kind);
	});

	it.each(Object.entries(ID))('reads %s from the Indonesian explanation', (kind, text) => {
		expect(failureKind(text)).toBe(kind);
	});

	it('calls anything else a failure of its own', () => {
		expect(failureKind('I went around in circles on that one and stopped.')).toBe('other');
	});
});

const message = (blocks: unknown[] | null, content = EN.provider_down) => ({ content, content_blocks: blocks as never });

describe('failureOf', () => {
	it('reads an error bubble, and whether Try again can rerun it', () => {
		expect(failureOf(message([{ kind: 'run_failed', retry: { target: 'turn', turn_job_id: 't' } }]))).toEqual({ kind: 'provider_down', text: EN.provider_down, retry: true });
		expect(failureOf(message([{ kind: 'run_failed' }], EN.timed_out))).toEqual({ kind: 'timed_out', text: EN.timed_out, retry: false });
	});

	it('reads the allowance running out from its card, whatever the words', () => {
		expect(failureOf(message([{ kind: 'quota_notice', reason: 'used_up' }], 'anything'))).toEqual({ kind: 'quota', text: 'anything', retry: false });
	});

	it('is nothing for an ordinary reply', () => {
		expect(failureOf(message(null, 'Hello'))).toBeNull();
		expect(failureOf(message([{ kind: 'reasoning', text: 'x' }]))).toBeNull();
	});
});
