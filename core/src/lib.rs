//! SQLite storage. All money is stored as integer minor units (cents).

use chrono::{Datelike, Local, Months, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Res<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'bank',
    color TEXT NOT NULL DEFAULT '#3b6e5a',
    opening_balance INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    sort INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('expense','income')),
    icon TEXT NOT NULL DEFAULT '•',
    color TEXT NOT NULL DEFAULT '#888888',
    archived INTEGER NOT NULL DEFAULT 0,
    sort INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('expense','income','transfer')),
    amount INTEGER NOT NULL CHECK (amount > 0),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    to_account_id INTEGER REFERENCES accounts(id),
    category_id INTEGER REFERENCES categories(id),
    date TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    recurring_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_tx_date ON transactions(date);

CREATE TABLE IF NOT EXISTS budgets (
    category_id INTEGER PRIMARY KEY REFERENCES categories(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS recurring (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('expense','income')),
    amount INTEGER NOT NULL CHECK (amount > 0),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    category_id INTEGER REFERENCES categories(id),
    note TEXT NOT NULL DEFAULT '',
    interval TEXT NOT NULL CHECK (interval IN ('weekly','monthly','yearly')),
    start_date TEXT NOT NULL,
    posted INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

// ---------- models ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub color: String,
    pub opening_balance: i64,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub sort: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub sort: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    #[serde(default)]
    pub id: i64,
    pub kind: String,
    pub amount: i64,
    pub account_id: i64,
    pub to_account_id: Option<i64>,
    pub category_id: Option<i64>,
    pub date: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub recurring_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Budget {
    pub category_id: i64,
    pub amount: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Recurring {
    #[serde(default)]
    pub id: i64,
    pub kind: String,
    pub amount: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    #[serde(default)]
    pub note: String,
    pub interval: String,
    pub start_date: String,
    #[serde(default)]
    pub posted: i64,
    #[serde(default = "yes")]
    pub active: bool,
    /// Computed: next date this rule will post.
    #[serde(default)]
    pub next_date: String,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize, Default, Debug)]
pub struct TxFilter {
    /// "YYYY-MM"
    pub month: Option<String>,
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
    pub kind: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Debug)]
pub struct Bootstrap {
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub budgets: Vec<Budget>,
    pub recurring: Vec<Recurring>,
    pub settings: HashMap<String, String>,
    pub posted_recurring: i64,
}

#[derive(Serialize, Debug)]
pub struct Amount2 {
    pub key: String,
    pub income: i64,
    pub expense: i64,
}

#[derive(Serialize, Debug)]
pub struct CatTotal {
    pub category_id: Option<i64>,
    pub total: i64,
    pub count: i64,
}

#[derive(Serialize, Debug)]
pub struct Balance {
    pub account_id: i64,
    pub balance: i64,
}

#[derive(Serialize, Debug)]
pub struct Summary {
    pub month: String,
    pub income: i64,
    pub expense: i64,
    pub prev_expense: i64,
    pub by_category: Vec<CatTotal>,
    pub daily: Vec<Amount2>,
    pub trend: Vec<Amount2>,
    pub balances: Vec<Balance>,
    pub net_worth: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Backup {
    pub version: i64,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub transactions: Vec<Transaction>,
    pub budgets: Vec<Budget>,
    pub recurring: Vec<Recurring>,
    pub settings: HashMap<String, String>,
}

// ---------- helpers ----------

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn parse_date(s: &str) -> Res<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| format!("Invalid date: {s}"))
}

fn month_bounds(month: &str) -> Res<(String, String)> {
    let first = parse_date(&format!("{month}-01"))?;
    let next = first + Months::new(1);
    Ok((first.to_string(), next.to_string()))
}

/// Date of the n-th occurrence (0-based) of a recurring rule. Computed from the
/// start date each time so month-end dates never drift (Jan 31 -> Feb 28 -> Mar 31).
fn occurrence(start: NaiveDate, interval: &str, n: i64) -> NaiveDate {
    let n32 = n.max(0) as u32;
    match interval {
        "weekly" => start + chrono::Duration::weeks(n),
        "yearly" => start + Months::new(12 * n32),
        _ => start + Months::new(n32),
    }
}

fn validate_tx(t: &Transaction) -> Res<()> {
    if t.amount <= 0 {
        return Err("Amount must be greater than zero".into());
    }
    parse_date(&t.date)?;
    match t.kind.as_str() {
        "transfer" => {
            let to = t.to_account_id.ok_or("Pick a destination account")?;
            if to == t.account_id {
                return Err("Transfer accounts must differ".into());
            }
        }
        "expense" | "income" => {}
        _ => return Err("Unknown transaction type".into()),
    }
    Ok(())
}

// ---------- the store ----------

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &std::path::Path) -> Res<Self> {
        let conn = Connection::open(path).map_err(e)?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn memory() -> Res<Self> {
        Self::init(Connection::open_in_memory().map_err(e)?)
    }

    fn init(conn: Connection) -> Res<Self> {
        conn.execute_batch(SCHEMA).map_err(e)?;
        let db = Db { conn };
        db.seed_if_empty()?;
        Ok(db)
    }

    fn seed_if_empty(&self) -> Res<()> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .map_err(e)?;
        if n > 0 {
            return Ok(());
        }
        let expense = [
            ("Groceries", "🛒", "#5b8c5a"),
            ("Eating out", "🍜", "#d1763a"),
            ("Transport", "🚌", "#4a7bb7"),
            ("Housing", "🏠", "#8a6bbf"),
            ("Bills", "💡", "#c9a227"),
            ("Shopping", "🛍️", "#c2577a"),
            ("Health", "💊", "#3a9a9a"),
            ("Fun", "🎬", "#e05d5d"),
            ("Subscriptions", "🔁", "#6b7a8f"),
            ("Other", "•", "#8d8d8d"),
        ];
        let income = [
            ("Salary", "💼", "#2f7d5b"),
            ("Freelance", "🧑‍💻", "#3f8f7a"),
            ("Gifts", "🎁", "#b5739d"),
            ("Other income", "➕", "#6f8f6f"),
        ];
        for (i, (n, ic, c)) in expense.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO categories (name, kind, icon, color, sort) VALUES (?1,'expense',?2,?3,?4)",
                    params![n, ic, c, i as i64],
                )
                .map_err(e)?;
        }
        for (i, (n, ic, c)) in income.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO categories (name, kind, icon, color, sort) VALUES (?1,'income',?2,?3,?4)",
                    params![n, ic, c, i as i64],
                )
                .map_err(e)?;
        }
        let accts: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .map_err(e)?;
        if accts == 0 {
            self.conn
                .execute(
                    "INSERT INTO accounts (name, kind, color, sort) VALUES ('Cash','cash','#5b8c5a',0), ('Bank','bank','#4a7bb7',1)",
                    [],
                )
                .map_err(e)?;
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('currency','EUR'), ('theme','system')",
                [],
            )
            .map_err(e)?;
        Ok(())
    }

    // ----- reads -----

    pub fn accounts(&self) -> Res<Vec<Account>> {
        let mut st = self
            .conn
            .prepare("SELECT id,name,kind,color,opening_balance,archived,sort FROM accounts ORDER BY archived, sort, id")
            .map_err(e)?;
        let rows = st
            .query_map([], |r| {
                Ok(Account {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    kind: r.get(2)?,
                    color: r.get(3)?,
                    opening_balance: r.get(4)?,
                    archived: r.get(5)?,
                    sort: r.get(6)?,
                })
            })
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    pub fn categories(&self) -> Res<Vec<Category>> {
        let mut st = self
            .conn
            .prepare("SELECT id,name,kind,icon,color,archived,sort FROM categories ORDER BY archived, kind, sort, id")
            .map_err(e)?;
        let rows = st
            .query_map([], |r| {
                Ok(Category {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    kind: r.get(2)?,
                    icon: r.get(3)?,
                    color: r.get(4)?,
                    archived: r.get(5)?,
                    sort: r.get(6)?,
                })
            })
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    pub fn budgets(&self) -> Res<Vec<Budget>> {
        let mut st = self
            .conn
            .prepare("SELECT category_id, amount FROM budgets")
            .map_err(e)?;
        let rows = st
            .query_map([], |r| {
                Ok(Budget {
                    category_id: r.get(0)?,
                    amount: r.get(1)?,
                })
            })
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    pub fn recurring(&self) -> Res<Vec<Recurring>> {
        let mut st = self
            .conn
            .prepare("SELECT id,kind,amount,account_id,category_id,note,interval,start_date,posted,active FROM recurring ORDER BY active DESC, id")
            .map_err(e)?;
        let rows = st
            .query_map([], |r| {
                Ok(Recurring {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    amount: r.get(2)?,
                    account_id: r.get(3)?,
                    category_id: r.get(4)?,
                    note: r.get(5)?,
                    interval: r.get(6)?,
                    start_date: r.get(7)?,
                    posted: r.get(8)?,
                    active: r.get(9)?,
                    next_date: String::new(),
                })
            })
            .map_err(e)?;
        let mut out: Vec<Recurring> = rows.collect::<Result<_, _>>().map_err(e)?;
        for r in &mut out {
            if let Ok(start) = parse_date(&r.start_date) {
                r.next_date = occurrence(start, &r.interval, r.posted).to_string();
            }
        }
        Ok(out)
    }

    pub fn settings(&self) -> Res<HashMap<String, String>> {
        let mut st = self
            .conn
            .prepare("SELECT key, value FROM settings")
            .map_err(e)?;
        let rows = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    pub fn bootstrap(&mut self) -> Res<Bootstrap> {
        let posted = self.process_recurring(today())?;
        Ok(Bootstrap {
            accounts: self.accounts()?,
            categories: self.categories()?,
            budgets: self.budgets()?,
            recurring: self.recurring()?,
            settings: self.settings()?,
            posted_recurring: posted,
        })
    }

    pub fn transactions(&self, f: &TxFilter) -> Res<Vec<Transaction>> {
        let mut sql = String::from(
            "SELECT t.id,t.kind,t.amount,t.account_id,t.to_account_id,t.category_id,t.date,t.note,t.recurring_id
             FROM transactions t LEFT JOIN categories c ON c.id = t.category_id WHERE 1=1",
        );
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(m) = &f.month {
            let (a, b) = month_bounds(m)?;
            sql.push_str(" AND t.date >= ? AND t.date < ?");
            args.push(Box::new(a));
            args.push(Box::new(b));
        }
        if let Some(a) = f.account_id {
            sql.push_str(" AND (t.account_id = ? OR t.to_account_id = ?)");
            args.push(Box::new(a));
            args.push(Box::new(a));
        }
        if let Some(c) = f.category_id {
            sql.push_str(" AND t.category_id = ?");
            args.push(Box::new(c));
        }
        if let Some(k) = &f.kind {
            sql.push_str(" AND t.kind = ?");
            args.push(Box::new(k.clone()));
        }
        if let Some(q) = f.query.as_ref().map(|q| q.trim()).filter(|q| !q.is_empty()) {
            sql.push_str(" AND (t.note LIKE ? OR c.name LIKE ?)");
            let like = format!("%{q}%");
            args.push(Box::new(like.clone()));
            args.push(Box::new(like));
        }
        sql.push_str(" ORDER BY t.date DESC, t.id DESC");
        if let Some(l) = f.limit {
            sql.push_str(&format!(" LIMIT {}", l.clamp(1, 100_000)));
        }
        let mut st = self.conn.prepare(&sql).map_err(e)?;
        let params = rusqlite::params_from_iter(args.iter().map(|b| b.as_ref()));
        let rows = st
            .query_map(params, |r| {
                Ok(Transaction {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    amount: r.get(2)?,
                    account_id: r.get(3)?,
                    to_account_id: r.get(4)?,
                    category_id: r.get(5)?,
                    date: r.get(6)?,
                    note: r.get(7)?,
                    recurring_id: r.get(8)?,
                })
            })
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    pub fn balances(&self) -> Res<Vec<Balance>> {
        let mut st = self
            .conn
            .prepare(
                "SELECT a.id, a.opening_balance
                   + COALESCE((SELECT SUM(amount) FROM transactions WHERE kind='income' AND account_id=a.id),0)
                   - COALESCE((SELECT SUM(amount) FROM transactions WHERE kind='expense' AND account_id=a.id),0)
                   - COALESCE((SELECT SUM(amount) FROM transactions WHERE kind='transfer' AND account_id=a.id),0)
                   + COALESCE((SELECT SUM(amount) FROM transactions WHERE kind='transfer' AND to_account_id=a.id),0)
                 FROM accounts a",
            )
            .map_err(e)?;
        let rows = st
            .query_map([], |r| {
                Ok(Balance {
                    account_id: r.get(0)?,
                    balance: r.get(1)?,
                })
            })
            .map_err(e)?;
        rows.collect::<Result<_, _>>().map_err(e)
    }

    fn totals(&self, from: &str, to: &str) -> Res<(i64, i64)> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(CASE WHEN kind='income' THEN amount END),0),
                        COALESCE(SUM(CASE WHEN kind='expense' THEN amount END),0)
                 FROM transactions WHERE date >= ?1 AND date < ?2",
                params![from, to],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(e)
    }

    pub fn summary(&self, month: &str) -> Res<Summary> {
        let (from, to) = month_bounds(month)?;
        let (income, expense) = self.totals(&from, &to)?;

        let first = parse_date(&from)?;
        let prev = first - Months::new(1);
        let (_, prev_expense) = self.totals(&prev.to_string(), &from)?;

        let mut st = self
            .conn
            .prepare(
                "SELECT category_id, SUM(amount), COUNT(*) FROM transactions
                 WHERE kind='expense' AND date >= ?1 AND date < ?2
                 GROUP BY category_id ORDER BY SUM(amount) DESC",
            )
            .map_err(e)?;
        let by_category = st
            .query_map(params![from, to], |r| {
                Ok(CatTotal {
                    category_id: r.get(0)?,
                    total: r.get(1)?,
                    count: r.get(2)?,
                })
            })
            .map_err(e)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(e)?;

        let mut st = self
            .conn
            .prepare(
                "SELECT date,
                        COALESCE(SUM(CASE WHEN kind='income' THEN amount END),0),
                        COALESCE(SUM(CASE WHEN kind='expense' THEN amount END),0)
                 FROM transactions WHERE date >= ?1 AND date < ?2 GROUP BY date ORDER BY date",
            )
            .map_err(e)?;
        let daily = st
            .query_map(params![from, to], |r| {
                Ok(Amount2 {
                    key: r.get(0)?,
                    income: r.get(1)?,
                    expense: r.get(2)?,
                })
            })
            .map_err(e)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(e)?;

        let mut trend = Vec::with_capacity(6);
        for i in (0..6).rev() {
            let m0 = first - Months::new(i);
            let m1 = m0 + Months::new(1);
            let (inc, exp) = self.totals(&m0.to_string(), &m1.to_string())?;
            trend.push(Amount2 {
                key: format!("{:04}-{:02}", m0.year(), m0.month()),
                income: inc,
                expense: exp,
            });
        }

        let accounts = self.accounts()?;
        let balances = self.balances()?;
        let net_worth = balances
            .iter()
            .filter(|b| accounts.iter().any(|a| a.id == b.account_id && !a.archived))
            .map(|b| b.balance)
            .sum();

        Ok(Summary {
            month: month.to_string(),
            income,
            expense,
            prev_expense,
            by_category,
            daily,
            trend,
            balances,
            net_worth,
        })
    }

    // ----- writes -----

    pub fn save_transaction(&self, t: &Transaction) -> Res<i64> {
        validate_tx(t)?;
        let (cat, to) = if t.kind == "transfer" {
            (None, t.to_account_id)
        } else {
            (t.category_id, None)
        };
        if t.id > 0 {
            self.conn
                .execute(
                    "UPDATE transactions SET kind=?1, amount=?2, account_id=?3, to_account_id=?4, category_id=?5, date=?6, note=?7 WHERE id=?8",
                    params![t.kind, t.amount, t.account_id, to, cat, t.date, t.note.trim(), t.id],
                )
                .map_err(e)?;
            Ok(t.id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO transactions (kind, amount, account_id, to_account_id, category_id, date, note, recurring_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![t.kind, t.amount, t.account_id, to, cat, t.date, t.note.trim(), t.recurring_id],
                )
                .map_err(e)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn delete_transaction(&self, id: i64) -> Res<()> {
        self.conn
            .execute("DELETE FROM transactions WHERE id=?1", [id])
            .map_err(e)?;
        Ok(())
    }

    pub fn save_account(&self, a: &Account) -> Res<i64> {
        let name = a.name.trim();
        if name.is_empty() {
            return Err("Name is required".into());
        }
        if a.id > 0 {
            self.conn
                .execute(
                    "UPDATE accounts SET name=?1, kind=?2, color=?3, opening_balance=?4, archived=?5 WHERE id=?6",
                    params![name, a.kind, a.color, a.opening_balance, a.archived, a.id],
                )
                .map_err(e)?;
            Ok(a.id)
        } else {
            let sort: i64 = self
                .conn
                .query_row("SELECT COALESCE(MAX(sort),0)+1 FROM accounts", [], |r| r.get(0))
                .map_err(e)?;
            self.conn
                .execute(
                    "INSERT INTO accounts (name, kind, color, opening_balance, sort) VALUES (?1,?2,?3,?4,?5)",
                    params![name, a.kind, a.color, a.opening_balance, sort],
                )
                .map_err(e)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    /// Deletes an unused account; archives it if transactions reference it.
    /// Returns true if it was archived instead of deleted.
    pub fn delete_account(&self, id: i64) -> Res<bool> {
        let used: i64 = self
            .conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM transactions WHERE account_id=?1 OR to_account_id=?1)
                      + (SELECT COUNT(*) FROM recurring WHERE account_id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(e)?;
        if used > 0 {
            self.conn
                .execute("UPDATE accounts SET archived=1 WHERE id=?1", [id])
                .map_err(e)?;
            Ok(true)
        } else {
            self.conn
                .execute("DELETE FROM accounts WHERE id=?1", [id])
                .map_err(e)?;
            Ok(false)
        }
    }

    pub fn save_category(&self, c: &Category) -> Res<i64> {
        let name = c.name.trim();
        if name.is_empty() {
            return Err("Name is required".into());
        }
        if c.kind != "expense" && c.kind != "income" {
            return Err("Unknown category type".into());
        }
        if c.id > 0 {
            self.conn
                .execute(
                    "UPDATE categories SET name=?1, kind=?2, icon=?3, color=?4, archived=?5 WHERE id=?6",
                    params![name, c.kind, c.icon, c.color, c.archived, c.id],
                )
                .map_err(e)?;
            Ok(c.id)
        } else {
            let sort: i64 = self
                .conn
                .query_row("SELECT COALESCE(MAX(sort),0)+1 FROM categories", [], |r| r.get(0))
                .map_err(e)?;
            self.conn
                .execute(
                    "INSERT INTO categories (name, kind, icon, color, sort) VALUES (?1,?2,?3,?4,?5)",
                    params![name, c.kind, c.icon, c.color, sort],
                )
                .map_err(e)?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    /// Deletes an unused category; archives it if referenced. Returns true if archived.
    pub fn delete_category(&self, id: i64) -> Res<bool> {
        let used: i64 = self
            .conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM transactions WHERE category_id=?1)
                      + (SELECT COUNT(*) FROM recurring WHERE category_id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(e)?;
        self.conn
            .execute("DELETE FROM budgets WHERE category_id=?1", [id])
            .map_err(e)?;
        if used > 0 {
            self.conn
                .execute("UPDATE categories SET archived=1 WHERE id=?1", [id])
                .map_err(e)?;
            Ok(true)
        } else {
            self.conn
                .execute("DELETE FROM categories WHERE id=?1", [id])
                .map_err(e)?;
            Ok(false)
        }
    }

    pub fn set_budget(&self, category_id: i64, amount: i64) -> Res<()> {
        if amount <= 0 {
            self.conn
                .execute("DELETE FROM budgets WHERE category_id=?1", [category_id])
                .map_err(e)?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO budgets (category_id, amount) VALUES (?1,?2)
                     ON CONFLICT(category_id) DO UPDATE SET amount=excluded.amount",
                    params![category_id, amount],
                )
                .map_err(e)?;
        }
        Ok(())
    }

    pub fn save_recurring(&mut self, r: &Recurring) -> Res<i64> {
        if r.amount <= 0 {
            return Err("Amount must be greater than zero".into());
        }
        parse_date(&r.start_date)?;
        let id = if r.id > 0 {
            // Editing the schedule restarts counting from the new start date,
            // skipping occurrences that already happened.
            let start = parse_date(&r.start_date)?;
            let t = today();
            let mut posted = 0;
            while occurrence(start, &r.interval, posted) < t {
                posted += 1;
            }
            self.conn
                .execute(
                    "UPDATE recurring SET kind=?1, amount=?2, account_id=?3, category_id=?4, note=?5, interval=?6, start_date=?7, active=?8, posted=?9 WHERE id=?10",
                    params![r.kind, r.amount, r.account_id, r.category_id, r.note.trim(), r.interval, r.start_date, r.active, posted, r.id],
                )
                .map_err(e)?;
            r.id
        } else {
            self.conn
                .execute(
                    "INSERT INTO recurring (kind, amount, account_id, category_id, note, interval, start_date) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![r.kind, r.amount, r.account_id, r.category_id, r.note.trim(), r.interval, r.start_date],
                )
                .map_err(e)?;
            self.conn.last_insert_rowid()
        };
        self.process_recurring(today())?;
        Ok(id)
    }

    pub fn delete_recurring(&self, id: i64) -> Res<()> {
        self.conn
            .execute("DELETE FROM recurring WHERE id=?1", [id])
            .map_err(e)?;
        Ok(())
    }

    /// Posts every due occurrence of active recurring rules up to and including `upto`.
    pub fn process_recurring(&mut self, upto: NaiveDate) -> Res<i64> {
        let rules = self.recurring()?;
        let tx = self.conn.transaction().map_err(e)?;
        let mut count = 0;
        for r in rules.into_iter().filter(|r| r.active) {
            let start = parse_date(&r.start_date)?;
            let mut n = r.posted;
            // Guard against runaway loops (e.g. weekly rule started decades ago).
            let mut guard = 0;
            loop {
                let d = occurrence(start, &r.interval, n);
                if d > upto || guard > 2000 {
                    break;
                }
                tx.execute(
                    "INSERT INTO transactions (kind, amount, account_id, category_id, date, note, recurring_id) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![r.kind, r.amount, r.account_id, r.category_id, d.to_string(), r.note, r.id],
                )
                .map_err(e)?;
                n += 1;
                count += 1;
                guard += 1;
            }
            if n != r.posted {
                tx.execute("UPDATE recurring SET posted=?1 WHERE id=?2", params![n, r.id])
                    .map_err(e)?;
            }
        }
        tx.commit().map_err(e)?;
        Ok(count)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Res<()> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![key, value],
            )
            .map_err(e)?;
        Ok(())
    }

    // ----- backup -----

    pub fn export(&self) -> Res<Backup> {
        Ok(Backup {
            version: 1,
            accounts: self.accounts()?,
            categories: self.categories()?,
            transactions: self.transactions(&TxFilter::default())?,
            budgets: self.budgets()?,
            recurring: self.recurring()?,
            settings: self.settings()?,
        })
    }

    /// Replaces all data with the backup contents, atomically.
    pub fn import(&mut self, b: &Backup) -> Res<()> {
        let tx = self.conn.transaction().map_err(e)?;
        tx.execute_batch(
            "DELETE FROM transactions; DELETE FROM budgets; DELETE FROM recurring; DELETE FROM categories; DELETE FROM accounts; DELETE FROM settings;",
        )
        .map_err(e)?;
        for a in &b.accounts {
            tx.execute(
                "INSERT INTO accounts (id,name,kind,color,opening_balance,archived,sort) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![a.id, a.name, a.kind, a.color, a.opening_balance, a.archived, a.sort],
            )
            .map_err(e)?;
        }
        for c in &b.categories {
            tx.execute(
                "INSERT INTO categories (id,name,kind,icon,color,archived,sort) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![c.id, c.name, c.kind, c.icon, c.color, c.archived, c.sort],
            )
            .map_err(e)?;
        }
        for t in &b.transactions {
            tx.execute(
                "INSERT INTO transactions (id,kind,amount,account_id,to_account_id,category_id,date,note,recurring_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![t.id, t.kind, t.amount, t.account_id, t.to_account_id, t.category_id, t.date, t.note, t.recurring_id],
            )
            .map_err(e)?;
        }
        for bu in &b.budgets {
            tx.execute(
                "INSERT INTO budgets (category_id, amount) VALUES (?1,?2)",
                params![bu.category_id, bu.amount],
            )
            .map_err(e)?;
        }
        for r in &b.recurring {
            tx.execute(
                "INSERT INTO recurring (id,kind,amount,account_id,category_id,note,interval,start_date,posted,active) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![r.id, r.kind, r.amount, r.account_id, r.category_id, r.note, r.interval, r.start_date, r.posted, r.active],
            )
            .map_err(e)?;
        }
        for (k, v) in &b.settings {
            tx.execute("INSERT INTO settings (key, value) VALUES (?1,?2)", params![k, v])
                .map_err(e)?;
        }
        tx.commit().map_err(e)?;
        Ok(())
    }

    pub fn export_csv(&self) -> Res<String> {
        let accounts: HashMap<i64, String> =
            self.accounts()?.into_iter().map(|a| (a.id, a.name)).collect();
        let cats: HashMap<i64, String> =
            self.categories()?.into_iter().map(|c| (c.id, c.name)).collect();
        let esc = |s: &str| {
            if s.contains([',', '"', '\n']) {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        let mut out = String::from("date,type,amount,account,to_account,category,note\n");
        for t in self.transactions(&TxFilter::default())? {
            let acc = accounts.get(&t.account_id).cloned().unwrap_or_default();
            let to = t
                .to_account_id
                .and_then(|i| accounts.get(&i).cloned())
                .unwrap_or_default();
            let cat = t
                .category_id
                .and_then(|i| cats.get(&i).cloned())
                .unwrap_or_default();
            let amt = format!("{}.{:02}", t.amount / 100, t.amount % 100);
            out.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                t.date,
                t.kind,
                amt,
                esc(&acc),
                esc(&to),
                esc(&cat),
                esc(&t.note)
            ));
        }
        Ok(out)
    }

    pub fn wipe(&mut self) -> Res<()> {
        self.conn
            .execute_batch(
                "DELETE FROM transactions; DELETE FROM budgets; DELETE FROM recurring; DELETE FROM categories; DELETE FROM accounts; DELETE FROM settings;",
            )
            .map_err(e)?;
        self.seed_if_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(kind: &str, amount: i64, acc: i64, to: Option<i64>, cat: Option<i64>, date: &str) -> Transaction {
        Transaction {
            id: 0,
            kind: kind.into(),
            amount,
            account_id: acc,
            to_account_id: to,
            category_id: cat,
            date: date.into(),
            note: String::new(),
            recurring_id: None,
        }
    }

    #[test]
    fn seeds_defaults() {
        let db = Db::memory().unwrap();
        assert_eq!(db.accounts().unwrap().len(), 2);
        assert!(db.categories().unwrap().len() >= 10);
        assert_eq!(db.settings().unwrap()["currency"], "EUR");
    }

    #[test]
    fn balances_and_summary() {
        let db = Db::memory().unwrap();
        let cash = 1;
        let bank = 2;
        db.save_transaction(&tx("income", 300_000, bank, None, Some(11), "2026-03-01")).unwrap();
        db.save_transaction(&tx("expense", 4_550, bank, None, Some(1), "2026-03-02")).unwrap();
        db.save_transaction(&tx("transfer", 10_000, bank, Some(cash), None, "2026-03-03")).unwrap();
        db.save_transaction(&tx("expense", 2_000, cash, None, Some(2), "2026-02-10")).unwrap();

        let b: HashMap<i64, i64> = db.balances().unwrap().into_iter().map(|b| (b.account_id, b.balance)).collect();
        assert_eq!(b[&bank], 300_000 - 4_550 - 10_000);
        assert_eq!(b[&cash], 10_000 - 2_000);

        let s = db.summary("2026-03").unwrap();
        assert_eq!(s.income, 300_000);
        assert_eq!(s.expense, 4_550);
        assert_eq!(s.prev_expense, 2_000);
        assert_eq!(s.net_worth, 300_000 - 4_550 - 2_000);
        assert_eq!(s.trend.len(), 6);
        assert_eq!(s.trend.last().unwrap().key, "2026-03");
    }

    #[test]
    fn rejects_bad_transactions() {
        let db = Db::memory().unwrap();
        assert!(db.save_transaction(&tx("expense", 0, 1, None, None, "2026-01-01")).is_err());
        assert!(db.save_transaction(&tx("transfer", 100, 1, Some(1), None, "2026-01-01")).is_err());
        assert!(db.save_transaction(&tx("expense", 100, 1, None, None, "nope")).is_err());
    }

    #[test]
    fn recurring_posts_without_month_end_drift() {
        let mut db = Db::memory().unwrap();
        db.conn
            .execute(
                "INSERT INTO recurring (kind, amount, account_id, category_id, interval, start_date) VALUES ('expense', 999, 2, 9, 'monthly', '2026-01-31')",
                [],
            )
            .unwrap();
        let n = db.process_recurring(parse_date("2026-04-15").unwrap()).unwrap();
        assert_eq!(n, 3);
        let dates: Vec<String> = db.transactions(&TxFilter::default()).unwrap().into_iter().map(|t| t.date).collect();
        assert_eq!(dates, vec!["2026-03-31", "2026-02-28", "2026-01-31"]);
        // Idempotent.
        assert_eq!(db.process_recurring(parse_date("2026-04-15").unwrap()).unwrap(), 0);
        assert_eq!(db.recurring().unwrap()[0].next_date, "2026-04-30");
    }

    #[test]
    fn delete_archives_used_entities() {
        let db = Db::memory().unwrap();
        db.save_transaction(&tx("expense", 100, 1, None, Some(1), "2026-01-01")).unwrap();
        assert!(db.delete_account(1).unwrap());
        assert!(!db.delete_account(2).unwrap());
        assert!(db.delete_category(1).unwrap());
        assert!(!db.delete_category(2).unwrap());
    }

    #[test]
    fn backup_roundtrip() {
        let mut db = Db::memory().unwrap();
        db.save_transaction(&tx("expense", 1234, 1, None, Some(1), "2026-01-01")).unwrap();
        db.set_budget(1, 50_000).unwrap();
        let b = db.export().unwrap();
        let json = serde_json::to_string(&b).unwrap();
        db.wipe().unwrap();
        assert!(db.transactions(&TxFilter::default()).unwrap().is_empty());
        db.import(&serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(db.transactions(&TxFilter::default()).unwrap().len(), 1);
        assert_eq!(db.budgets().unwrap()[0].amount, 50_000);
        assert!(db.export_csv().unwrap().contains("12.34"));
    }
}
