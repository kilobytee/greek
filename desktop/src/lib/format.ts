export function relativeTime(timestampMs: number, now = Date.now()): string {
	const diff = Math.max(0, now - timestampMs);
	const seconds = Math.floor(diff / 1000);
	if (seconds < 60) return 'ahora';
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h`;
	const days = Math.floor(hours / 24);
	if (days < 7) return `${days}d`;
	const weeks = Math.floor(days / 7);
	if (weeks < 5) return `${weeks}sem`;
	const months = Math.floor(days / 30);
	if (months < 12) return `${months}mes`;
	const years = Math.floor(days / 365);
	return `${years}a`;
}

export function shortModelName(model: string): string {
	if (model === 'auto') return 'Auto';
	if (model === 'greek-local') return 'Local';
	const tail = model.split('/').pop() ?? model;
	return tail.replace(/:free$/, '').replace(/-free$/, '');
}

export function copyText(value: string): Promise<void> {
	if (navigator.clipboard?.writeText) return navigator.clipboard.writeText(value);
	return Promise.reject(new Error('El portapapeles no está disponible.'));
}

export function prefersReducedMotion(): boolean {
	return (
		typeof window !== 'undefined' &&
		window.matchMedia?.('(prefers-reduced-motion: reduce)').matches === true
	);
}
