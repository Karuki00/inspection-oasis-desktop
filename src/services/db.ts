import Database from '@tauri-apps/plugin-sql';

let dbInstance: Database | null = null;

/**
 * Initializes and returns the local SQLite database instance.
 */
export async function getDb(): Promise<Database> {
  if (!dbInstance) {
    dbInstance = await Database.load('sqlite:oasis_inspection.db');
  }
  return dbInstance;
}

/**
 * Fetches dashboard statistics summary.
 */
export async function getDashboardStats() {
  const db = await getDb();
  
  const totalAssets = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM assets WHERE is_active = 1');
  const openIssues = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM problems WHERE status = "open"');
  const pendingReviews = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM inspections WHERE review_status = "pending_review"');

  return {
    totalAssets: totalAssets[0]?.count || 0,
    openIssues: openIssues[0]?.count || 0,
    pendingReviews: pendingReviews[0]?.count || 0,
  };
}

/**
 * Example Insert Query with Parameter Binding
 */
export async function insertAsset(code: string, name: string, categoryId: number, locationId: number) {
  const db = await getDb();
  return await db.execute(
    'INSERT INTO assets (code, name, category_id, location_id) VALUES (?, ?, ?, ?)',
    [code, name, categoryId, locationId]
  );
}