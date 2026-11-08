import type { Account, Bootstrap, Category, Recurring, Summary, Tx, TxFilter } from './types';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }
  // Browser preview: same contract, backed by localStorage.
  const mock = await import('./mock');
  return mock.handle(cmd, args) as T;
}

export const api = {
  bootstrap: () => call<Bootstrap>('bootstrap'),
  transactions: (filter: TxFilter) => call<Tx[]>('list_transactions', { filter }),
  saveTx: (tx: Tx) => call<number>('save_transaction', { tx }),
  deleteTx: (id: number) => call<void>('delete_transaction', { id }),
  summary: (month: string) => call<Summary>('summary', { month }),
  saveAccount: (account: Account) => call<number>('save_account', { account }),
  deleteAccount: (id: number) => call<boolean>('delete_account', { id }),
  saveCategory: (category: Category) => call<number>('save_category', { category }),
  deleteCategory: (id: number) => call<boolean>('delete_category', { id }),
  setBudget: (categoryId: number, amount: number) => call<void>('set_budget', { categoryId, amount }),
  saveRecurring: (rule: Recurring) => call<number>('save_recurring', { rule }),
  deleteRecurring: (id: number) => call<void>('delete_recurring', { id }),
  setSetting: (key: string, value: string) => call<void>('set_setting', { key, value }),
  exportJson: () => call<string>('export_json'),
  importJson: (data: string) => call<void>('import_json', { data }),
  exportCsv: () => call<string>('export_csv'),
  wipe: () => call<void>('wipe_data'),
};

/** Ask where to save a text file and write it. Returns false if cancelled. */
export async function saveTextFile(name: string, contents: string, ext: string): Promise<boolean> {
  if (isTauri) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    const path = await save({ defaultPath: name, filters: [{ name: ext.toUpperCase(), extensions: [ext] }] });
    if (!path) return false;
    await writeTextFile(path, contents);
    return true;
  }
  const url = URL.createObjectURL(new Blob([contents], { type: 'text/plain' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
  return true;
}

/** Pick a text file and read it. Returns null if cancelled. */
export async function openTextFile(ext: string): Promise<string | null> {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const path = await open({ multiple: false, filters: [{ name: ext.toUpperCase(), extensions: [ext] }] });
    if (!path || Array.isArray(path)) return null;
    return readTextFile(path);
  }
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.' + ext;
    input.onchange = async () => resolve(input.files?.[0] ? await input.files[0].text() : null);
    input.click();
  });
}
