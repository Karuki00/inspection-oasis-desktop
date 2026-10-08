use tauri_plugin_sql::{Migration, MigrationKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migration_v1 = Migration {
        version: 1,
        description: "create_initial_inspection_tables",
        sql: "
            CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                building_code TEXT
            );

            CREATE TABLE IF NOT EXISTS asset_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT
            );

            CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER NOT NULL,
                location_id INTEGER NOT NULL,
                code TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                condition_status TEXT DEFAULT 'good',
                status TEXT DEFAULT 'operational',
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS inspections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                google_response_id TEXT UNIQUE,
                asset_id INTEGER NOT NULL,
                inspector_badge_id TEXT NOT NULL,
                inspected_at DATETIME NOT NULL,
                notes TEXT,
                review_status TEXT DEFAULT 'pending_review'
            );

            CREATE TABLE IF NOT EXISTS inspection_item_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                inspection_id INTEGER NOT NULL,
                component_name TEXT NOT NULL,
                condition_code TEXT NOT NULL,
                remark TEXT
            );

            CREATE TABLE IF NOT EXISTS problems (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                problem_code TEXT UNIQUE NOT NULL,
                inspection_id INTEGER,
                location_id INTEGER NOT NULL,
                category TEXT DEFAULT 'general',
                severity TEXT DEFAULT 'medium',
                description TEXT NOT NULL,
                status TEXT DEFAULT 'open',
                reported_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        ",
        kind: MigrationKind::Up,
    };

    // Incremental Migration v2: Adds qr_code_base64 column safely
    let migration_v2 = Migration {
        version: 2,
        description: "add_qr_code_base64_to_assets",
        sql: "ALTER TABLE assets ADD COLUMN qr_code_base64 TEXT;",
        kind: MigrationKind::Up,
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:oasis_inspection.db", vec![migration_v1, migration_v2])
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}