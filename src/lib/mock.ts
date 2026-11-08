// In-browser stand-in for the Rust backend so the UI can be previewed without a device.
// Mirrors moneywort-core's behaviour closely enough for design work; the APK never loads this.
import type { Account, Budget, Category, Recurring, Summary, Tx, TxFilter } from './types';
import { addMonths, iso, parseISO, todayISO } from './fmt';

interface Store {
  accounts: Account[];
  categories: Category[];
  transactions: Tx[];
  budgets: Budget[];
  recurring: Recurring[];
  settings: Record<string, string>;
  seq: number;
}

const KEY = 'moneywort-preview';

function seed(): Store {
  const cats: [string, string, string, 'expense' | 'income'][] = [
    ['Groceries', '🛒', '#5b8c5a', 'expense'],
    ['Eating out', '🍜', '#d1763a', 'expense'],
    ['Transport', '🚌', '#4a7bb7', 'expense'],
    ['Housing', '🏠', '#8a6bbf', 'expense'],
    ['Bills', '💡', '#c9a227', 'expense'],
    ['Shopping', '🛍️', '#c2577a', 'expense'],
    ['Health', '💊', '#3a9a9a', 'expense'],
    ['Fun', '🎬', '#e05d5d', 'expense'],
    ['Subscriptions', '🔁', '#6b7a8f', 'expense'],
    ['Other', '•', '#8d8d8d', 'expense'],
    ['Salary', '💼', '#2f7d5b', 'income'],
    ['Freelance', '🧑‍💻', '#3f8f7a', 'income'],
    ['Gifts', '🎁', '#b5739d', 'income'],
    ['Other income', '➕', '#6f8f6f', 'income'],
  ];
  const s: Store = {
    accounts: [
      { id: 1, name: 'Cash', kind: 'cash', color: '#5b8c5a', opening_balance: 45000, archived: false, sort: 0 },
      { id: 2, name: 'Bank', kind: 'bank', color: '#4a7bb7', opening_balance: 240000, archived: false, sort: 1 },
    ],
    categories: cats.map(([name, icon, color, kind], i) => ({ id: i + 1, name, icon, color, kind, archived: false, sort: i })),
    transactions: [],
    budgets: [
      { category_id: 1, amount: 40000 },
      { category_id: 2, amount: 15000 },
      { category_id: 3, amount: 8000 },
    ],
    recurring: [],
    settings: { currency: 'EUR', theme: 'system' },
    seq: 100,
  };
  // Demo history so the preview isn't empty.
  const notes: Record<number, string[]> = {
    1: ['Lidl', 'Farmers market', 'Spar'],
    2: ['Pho place', 'Pizza with Ana', 'Coffee'],
    3: ['Bus card', 'Taxi'],
    6: ['Sneakers', 'Book'],
    8: ['Cinema', 'Concert'],
  };
  const amounts: Record<number, [number, number]> = { 1: [1200, 6500], 2: [350, 3800], 3: [150, 2200], 6: [1500, 9000], 8: [900, 4000] };
  let r = 7;
  const rnd = () => ((r = (r * 9301 + 49297) % 233280) / 233280);
  const start = new Date();
  start.setMonth(start.getMonth() - 5, 1);
  for (let d = new Date(start); iso(d) <= todayISO(); d.setDate(d.getDate() + 1)) {
    const date = iso(d);
    if (d.getDate() === 1) {
      s.transactions.push({ id: s.seq++, kind: 'income', amount: 320000, account_id: 2, to_account_id: null, category_id: 11, date, note: 'Monthly salary' });
      s.transactions.push({ id: s.seq++, kind: 'expense', amount: 85000, account_id: 2, to_account_id: null, category_id: 4, date, note: 'Rent' });
    }
    if (d.getDate() === 12) s.transactions.push({ id: s.seq++, kind: 'expense', amount: 1299, account_id: 2, to_account_id: null, category_id: 9, date, note: 'Music' });
    if (d.getDate() === 15) s.transactions.push({ id: s.seq++, kind: 'expense', amount: 6420, account_id: 2, to_account_id: null, category_id: 5, date, note: 'Electricity' });
    for (const c of [1, 2, 3, 6, 8]) {
      const chance = c === 1 ? 0.35 : c === 2 ? 0.3 : c === 3 ? 0.25 : 0.06;
      if (rnd() < chance) {
        const [lo, hi] = amounts[c];
        const n = notes[c];
        s.transactions.push({
          id: s.seq++, kind: 'expense', amount: Math.round(lo + rnd() * (hi - lo)), account_id: rnd() < 0.08 ? 1 : 2,
          to_account_id: null, category_id: c, date, note: n[Math.floor(rnd() * n.length)],
        });
      }
    }
  }
  s.recurring.push({
    id: 1, kind: 'expense', amount: 1299, account_id: 2, category_id: 9, note: 'Music', interval: 'monthly',
    start_date: iso(start).slice(0, 8) + '12', posted: 999, active: true, next_date: '',
  });
  return s;
}

function load(): Store {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    /* fall through */
  }
  return seed();
}

let db = load();
const persist = () => {
  try {
    localStorage.setItem(KEY, JSON.stringify(db));
  } catch {
    /* preview only */
  }
};

function occurrence(start: string, interval: string, n: number): string {
  const d = parseISO(start);
  if (interval === 'weekly') d.setDate(d.getDate() + 7 * n);
  else {
    const day = d.getDate();
    const months = interval === 'yearly' ? 12 * n : n;
    d.setDate(1);
    d.setMonth(d.getMonth() + months);
    const last = new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
    d.setDate(Math.min(day, last));
  }
  return iso(d);
}

function withNext(r: Recurring): Recurring {
  return { ...r, next_date: occurrence(r.start_date, r.interval, r.posted) };
}

function processRecurring(): number {
  let count = 0;
  const t = todayISO();
  for (const r of db.recurring) {
    if (!r.active) continue;
    if (r.posted === 999) {
      // demo rule: mark as caught up
      r.posted = 0;
      while (occurrence(r.start_date, r.interval, r.posted) <= t) r.posted++;
      continue;
    }
    while (occurrence(r.start_date, r.interval, r.posted) <= t) {
      db.transactions.push({
        id: db.seq++, kind: r.kind, amount: r.amount, account_id: r.account_id, to_account_id: null,
        category_id: r.category_id, date: occurrence(r.start_date, r.interval, r.posted), note: r.note, recurring_id: r.id,
      });
      r.posted++;
      count++;
    }
  }
  return count;
}

const sortTx = (a: Tx, b: Tx) => (a.date < b.date ? 1 : a.date > b.date ? -1 : b.id - a.id);

function totals(from: string, to: string) {
  let income = 0;
  let expense = 0;
  for (const t of db.transactions) {
    if (t.date < from || t.date >= to) continue;
    if (t.kind === 'income') income += t.amount;
    if (t.kind === 'expense') expense += t.amount;
  }
  return { income, expense };
}

function balances() {
  return db.accounts.map((a) => {
    let b = a.opening_balance;
    for (const t of db.transactions) {
      if (t.kind === 'income' && t.account_id === a.id) b += t.amount;
      if (t.kind === 'expense' && t.account_id === a.id) b -= t.amount;
      if (t.kind === 'transfer' && t.account_id === a.id) b -= t.amount;
      if (t.kind === 'transfer' && t.to_account_id === a.id) b += t.amount;
    }
    return { account_id: a.id, balance: b };
  });
}

function summary(month: string): Summary {
  const from = month + '-01';
  const to = addMonths(month, 1) + '-01';
  const { income, expense } = totals(from, to);
  const prev = addMonths(month, -1) + '-01';
  const cats = new Map<number | null, { total: number; count: number }>();
  const days = new Map<string, { income: number; expense: number }>();
  for (const t of db.transactions) {
    if (t.date < from || t.date >= to || t.kind === 'transfer') continue;
    const d = days.get(t.date) ?? { income: 0, expense: 0 };
    d[t.kind] += t.amount;
    days.set(t.date, d);
    if (t.kind === 'expense') {
      const c = cats.get(t.category_id) ?? { total: 0, count: 0 };
      c.total += t.amount;
      c.count++;
      cats.set(t.category_id, c);
    }
  }
  const bal = balances();
  return {
    month,
    income,
    expense,
    prev_expense: totals(prev, from).expense,
    by_category: [...cats].map(([category_id, v]) => ({ category_id, ...v })).sort((a, b) => b.total - a.total),
    daily: [...days].map(([key, v]) => ({ key, ...v })).sort((a, b) => (a.key < b.key ? -1 : 1)),
    trend: [5, 4, 3, 2, 1, 0].map((i) => {
      const m = addMonths(month, -i);
      return { key: m, ...totals(m + '-01', addMonths(m, 1) + '-01') };
    }),
    balances: bal,
    net_worth: bal.filter((b) => !db.accounts.find((a) => a.id === b.account_id)?.archived).reduce((s, b) => s + b.balance, 0),
  };
}

function list(f: TxFilter): Tx[] {
  const q = f.query?.trim().toLowerCase();
  let out = db.transactions.filter((t) => {
    if (f.month && t.date.slice(0, 7) !== f.month) return false;
    if (f.account_id && t.account_id !== f.account_id && t.to_account_id !== f.account_id) return false;
    if (f.category_id && t.category_id !== f.category_id) return false;
    if (f.kind && t.kind !== f.kind) return false;
    if (q) {
      const cat = db.categories.find((c) => c.id === t.category_id)?.name.toLowerCase() ?? '';
      if (!t.note.toLowerCase().includes(q) && !cat.includes(q)) return false;
    }
    return true;
  });
  out.sort(sortTx);
  if (f.limit) out = out.slice(0, f.limit);
  return out.map((t) => ({ ...t }));
}

function upsert<T extends { id: number }>(arr: T[], item: T): number {
  if (item.id > 0) {
    const i = arr.findIndex((x) => x.id === item.id);
    if (i >= 0) arr[i] = { ...item };
    return item.id;
  }
  const id = db.seq++;
  arr.push({ ...item, id });
  return id;
}

export function handle(cmd: string, a: Record<string, any>): unknown {
  const fail = (m: string) => {
    throw m;
  };
  let out: unknown;
  switch (cmd) {
    case 'bootstrap': {
      const posted = processRecurring();
      out = {
        accounts: [...db.accounts].sort((x, y) => +x.archived - +y.archived || x.sort - y.sort),
        categories: [...db.categories].sort((x, y) => +x.archived - +y.archived || x.sort - y.sort),
        budgets: db.budgets,
        recurring: db.recurring.map(withNext),
        settings: db.settings,
        posted_recurring: posted,
      };
      break;
    }
    case 'list_transactions':
      return list(a.filter ?? {});
    case 'summary':
      return summary(a.month);
    case 'save_transaction': {
      const t: Tx = a.tx;
      if (!(t.amount > 0)) fail('Amount must be greater than zero');
      if (t.kind === 'transfer' && (!t.to_account_id || t.to_account_id === t.account_id)) fail('Transfer accounts must differ');
      if (t.kind === 'transfer') t.category_id = null;
      else t.to_account_id = null;
      out = upsert(db.transactions, { ...t, note: t.note.trim() });
      break;
    }
    case 'delete_transaction':
      db.transactions = db.transactions.filter((t) => t.id !== a.id);
      break;
    case 'save_account':
      if (!a.account.name.trim()) fail('Name is required');
      out = upsert(db.accounts, { ...a.account, sort: a.account.sort || db.accounts.length });
      break;
    case 'delete_account': {
      const used = db.transactions.some((t) => t.account_id === a.id || t.to_account_id === a.id);
      if (used) db.accounts.find((x) => x.id === a.id)!.archived = true;
      else db.accounts = db.accounts.filter((x) => x.id !== a.id);
      out = used;
      break;
    }
    case 'save_category':
      if (!a.category.name.trim()) fail('Name is required');
      out = upsert(db.categories, { ...a.category, sort: a.category.sort || db.categories.length });
      break;
    case 'delete_category': {
      const used = db.transactions.some((t) => t.category_id === a.id);
      db.budgets = db.budgets.filter((b) => b.category_id !== a.id);
      if (used) db.categories.find((x) => x.id === a.id)!.archived = true;
      else db.categories = db.categories.filter((x) => x.id !== a.id);
      out = used;
      break;
    }
    case 'set_budget':
      db.budgets = db.budgets.filter((b) => b.category_id !== a.categoryId);
      if (a.amount > 0) db.budgets.push({ category_id: a.categoryId, amount: a.amount });
      break;
    case 'save_recurring': {
      const r: Recurring = { ...a.rule };
      if (!(r.amount > 0)) fail('Amount must be greater than zero');
      if (r.id > 0) {
        r.posted = 0;
        while (occurrence(r.start_date, r.interval, r.posted) < todayISO()) r.posted++;
      } else r.posted = 0;
      out = upsert(db.recurring, r);
      processRecurring();
      break;
    }
    case 'delete_recurring':
      db.recurring = db.recurring.filter((r) => r.id !== a.id);
      break;
    case 'set_setting':
      db.settings[a.key] = a.value;
      break;
    case 'export_json': {
      const { seq, ...rest } = db;
      return JSON.stringify({ version: 1, ...rest }, null, 2);
    }
    case 'import_json': {
      let b: any;
      try {
        b = JSON.parse(a.data);
      } catch (e) {
        fail('Not a valid Moneywort backup: ' + e);
      }
      if (!Array.isArray(b.accounts) || !Array.isArray(b.transactions)) fail('Not a valid Moneywort backup');
      const ids = [...b.accounts, ...b.categories, ...b.transactions, ...b.recurring].map((x: any) => x.id);
      db = { ...b, seq: Math.max(100, ...ids) + 1 };
      break;
    }
    case 'export_csv':
      return 'date,type,amount,account,to_account,category,note\n' +
        list({}).map((t) => [t.date, t.kind, (t.amount / 100).toFixed(2), t.account_id, t.to_account_id ?? '', t.category_id ?? '', t.note].join(',')).join('\n');
    case 'wipe_data':
      db = seed();
      db.transactions = [];
      db.budgets = [];
      db.recurring = [];
      break;
    default:
      fail('Unknown command ' + cmd);
  }
  persist();
  return out;
}
