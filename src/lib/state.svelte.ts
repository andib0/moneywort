import { api } from './api';
import { thisMonth } from './fmt';
import type { Account, Budget, Category, Recurring, Tx } from './types';

export type Tab = 'home' | 'activity' | 'insights' | 'plan';

export type Sheet =
  | { type: 'tx'; tx?: Tx; kind?: Tx['kind'] }
  | { type: 'settings' }
  | { type: 'account'; account?: Account }
  | { type: 'category'; category?: Category; kind?: Category['kind'] }
  | { type: 'budget'; categoryId?: number }
  | { type: 'recurring'; rule?: Recurring }
  | { type: 'confirm'; title: string; body: string; action: string; danger?: boolean; onyes: () => void };

export const app = $state({
  ready: false,
  error: '',
  tab: 'home' as Tab,
  month: thisMonth(),
  accounts: [] as Account[],
  categories: [] as Category[],
  budgets: [] as Budget[],
  recurring: [] as Recurring[],
  settings: { currency: 'EUR', theme: 'system' } as Record<string, string>,
  /** Bumped after every write so screens refetch. */
  version: 0,
  sheets: [] as Sheet[],
  toast: '',
});

export const currency = () => app.settings.currency || 'EUR';

export function category(id: number | null | undefined): Category | undefined {
  return id == null ? undefined : app.categories.find((c) => c.id === id);
}

export function account(id: number | null | undefined): Account | undefined {
  return id == null ? undefined : app.accounts.find((a) => a.id === id);
}

export async function load() {
  try {
    const b = await api.bootstrap();
    app.accounts = b.accounts;
    app.categories = b.categories;
    app.budgets = b.budgets;
    app.recurring = b.recurring;
    app.settings = { currency: 'EUR', theme: 'system', ...b.settings };
    app.ready = true;
    app.version++;
    if (b.posted_recurring > 0) {
      toast(`Added ${b.posted_recurring} recurring ${b.posted_recurring === 1 ? 'entry' : 'entries'}`);
    }
  } catch (e) {
    app.error = String(e);
  }
}

/** Run a write, then reload reference data and notify screens. */
export async function mutate<T>(fn: () => Promise<T>, done?: string): Promise<{ value: T } | null> {
  try {
    const r = await fn();
    const b = await api.bootstrap();
    app.accounts = b.accounts;
    app.categories = b.categories;
    app.budgets = b.budgets;
    app.recurring = b.recurring;
    app.settings = { currency: 'EUR', theme: 'system', ...b.settings };
    app.version++;
    if (done) toast(done);
    return { value: r };
  } catch (e) {
    toast(String(e));
    return null;
  }
}

let toastTimer: ReturnType<typeof setTimeout> | undefined;
export function toast(msg: string) {
  app.toast = msg;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (app.toast = ''), 2600);
}

// ----- sheet stack, wired to history so Android back closes the top sheet -----

export function openSheet(s: Sheet) {
  app.sheets.push(s);
  history.pushState({ sheet: app.sheets.length }, '');
}

let popWaiters: (() => void)[] = [];

/** Closes the top sheet. Resolves once the history pop has been handled. */
export function closeSheet(): Promise<void> {
  if (!app.sheets.length) return Promise.resolve();
  return new Promise((resolve) => {
    popWaiters.push(resolve);
    history.back();
  });
}

export function onPopState() {
  const waiters = popWaiters;
  popWaiters = [];
  queueMicrotask(() => waiters.forEach((w) => w()));
  if (app.sheets.length) app.sheets.pop();
  else if (app.tab !== 'home') {
    app.tab = 'home';
  }
}

export function setTab(t: Tab) {
  if (t === app.tab) return;
  if (app.tab === 'home' && t !== 'home') history.pushState({ tab: t }, '');
  else if (t === 'home') {
    history.back();
    return;
  }
  app.tab = t;
}

export function confirmAction(title: string, body: string, action: string, onyes: () => void, danger = true) {
  openSheet({ type: 'confirm', title, body, action, danger, onyes });
}
