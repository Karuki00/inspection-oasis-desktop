<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { exit } from '@tauri-apps/api/app';
import DashboardView from './views/DashboardView.vue';
import QrGeneratorView from './views/QrGeneratorView.vue';

// Tab state management
const currentTab = ref<'dashboard' | 'qr'>('dashboard');

// Exit Modal Guard State
const showExitModal = ref(false);
let unlistenClose: (() => void) | null = null;

// Background Auto-Updater check on application launch
async function checkForUpdates() {
  try {
    const update = await check();
    if (update) {
      console.log(`Update available: ${update.version}`);
      await update.downloadAndInstall();
    }
  } catch (error) {
    console.error('Failed to check for desktop updates:', error);
  }
}

// Intercept window close requests (e.g., top-right 'X' button or print overlay)
async function setupCloseGuard() {
  try {
    const appWindow = getCurrentWindow();
    
    unlistenClose = await appWindow.onCloseRequested((event) => {
      // 1. Intercept and block default window destruction
      event.preventDefault();
      
      // 2. Open confirmation modal
      showExitModal.value = true;
    });
  } catch (error) {
    console.error('Failed to set up close guard listener:', error);
  }
}

// Called when user clicks "Ya, Keluar" in the modal
async function confirmExit() {
  showExitModal.value = false;
  
  try {
    // Cleanly terminates the application process via core:app:allow-exit
    await exit(0);
  } catch (err) {
    console.error('Process exit failed, destroying window fallback:', err);
    const appWindow = getCurrentWindow();
    await appWindow.destroy();
  }
}

// Called when user clicks "Batal"
function cancelExit() {
  showExitModal.value = false;
}

onMounted(async () => {
  checkForUpdates();
  await setupCloseGuard();
});

onUnmounted(() => {
  if (unlistenClose) unlistenClose();
});
</script>

<template>
  <main class="relative min-h-screen bg-slate-50 font-sans text-slate-800 antialiased flex flex-col select-none">
    <!-- Top Navigation Switcher (Hidden during printing) -->
    <header class="bg-[#1c3323] text-white px-6 py-2.5 flex items-center justify-between border-b border-emerald-900 print:hidden shrink-0">
      <div class="flex items-center space-x-3">
        <div class="w-6 h-6 rounded bg-white/20 border border-white/30 flex items-center justify-center text-white font-bold text-[10px]">
          OA
        </div>
        <span class="text-xs font-bold tracking-wide">OASIS Inspection System</span>
      </div>

      <nav class="flex space-x-2 text-xs">
        <button 
          @click="currentTab = 'dashboard'" 
          :class="[
            currentTab === 'dashboard' 
              ? 'bg-[#2a4732] text-white font-bold border-b-2 border-amber-400' 
              : 'text-slate-300 hover:text-white hover:bg-[#233f2b]'
          ]"
          class="px-3 py-1.5 rounded-t-md transition flex items-center space-x-1.5"
        >
          <span>Dasbor Utama</span>
        </button>

        <button 
          @click="currentTab = 'qr'" 
          :class="[
            currentTab === 'qr' 
              ? 'bg-[#2a4732] text-white font-bold border-b-2 border-amber-400' 
              : 'text-slate-300 hover:text-white hover:bg-[#233f2b]'
          ]"
          class="px-3 py-1.5 rounded-t-md transition flex items-center space-x-1.5"
        >
          <span>Cetak QR Code</span>
        </button>
      </nav>
    </header>

    <!-- Main View Rendering Area -->
    <div class="flex-1 min-h-0">
      <DashboardView v-if="currentTab === 'dashboard'"/>
      <QrGeneratorView v-else-if="currentTab === 'qr'"/>
    </div>

    <!-- Exit Confirmation Modal Backdrop -->
    <div 
      v-if="showExitModal" 
      class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/60 backdrop-blur-sm p-4 animate-fade-in print:hidden"
    >
      <div class="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-sm w-full p-6 text-center space-y-4">
        
        <!-- Warning Icon -->
        <div class="w-12 h-12 bg-amber-100 text-amber-700 rounded-full flex items-center justify-center mx-auto text-xl font-bold">
          ⚠️
        </div>

        <!-- Text Content -->
        <div class="space-y-1">
          <h3 class="text-lg font-extrabold text-slate-900">Keluar dari Aplikasi?</h3>
          <p class="text-xs text-slate-500 leading-relaxed">
            Apakah Anda yakin ingin menutup aplikasi Sistem Inspeksi OASIS?
          </p>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center space-x-3 pt-2">
          <button 
            @click="cancelExit" 
            class="flex-1 py-2.5 bg-slate-100 hover:bg-slate-200 text-slate-700 text-xs font-semibold rounded-xl transition"
          >
            Batal
          </button>
          <button 
            @click="confirmExit" 
            class="flex-1 py-2.5 bg-red-600 hover:bg-red-700 text-white text-xs font-semibold rounded-xl shadow-sm transition"
          >
            Ya, Keluar
          </button>
        </div>

      </div>
    </div>
  </main>
</template>

<style>
@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
.animate-fade-in {
  animation: fadeIn 0.15s ease-out forwards;
}
</style>