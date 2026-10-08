<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import QrCard from '../components/QrCard.vue';

const WEB_APP_BASE_URL = ref('https://script.google.com/macros/s/AKfycbzs0BmnVeZUYE7E2xfZ7A_-O7dlK4u9yJcq6cBcxTaPrAHFOthRF9gmXU1qXgzEKQpn/exec');
const SECURITY_TOKEN = ref('OASIS-2026-TOKEN');

// Filters & Search
const selectedCategory = ref('ALL');
const searchQuery = ref('');

// Pagination Controls
const currentPage = ref(1);
const itemsPerPage = ref(6);
const printAllPages = ref(false);
const isPreparingPrint = ref(false);

interface QrItem {
  code: string;
  location: string;
  category: string;
  qrDataUrl?: string;
}

const assets = ref<QrItem[]>([]);

function generateAllAssets(): QrItem[] {
  const list: QrItem[] = [];

  const hldSpecialLocations: Record<number, string> = {
    1: 'Pos belakang', 2: 'Cucian mobil', 3: 'Parkir motor lt. dsr TC',
    4: 'Pos depan / kiri', 5: 'Pos depan / kanan', 6: 'Halte lt. dsr TA',
    7: 'Lapangan tenis TA', 8: 'Toilet lapangan tenis', 9: 'Pos anjungan', 10: 'Kolam renang'
  };
  for (let i = 1; i <= 10; i++) {
    list.push({ code: `HLD-${i}`, location: hldSpecialLocations[i], category: 'Lantai Dasar Special' });
  }

  const hldStandardLocations: Record<number, string> = {
    11: 'Ruang Kasuari TA', 12: 'Pos jaga Lobby TA', 13: 'Pos jaga Lobby TB',
    14: 'Ruang Fitnes', 15: 'Pos jaga Lobby TC', 16: 'Ruang Serbaguna TC'
  };
  for (let i = 11; i <= 16; i++) {
    list.push({ code: `HLD-${i}`, location: hldStandardLocations[i], category: 'Lantai Dasar Standard' });
  }

  const towerFloors = [2, 3, 5, 6, 7, 8, 9, 10, 11, 12, '12A', 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];
  const towers = [
    { prefix: 'HTA', name: 'Tower A', category: 'Tower A' },
    { prefix: 'HTB', name: 'Tower B', category: 'Tower B' },
    { prefix: 'HTC', name: 'Tower C', category: 'Tower C' }
  ];

  for (const tower of towers) {
    for (const fl of towerFloors) {
      for (const suffix of ['A', 'B']) {
        list.push({
          code: `${tower.prefix}-${fl}${suffix}`,
          location: `${tower.name} Lt.${fl} Slot ${suffix}`,
          category: tower.category
        });
      }
    }
  }

  const hb1Locations: Record<number, string> = {
    1: 'B1A pintu lobby lift', 2: 'B1A lot parkir no.341', 3: 'B1A lot parkir no.378',
    4: 'B1A lot parkir no.281', 5: 'B1A lot parkir no.042', 6: 'B1B lot parkir no.229',
    7: 'B1B lot parkir no.310', 8: 'B1B lot parkir no.343', 9: 'B1B lobby lift',
    10: 'B1B lot parkir no.158', 11: 'B1C lot parkir no.074', 12: 'B1C lot parkir no.009',
    13: 'B1C Musholla', 14: 'B1C pintu lobby lift', 15: 'B1C r. tunggu supir',
    16: 'B1C arah klm renang', 17: 'B1C Kontrol room'
  };
  for (let i = 1; i <= 17; i++) {
    list.push({ code: `HB1-${i}`, location: hb1Locations[i] || `Basement 1 Slot ${i}`, category: 'Basement 1' });
  }

  for (let i = 1; i <= 13; i++) {
    list.push({ code: `HB2-${i}`, location: `Basement 2 Slot ${i}`, category: 'Basement 2' });
  }

  return list;
}

const filteredAssets = computed(() => {
  return assets.value.filter(item => {
    const matchesCategory = selectedCategory.value === 'ALL' || item.category === selectedCategory.value;
    const matchesSearch = item.code.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                          item.location.toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchesCategory && matchesSearch;
  });
});

const totalPages = computed(() => {
  return Math.max(1, Math.ceil(filteredAssets.value.length / itemsPerPage.value));
});

const displayedAssets = computed(() => {
  if (printAllPages.value) return filteredAssets.value;
  const start = (currentPage.value - 1) * itemsPerPage.value;
  return filteredAssets.value.slice(start, start + itemsPerPage.value);
});

watch([selectedCategory, searchQuery, itemsPerPage], () => {
  currentPage.value = 1;
});

function getFormUrl(code: string): string {
  return `${WEB_APP_BASE_URL.value}?token=${encodeURIComponent(SECURITY_TOKEN.value)}&asset=${encodeURIComponent(code)}`;
}

function getQrImageUrl(code: string): string {
  const targetUrl = getFormUrl(code);
  return `https://api.qrserver.com/v1/create-qr-code/?size=250x250&data=${encodeURIComponent(targetUrl)}`;
}

function openLink(code: string) {
  window.open(getFormUrl(code), '_blank');
}

async function prepareImagesForPrint() {
  const images = document.querySelectorAll<HTMLImageElement>('.qr-image-el');
  const promises = Array.from(images).map(img => {
    return new Promise<void>((resolve) => {
      if (img.complete && img.naturalWidth !== 0) {
        resolve();
      } else {
        img.onload = () => resolve();
        img.onerror = () => resolve();
      }
    });
  });
  await Promise.all(promises);
}

async function handlePrintCurrentPage() {
  isPreparingPrint.value = true;
  printAllPages.value = false;
  await prepareImagesForPrint();
  isPreparingPrint.value = false;
  window.print();
}

async function handlePrintAllPages() {
  isPreparingPrint.value = true;
  printAllPages.value = true;
  setTimeout(async () => {
    await prepareImagesForPrint();
    isPreparingPrint.value = false;
    window.print();
    printAllPages.value = false;
  }, 300);
}

onMounted(() => {
  assets.value = generateAllAssets();
});
</script>

<template>
  <div class="p-6 space-y-6 bg-slate-50 min-h-screen print:p-0 print:bg-white">
    
    <!-- Controls Header -->
    <div class="print:hidden flex flex-col lg:flex-row lg:items-center justify-between gap-4 bg-white p-5 rounded-xl border border-slate-200 shadow-sm">
      <div>
        <h2 class="text-xl font-bold text-slate-900">Generator & Cetak QR Code Hydrant</h2>
        <p class="text-xs text-slate-500 mt-0.5">
          Menampilkan {{ displayedAssets.length }} dari {{ filteredAssets.length }} Box Hydrant (Halaman {{ currentPage }} dari {{ totalPages }})
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Cari kode (mis. HTA-12A)..."
          class="px-3 py-2 text-xs border border-slate-200 rounded-lg w-44 focus:outline-none focus:ring-2 focus:ring-emerald-500/20"
        />

        <select 
          v-model="selectedCategory" 
          class="px-3 py-2 text-xs border border-slate-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500/20"
        >
          <option value="ALL">Semua Kategori ({{ assets.length }})</option>
          <option value="Lantai Dasar Special">Lantai Dasar Special (HLD 1-10)</option>
          <option value="Lantai Dasar Standard">Lantai Dasar Standard (HLD 11-16)</option>
          <option value="Tower A">Tower A (HTA)</option>
          <option value="Tower B">Tower B (HTB)</option>
          <option value="Tower C">Tower C (HTC)</option>
          <option value="Basement 1">Basement 1 (HB1)</option>
          <option value="Basement 2">Basement 2 (HB2)</option>
        </select>

        <select 
          v-model="itemsPerPage" 
          class="px-3 py-2 text-xs border border-slate-200 rounded-lg bg-white font-semibold"
        >
          <option :value="6">6 per Hal (Ukuran Besar & Mudah Scan)</option>
          <option :value="12">12 per Hal</option>
          <option :value="15">15 per Hal (Ukuran Standar A4)</option>
        </select>

        <div class="flex items-center space-x-2">
          <button 
            @click="handlePrintCurrentPage"
            :disabled="isPreparingPrint"
            class="px-3.5 py-2 bg-[#1c3323] hover:bg-[#25422e] text-white text-xs font-semibold rounded-lg shadow transition disabled:opacity-50"
          >
            <span>{{ isPreparingPrint ? 'Menyiapkan...' : 'Cetak Hal Ini 🖨️' }}</span>
          </button>
          <button 
            @click="handlePrintAllPages"
            :disabled="isPreparingPrint"
            class="px-3.5 py-2 bg-amber-700 hover:bg-amber-800 text-white text-xs font-semibold rounded-lg shadow transition disabled:opacity-50"
          >
            Cetak Semua ({{ filteredAssets.length }})
          </button>
        </div>
      </div>
    </div>

    <!-- Web App Config Bar -->
    <div class="print:hidden bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-center justify-between text-xs text-amber-900 space-x-4">
      <div class="flex-1 space-y-1">
        <label class="font-bold block">Base Web App URL (Google Apps Script)</label>
        <input 
          v-model="WEB_APP_BASE_URL" 
          type="text" 
          class="w-full px-3 py-1.5 bg-white border border-amber-300 rounded font-mono text-[11px]"
        />
      </div>
      <div class="w-48 space-y-1">
        <label class="font-bold block">Token Keamanan</label>
        <input 
          v-model="SECURITY_TOKEN" 
          type="text" 
          class="w-full px-3 py-1.5 bg-white border border-amber-300 rounded font-mono text-[11px]"
        />
      </div>
    </div>

    <!-- Modular Cards Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6 print:grid-cols-2 print:gap-4">
      <QrCard 
        v-for="item in displayedAssets" 
        :key="item.code" 
        :item="item" 
        :qr-image-url="getQrImageUrl(item.code)"
        @open-link="openLink"
      />
    </div>

    <!-- Pagination Controls -->
    <div v-if="totalPages > 1" class="print:hidden flex items-center justify-between bg-white px-5 py-3 rounded-xl border border-slate-200 shadow-sm text-xs font-semibold text-slate-700">
      <div>
        Halaman <span class="text-emerald-700 font-bold">{{ currentPage }}</span> dari {{ totalPages }}
      </div>

      <div class="flex items-center space-x-1">
        <button 
          @click="currentPage--" 
          :disabled="currentPage === 1"
          class="px-3 py-1.5 rounded border border-slate-200 bg-white hover:bg-slate-50 disabled:opacity-40 transition"
        >
          ← Sebelumnya
        </button>

        <div class="flex items-center space-x-1 px-2">
          <button 
            v-for="p in totalPages" 
            :key="p"
            @click="currentPage = p"
            :class="[
              currentPage === p 
                ? 'bg-[#1c3323] text-white font-bold' 
                : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'
            ]"
            class="w-7 h-7 rounded text-xs transition"
          >
            {{ p }}
          </button>
        </div>

        <button 
          @click="currentPage++" 
          :disabled="currentPage === totalPages"
          class="px-3 py-1.5 rounded border border-slate-200 bg-white hover:bg-slate-50 disabled:opacity-40 transition"
        >
          Berikutnya →
        </button>
      </div>
    </div>

  </div>
</template>

<style scoped>
@media print {
  @page {
    margin: 10mm;
  }

  body {
    background: white !important;
  }

  .qr-card {
    break-inside: avoid;
    page-break-inside: avoid;
  }
}
</style>