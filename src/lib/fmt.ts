const fmtCache = new Map<string, Intl.NumberFormat>();

function nf(currency: string, compact = false): Intl.NumberFormat {
  const key = currency + (compact ? ':c' : '');
  let f = fmtCache.get(key);
  if (!f) {
    try {
      f = new Intl.NumberFormat(undefined, {
        style: 'currency',
        currency,
        ...(compact ? { notation: 'compact', maximumFractionDigits: 1 } : {}),
      });
    } catch {
      f = new Intl.NumberFormat(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    }
    fmtCache.set(key, f);
  }
  return f;
}

/** Format integer cents as currency. */
export function money(cents: number, currency: string, opts: { sign?: boolean; compact?: boolean } = {}): string {
  const s = nf(currency, opts.compact).format(Math.abs(cents) / 100);
  if (cents < 0) return '−' + s;
  if (opts.sign && cents > 0) return '+' + s;
  return s;
}

/** Split formatted money into whole/fraction parts for big displays. */
export function moneyParts(cents: number, currency: string): { main: string; frac: string } {
  const parts = nf(currency).formatToParts(Math.abs(cents) / 100);
  let main = '';
  let frac = '';
  let afterDecimal = false;
  for (const p of parts) {
    if (p.type === 'decimal') afterDecimal = true;
    if (afterDecimal && (p.type === 'decimal' || p.type === 'fraction')) frac += p.value;
    else main += p.value;
  }
  return { main: (cents < 0 ? '−' : '') + main.trim(), frac };
}

/** "12.5" -> 1250 */
export function toCents(s: string): number {
  if (!s) return 0;
  const [w, f = ''] = s.split('.');
  return parseInt(w || '0', 10) * 100 + parseInt((f + '00').slice(0, 2), 10);
}

/** 1250 -> "12.5" (keypad string) */
export function fromCents(c: number): string {
  if (!c) return '';
  const w = Math.floor(c / 100);
  const f = c % 100;
  if (!f) return String(w);
  return `${w}.${String(f).padStart(2, '0').replace(/0$/, '')}`;
}

export function iso(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

export const todayISO = () => iso(new Date());
export const thisMonth = () => todayISO().slice(0, 7);

export function parseISO(s: string): Date {
  const [y, m, d] = s.split('-').map(Number);
  return new Date(y, m - 1, d || 1);
}

export function addMonths(month: string, n: number): string {
  const d = parseISO(month + '-01');
  d.setMonth(d.getMonth() + n);
  return iso(d).slice(0, 7);
}

export function monthName(month: string, style: 'long' | 'short' = 'long'): string {
  const d = parseISO(month + '-01');
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleDateString(undefined, {
    month: style,
    ...(sameYear || style === 'short' ? {} : { year: 'numeric' }),
  });
}

export function dayLabel(date: string): string {
  const t = todayISO();
  if (date === t) return 'Today';
  const y = new Date();
  y.setDate(y.getDate() - 1);
  if (date === iso(y)) return 'Yesterday';
  const d = parseISO(date);
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleDateString(undefined, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    ...(sameYear ? {} : { year: 'numeric' }),
  });
}

export function shortDate(date: string): string {
  return parseISO(date).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });
}

export function daysInMonth(month: string): number {
  const d = parseISO(month + '-01');
  return new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
}

export function pct(part: number, whole: number): number {
  return whole > 0 ? Math.round((part / whole) * 100) : 0;
}
