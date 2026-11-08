use penny_core::{Account, Backup, Bootstrap, Category, Db, Recurring, Res, Summary, Transaction, TxFilter};
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState(Mutex<Db>);

fn with<T>(s: &State<AppState>, f: impl FnOnce(&mut Db) -> Res<T>) -> Res<T> {
    let mut db = s.0.lock().map_err(|_| "Database is busy".to_string())?;
    f(&mut db)
}

#[tauri::command]
fn bootstrap(s: State<AppState>) -> Res<Bootstrap> {
    with(&s, |d| d.bootstrap())
}

#[tauri::command]
fn list_transactions(s: State<AppState>, filter: TxFilter) -> Res<Vec<Transaction>> {
    with(&s, |d| d.transactions(&filter))
}

#[tauri::command]
fn save_transaction(s: State<AppState>, tx: Transaction) -> Res<i64> {
    with(&s, |d| d.save_transaction(&tx))
}

#[tauri::command]
fn delete_transaction(s: State<AppState>, id: i64) -> Res<()> {
    with(&s, |d| d.delete_transaction(id))
}

#[tauri::command]
fn summary(s: State<AppState>, month: String) -> Res<Summary> {
    with(&s, |d| d.summary(&month))
}

#[tauri::command]
fn save_account(s: State<AppState>, account: Account) -> Res<i64> {
    with(&s, |d| d.save_account(&account))
}

#[tauri::command]
fn delete_account(s: State<AppState>, id: i64) -> Res<bool> {
    with(&s, |d| d.delete_account(id))
}

#[tauri::command]
fn save_category(s: State<AppState>, category: Category) -> Res<i64> {
    with(&s, |d| d.save_category(&category))
}

#[tauri::command]
fn delete_category(s: State<AppState>, id: i64) -> Res<bool> {
    with(&s, |d| d.delete_category(id))
}

#[tauri::command]
fn set_budget(s: State<AppState>, category_id: i64, amount: i64) -> Res<()> {
    with(&s, |d| d.set_budget(category_id, amount))
}

#[tauri::command]
fn save_recurring(s: State<AppState>, rule: Recurring) -> Res<i64> {
    with(&s, |d| d.save_recurring(&rule))
}

#[tauri::command]
fn delete_recurring(s: State<AppState>, id: i64) -> Res<()> {
    with(&s, |d| d.delete_recurring(id))
}

#[tauri::command]
fn set_setting(s: State<AppState>, key: String, value: String) -> Res<()> {
    with(&s, |d| d.set_setting(&key, &value))
}

#[tauri::command]
fn export_json(s: State<AppState>) -> Res<String> {
    with(&s, |d| serde_json::to_string_pretty(&d.export()?).map_err(|e| e.to_string()))
}

#[tauri::command]
fn import_json(s: State<AppState>, data: String) -> Res<()> {
    let backup: Backup =
        serde_json::from_str(&data).map_err(|e| format!("Not a valid Penny backup: {e}"))?;
    with(&s, |d| d.import(&backup))
}

#[tauri::command]
fn export_csv(s: State<AppState>) -> Res<String> {
    with(&s, |d| d.export_csv())
}

#[tauri::command]
fn wipe_data(s: State<AppState>) -> Res<()> {
    with(&s, |d| d.wipe())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Db::open(&dir.join("penny.db")).map_err(std::io::Error::other)?;
            app.manage(AppState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            list_transactions,
            save_transaction,
            delete_transaction,
            summary,
            save_account,
            delete_account,
            save_category,
            delete_category,
            set_budget,
            save_recurring,
            delete_recurring,
            set_setting,
            export_json,
            import_json,
            export_csv,
            wipe_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running Penny");
}
