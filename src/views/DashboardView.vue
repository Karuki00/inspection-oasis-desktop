<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Sidebar from '../components/Sidebar.vue';
import { getDb, fetchDashboardStats, fetchReportedProblems, seedAllHydrantAssets } from '../services/db';
import { syncGoogleSheetToSqlite } from '../services/syncService';
import type { StatMetric, FacilityProblem } from '../types/inspection';

interface SpreadsheetInspectionRow {
  id: number;
  response_id: string;
  inspected_at: string;
  token: string;
  asset_code: string;
  selang: string;
  nozle: string;
  valve: string;
  lampu: string;
  bell: string;
  jack_inter: string;
  apar: string;
  pressure_gauge: string;
  kunci_pilar: string;
  notes: string;
}

// Reactive States
const isSyncing = ref(false);
const syncMessage = ref('');
//const searchQuery = ref('');
const loading = ref(false);

// Reactive Database Data
const stats = ref<StatMetric[]>([]);
const problems = ref<FacilityProblem[]>([]);
const spreadsheetRows = ref<SpreadsheetInspectionRow[]>([]);

// Fetch real data from SQLite database matching exact spreadsheet columns
async function loadData() {
  loading.value = true;
  try {
    const db = await getDb();

    stats.value = await fetchDashboardStats();
    problems.value = await fetchReportedProblems();

    // Pivot query to map SQLite child check rows to spreadsheet columns
    const rows = await db.select<SpreadsheetInspectionRow[]>(`
      SELECT 
        i.id,
        i.google_response_id as response_id,
        i.inspected_at,
        i.inspector_badge_id as token,
        a.code as asset_code,
        MAX(CASE WHEN c.component_name IN ('Selang') THEN c.condition_code END) as selang,
        MAX(CASE WHEN c.component_name IN ('Nozle', 'Nozzle') THEN c.condition_code END) as nozle,
        MAX(CASE WHEN c.component_name IN ('Valve') THEN c.condition_code END) as valve,
        MAX(CASE WHEN c.component_name IN ('Lampu') THEN c.condition_code END) as lampu,
        MAX(CASE WHEN c.component_name IN ('Bell') THEN c.condition_code END) as bell,
        MAX(CASE WHEN c.component_name IN ('J.Inter', 'Jack Inter') THEN c.condition_code END) as jack_inter,
        MAX(CASE WHEN c.component_name IN ('Apar', 'APAR') THEN c.condition_code END) as apar,
        MAX(CASE WHEN c.component_name IN ('Pressure Gauge') THEN c.condition_code END) as pressure_gauge,
        MAX(CASE WHEN c.component_name IN ('Kunci pilar', 'Kunci Box') THEN c.condition_code END) as kunci_pilar,
        i.notes
      FROM inspections i
      JOIN assets a ON i.asset_id = a.id
      LEFT JOIN inspection_item_checks c ON i.id = c.inspection_id
      GROUP BY i.id
      ORDER BY i.inspected_at DESC
      LIMIT 20
    `);

    spreadsheetRows.value = rows;
  } catch (err) {
    console.error('Error fetching SQLite data:', err);
  } finally {
    loading.value = false;
  }
}

// Standard Sync
async function handleSync() {
  isSyncing.value = true;
  syncMessage.value = 'Syncing with Google Sheets...';
  
  try {
    const result = await syncGoogleSheetToSqlite();
    syncMessage.value = `Sync Complete: ${result.synced} new records, ${result.skipped} skipped.`;
    await loadData();
  } catch (err) {
    syncMessage.value = 'Sync failed. Check internet connection.';
    console.error('Sync Error:', err);
  } finally {
    isSyncing.value = false;
  }
}

// Force Re-sync: Clears SQLite inspection tables and re-pulls everything
async function handleForceResync() {
  if (!confirm('Re-sync will clear local inspection records in SQLite and re-pull all data from Google Sheets. Continue?')) {
    return;
  }

  isSyncing.value = true;
  syncMessage.value = 'Clearing SQLite & re-pulling from Google...';
  
  try {
    const db = await getDb();
    await db.execute('DELETE FROM inspection_item_checks;');
    await db.execute('DELETE FROM problems;');
    await db.execute('DELETE FROM inspections;');

    const result = await syncGoogleSheetToSqlite();
    syncMessage.value = `Reset & Sync Complete: ${result.synced} records pulled!`;
    await loadData();
  } catch (err) {
    syncMessage.value = 'Reset sync failed.';
    console.error('Force Resync Error:', err);
  } finally {
    isSyncing.value = false;
  }
}

function getCellClass(val: string | null): string {
  switch (val) {
    case 'V': return 'bg-emerald-100 text-emerald-800 font-bold';
    case 'X': return 'bg-red-100 text-red-800 font-bold';
    case 'R': return 'bg-amber-100 text-amber-800 font-bold';
    default: return 'text-slate-300';
  }
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '-';
  return new Date(dateStr).toLocaleString('id-ID', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

onMounted(async () => {
  await seedAllHydrantAssets();
  await loadData();
});
</script>

<template>
  <div class="flex h-screen bg-[#f8f9fa] font-sans text-slate-800 antialiased overflow-hidden">
    
    <Sidebar/>

    <div class="flex-1 flex flex-col min-w-0 overflow-y-auto">
      
      <!-- Top Bar Controls -->
      <header class="p-6 pb-0 flex items-center justify-between">
        <div>
          <h2 class="text-2xl font-bold text-slate-900">Dashboard</h2>
          <p class="text-xs text-slate-400 mt-0.5">Apartemen OASIS Mitra Sarana • Portal Inspeksi & Fasilitas</p>
        </div>

        <div class="flex items-center space-x-3">
          <span v-if="syncMessage" class="text-xs text-slate-500 font-medium">{{ syncMessage }}</span>

          <!-- Reload Button -->
          <button 
            @click="loadData" 
            :disabled="loading"
            class="p-2 bg-white border border-slate-200 rounded-lg text-slate-500 hover:bg-slate-50 shadow-sm disabled:opacity-50"
            title="Reload Local SQLite Data"
          >
            <svg class="w-4 h-4" :class="{ 'animate-spin': loading }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path></svg>
          </button>

          <!-- Normal Sync Button -->
          <button 
            @click="handleSync" 
            :disabled="isSyncing"
            class="px-3.5 py-2 bg-[#1c3323] hover:bg-[#25422e] text-white text-xs font-medium rounded-lg shadow-sm transition disabled:opacity-50"
          >
            <span>{{ isSyncing ? 'Syncing...' : 'Sync Data' }}</span>
          </button>

          <!-- Force Reset & Re-sync Button -->
          <button 
            @click="handleForceResync" 
            :disabled="isSyncing"
            class="px-3 py-2 bg-amber-700 hover:bg-amber-800 text-white text-xs font-medium rounded-lg shadow-sm transition disabled:opacity-50"
            title="Clear SQLite inspection logs and re-pull all records from Google Sheets"
          >
            Force Re-sync 🔄
          </button>
        </div>
      </header>

      <!-- Main Content Area -->
      <div class="p-6 space-y-6">
        
        <!-- Metrics -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div v-for="(stat, idx) in stats" :key="idx" class="p-5 bg-white rounded-xl border border-slate-200/80 shadow-sm flex items-start justify-between">
            <div>
              <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">{{ stat.label }}</p>
              <h3 class="text-2xl font-extrabold text-slate-900 mt-2">{{ stat.value }}</h3>
            </div>
          </div>
        </div>

        <!-- Google Sheets Submissions Mirror Table -->
        <div class="bg-white rounded-xl border border-slate-200/80 shadow-sm p-5">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-bold text-slate-900">Hasil Inspeksi Hydrant (Live SQLite)</h3>
            <span class="text-xs text-slate-400">Synced from Google Sheets</span>
          </div>

          <div class="overflow-x-auto">
            <table class="w-full text-center border-collapse text-xs">
              <thead>
                <tr class="bg-[#fcfaf7] text-slate-500 text-[10px] uppercase font-bold border-b border-slate-200 tracking-wider">
                  <th class="py-3 px-2 text-left">Timestamp</th>
                  <th class="py-3 px-2 text-left">Security Token</th>
                  <th class="py-3 px-2 text-left">Asset Code</th>
                  <th class="py-3 px-1 border-l">Selang</th>
                  <th class="py-3 px-1">Nozle</th>
                  <th class="py-3 px-1">Valve</th>
                  <th class="py-3 px-1">Lampu</th>
                  <th class="py-3 px-1">Bell</th>
                  <th class="py-3 px-1">J.Inter</th>
                  <th class="py-3 px-1">Apar</th>
                  <th class="py-3 px-1">Pressure Gauge</th>
                  <th class="py-3 px-1 border-r">Kunci pilar</th>
                  <th class="py-3 px-2 text-left">Notes</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100 font-medium text-slate-700">
                <tr v-for="row in spreadsheetRows" :key="row.id" class="hover:bg-slate-50 transition">
                  <td class="py-3 px-2 text-left text-slate-500 text-[11px] whitespace-nowrap">{{ formatDate(row.inspected_at) }}</td>
                  <td class="py-3 px-2 text-left text-slate-600 font-mono text-[11px]">{{ row.token }}</td>
                  <td class="py-3 px-2 text-left font-bold text-emerald-950">{{ row.asset_code }}</td>
                  
                  <!-- Equipment Condition Cells -->
                  <td class="py-3 px-1 border-l" :class="getCellClass(row.selang)">{{ row.selang || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.nozle)">{{ row.nozle || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.valve)">{{ row.valve || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.lampu)">{{ row.lampu || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.bell)">{{ row.bell || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.jack_inter)">{{ row.jack_inter || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.apar)">{{ row.apar || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.pressure_gauge)">{{ row.pressure_gauge || '-' }}</td>
                  <td class="py-3 px-1 border-r" :class="getCellClass(row.kunci_pilar)">{{ row.kunci_pilar || '-' }}</td>
                  
                  <td class="py-3 px-2 text-left text-slate-500 italic max-w-xs truncate">{{ row.notes || '-' }}</td>
                </tr>
                <tr v-if="spreadsheetRows.length === 0">
                  <td colspan="13" class="py-8 text-center text-slate-400">
                    No inspection records found in local database. Click <strong>Sync Data</strong> or <strong>Force Re-sync</strong>.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Reported Defect Summary -->
        <div class="bg-white rounded-xl border border-slate-200/80 shadow-sm p-5">
          <h3 class="text-sm font-bold text-slate-900 mb-3">Recent Reported Defects</h3>
          <div class="divide-y divide-slate-100">
            <div v-for="(prob, i) in problems" :key="i" class="py-2.5 flex items-center justify-between">
              <div>
                <h4 class="text-xs font-bold text-slate-900">{{ prob.title }}</h4>
                <p class="text-[11px] text-slate-400">{{ prob.location }}</p>
              </div>
              <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase text-red-600 border border-red-200 bg-red-50">
                {{ prob.severity }}
              </span>
            </div>
            <div v-if="problems.length === 0" class="py-4 text-center text-slate-400 text-xs">
              No active defects reported.
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>