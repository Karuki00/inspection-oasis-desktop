import { fetch } from '@tauri-apps/plugin-http';
import { getDb } from './db';

const GOOGLE_APPS_SCRIPT_URL = 'https://script.google.com/macros/s/AKfycbzs0BmnVeZUYE7E2xfZ7A_-O7dlK4u9yJcq6cBcxTaPrAHFOthRF9gmXU1qXgzEKQpn/exec';
const ACTIVE_DAILY_TOKEN = 'OASIS-2026-TOKEN';

export async function syncGoogleSheetToSqlite(): Promise<{ synced: number; skipped: number; errors: number }> {
  let synced = 0;
  let skipped = 0;
  let errors = 0;

  try {
    const syncUrl = `${GOOGLE_APPS_SCRIPT_URL}?action=json`;
    const response = await fetch(syncUrl, { method: 'GET' });

    if (!response.ok) {
      throw new Error(`HTTP Error Status: ${response.status}`);
    }

    const records: any[] = await response.json();

    if (!Array.isArray(records) || records.length === 0) {
      return { synced: 0, skipped: 0, errors: 0 };
    }

    const db = await getDb();

    for (const record of records) {
      // 1. Verify Security Token
      const sheetToken = String(record['Security Token'] || '').trim();
      if (sheetToken && sheetToken !== ACTIVE_DAILY_TOKEN) {
        skipped++;
        continue;
      }

      // 2. Resolve Asset Code
      const assetCodeRaw = record['Asset Code'] || record['Kode Box Hydrant'] || record['asset'] || '';
      const assetCode = String(assetCodeRaw).trim().toUpperCase();

      if (!assetCode) {
        errors++;
        continue;
      }

      const assetRows = await db.select<{ id: number; location_id: number }[]>(
        'SELECT id, location_id FROM assets WHERE code = ?',
        [assetCode]
      );

      if (assetRows.length === 0) {
        errors++;
        continue;
      }

      const asset = assetRows[0];

      // 3. Upsert Inspection Record (INSERT OR REPLACE prevents skips)
      await db.execute(
        `INSERT INTO inspections (google_response_id, asset_id, inspector_badge_id, inspected_at, notes, review_status)
         VALUES (?, ?, ?, ?, ?, 'pending_review')
         ON CONFLICT(google_response_id) DO UPDATE SET
           asset_id = excluded.asset_id,
           inspector_badge_id = excluded.inspector_badge_id,
           inspected_at = excluded.inspected_at,
           notes = excluded.notes;`,
        [
          record.response_id,
          asset.id,
          `TOKEN:${sheetToken || ACTIVE_DAILY_TOKEN}`,
          new Date(record.Timestamp || Date.now()).toISOString(),
          record.Notes || ''
        ]
      );

      // Get Inspection ID for linking component child rows
      const inspectionRes = await db.select<{ id: number }[]>(
        'SELECT id FROM inspections WHERE google_response_id = ?',
        [record.response_id]
      );
      
      if (inspectionRes.length === 0) continue;
      const inspectionId = inspectionRes[0].id;

      // 4. Refresh Component Item Checks for this inspection ID
      await db.execute('DELETE FROM inspection_item_checks WHERE inspection_id = ?', [inspectionId]);

      const components = [
        { name: 'Selang', status: record.Selang },
        { name: 'Nozle', status: record.Nozle || record.Nozzle },
        { name: 'Valve', status: record.Valve },
        { name: 'Lampu', status: record.Lampu },
        { name: 'Bell', status: record.Bell },
        { name: 'J.Inter', status: record['J.Inter'] || record['Jack Inter'] },
        { name: 'Apar', status: record.Apar || record.APAR },
        { name: 'Pressure Gauge', status: record['Pressure Gauge'] },
        { name: 'Kunci pilar', status: record['Kunci pilar'] || record['Kunci Box'] }
      ];

      let hasDefect = false;

      for (const item of components) {
        if (!item.status || item.status === '-') continue;

        await db.execute(
          `INSERT INTO inspection_item_checks (inspection_id, component_name, condition_code, remark)
           VALUES (?, ?, ?, ?)`,
          [inspectionId, item.name, item.status, record.Notes || '']
        );

        if (item.status === 'X' || item.status === 'R') {
          hasDefect = true;
        }
      }

      // 5. Upsert Problems/Defects
      if (hasDefect) {
        const problemCode = `PROB-${record.response_id}`;
        await db.execute(
          `INSERT OR REPLACE INTO problems (problem_code, inspection_id, location_id, category, severity, description, status)
           VALUES (?, ?, ?, 'fire_safety', 'high', ?, 'open')`,
          [
            problemCode,
            inspectionId,
            asset.location_id,
            `Kerusakan pada Box Hydrant ${assetCode}: ${record.Notes || 'Memerlukan perbaikan'}`
          ]
        );

        await db.execute('UPDATE assets SET status = "needs_service" WHERE id = ?', [asset.id]);
      }

      synced++;
    }

    return { synced, skipped, errors };
  } catch (error) {
    console.error('Sync execution failed:', error);
    throw error;
  }
}