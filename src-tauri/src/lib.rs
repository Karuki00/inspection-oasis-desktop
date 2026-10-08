// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_sql::{Migration, MigrationKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Define schema migration
    let migration_v1 = Migration {
        version: 1,
        description: "create_initial_inspection_tables",
        sql: "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                badge_id TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                pin_hash TEXT,
                role TEXT CHECK(role IN ('security', 'danru', 'chief_security', 'admin')) NOT NULL DEFAULT 'security',
                employment_status TEXT NOT NULL DEFAULT 'active',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_location_id INTEGER REFERENCES locations(id) ON DELETE SET NULL,
                name TEXT NOT NULL,
                type TEXT CHECK(type IN ('apartment', 'tower', 'floor', 'area')) NOT NULL,
                building_code TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS asset_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT
            );

            CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER NOT NULL REFERENCES asset_categories(id),
                location_id INTEGER NOT NULL REFERENCES locations(id),
                code TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                condition_status TEXT CHECK(condition_status IN ('good', 'fair', 'poor')) DEFAULT 'good',
                status TEXT CHECK(status IN ('operational', 'needs_service', 'under_repair')) DEFAULT 'operational',
                qr_code TEXT UNIQUE,
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS inspections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                google_response_id TEXT UNIQUE,
                asset_id INTEGER NOT NULL REFERENCES assets(id),
                inspector_badge_id TEXT NOT NULL,
                inspected_at DATETIME NOT NULL,
                type TEXT CHECK(type IN ('routine', 'emergency', 'follow_up')) DEFAULT 'routine',
                notes TEXT,
                review_status TEXT CHECK(review_status IN ('pending_review', 'approved', 'rejected')) DEFAULT 'pending_review',
                review_notes TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS problems (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                problem_code TEXT UNIQUE NOT NULL,
                inspection_id INTEGER REFERENCES inspections(id) ON DELETE SET NULL,
                location_id INTEGER NOT NULL REFERENCES locations(id),
                category TEXT CHECK(category IN ('electrical', 'plumbing', 'fire_safety', 'hvac', 'structural', 'general')) DEFAULT 'general',
                severity TEXT CHECK(severity IN ('critical', 'high', 'medium', 'low')) DEFAULT 'medium',
                description TEXT NOT NULL,
                status TEXT CHECK(status IN ('open', 'in_progress', 'resolved', 'escalated')) DEFAULT 'open',
                reported_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        ",
        kind: MigrationKind::Up,
    };

    // 2. Build Tauri App with SQL plugin & migrations
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:oasis_inspection.db", vec![migration_v1])
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
