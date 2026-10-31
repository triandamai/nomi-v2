// The static copy's pages are files (components/button.html); the app's routes have no .html.
export const routeOf = (pathname: string) => pathname.replace(/\.html$/, '');
