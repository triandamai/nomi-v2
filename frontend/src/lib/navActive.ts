/** Whether the drawer row leading to `href` is the page at `path` (Chats covers each chat). */
export function isNavActive(path: string, href: string): boolean {
	if (href === '/') return path === '/';
	if (href === '/chats') return path === '/chats' || path.startsWith('/chat/');
	return path === href || path.startsWith(`${href}/`);
}
