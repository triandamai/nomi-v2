import { describe, expect, it } from 'vitest';
import { loadingPage, opensAnotherPage } from './loading';

describe('loadingPage', () => {
	it('names the page on its way by where it goes', () => {
		expect(loadingPage('/')).toBe('home');
		expect(loadingPage('/chat/c1')).toBe('chat');
		expect(loadingPage('/chat/new')).toBe('chat');
		expect(loadingPage('/chats')).toBe('chats');
		expect(loadingPage('/projects')).toBe('projects');
		expect(loadingPage('/projects/p1')).toBe('project');
		expect(loadingPage('/projects/p1/about')).toBe('project');
		expect(loadingPage('/inbox')).toBe('inbox');
		expect(loadingPage('/settings/general')).toBe('settings');
	});

	it('has no name for any other page', () => {
		expect(loadingPage('/search')).toBeNull();
		expect(loadingPage('/profile/budi')).toBeNull();
	});
});

describe('opensAnotherPage', () => {
	it('is true only for a different path', () => {
		expect(opensAnotherPage('/projects', '/projects/p1')).toBe(true);
		expect(opensAnotherPage('/projects', '/projects')).toBe(false);
		expect(opensAnotherPage('/projects', null)).toBe(false);
	});
});
