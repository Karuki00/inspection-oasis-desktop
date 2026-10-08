<script setup lang="ts">
defineProps<{
  item: {
    code: string;
    location: string;
    category: string;
    qrDataUrl?: string;
  };
  qrImageUrl: string;
}>();

const emit = defineEmits<{
  (e: 'open-link', code: string): void;
}>();
</script>

<template>
  <div class="qr-card bg-white p-5 rounded-2xl border-2 border-slate-200 shadow-sm flex flex-col items-center justify-between text-center print:p-4 print:border-2 print:border-slate-900 print:shadow-none print:rounded-xl">
    <div class="mb-1">
      <p class="text-[10px] font-bold text-emerald-800 tracking-wider uppercase print:text-[9px]">{{ item.category }}</p>
      <h3 class="text-xl font-black text-slate-900 print:text-lg leading-tight">{{ item.code }}</h3>
      <p class="text-xs text-slate-600 font-medium line-clamp-1 mt-1 print:text-[10px]">{{ item.location }}</p>
    </div>

    <!-- High-contrast Large QR Container -->
    <div class="w-44 h-44 bg-white rounded-xl p-2 border-2 border-slate-100 flex items-center justify-center my-2 print:w-36 print:h-36 print:border-none print:p-0">
      <img 
        :src="item.qrDataUrl || qrImageUrl" 
        :alt="`QR Code ${item.code}`"
        class="qr-image-el w-full h-full object-contain"
        loading="eager"
      />
    </div>

    <div class="mt-2 w-full print:hidden">
      <button 
        @click="emit('open-link', item.code)" 
        class="w-full py-1.5 bg-slate-100 hover:bg-slate-200 text-slate-700 text-xs font-semibold rounded-lg transition"
      >
        Buka Form Mobile ↗
      </button>
    </div>

    <p class="hidden print:block text-[8px] font-mono text-slate-500 font-semibold tracking-wider mt-1 uppercase">
      APARTEMEN OASIS MITRA SARANA
    </p>
  </div>
</template>