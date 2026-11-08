export type TxKind = 'expense' | 'income' | 'transfer';
export type CatKind = 'expense' | 'income';
export type Interval = 'weekly' | 'monthly' | 'yearly';

export interface Account {
  id: number;
  name: string;
  kind: string;
  color: string;
  opening_balance: number;
  archived: boolean;
  sort: number;
}

export interface Category {
  id: number;
  name: string;
  kind: CatKind;
  icon: string;
  color: string;
  archived: boolean;
  sort: number;
}

export interface Tx {
  id: number;
  kind: TxKind;
  amount: number;
  account_id: number;
  to_account_id: number | null;
  category_id: number | null;
  date: string;
  note: string;
  recurring_id?: number | null;
}

export interface Budget {
  category_id: number;
  amount: number;
}

export interface Recurring {
  id: number;
  kind: CatKind;
  amount: number;
  account_id: number;
  category_id: number | null;
  note: string;
  interval: Interval;
  start_date: string;
  posted: number;
  active: boolean;
  next_date: string;
}

export interface Bootstrap {
  accounts: Account[];
  categories: Category[];
  budgets: Budget[];
  recurring: Recurring[];
  settings: Record<string, string>;
  posted_recurring: number;
}

export interface Pair {
  key: string;
  income: number;
  expense: number;
}

export interface Summary {
  month: string;
  income: number;
  expense: number;
  prev_expense: number;
  by_category: { category_id: number | null; total: number; count: number }[];
  daily: Pair[];
  trend: Pair[];
  balances: { account_id: number; balance: number }[];
  net_worth: number;
}

export interface TxFilter {
  month?: string;
  account_id?: number;
  category_id?: number;
  kind?: TxKind;
  query?: string;
  limit?: number;
}
