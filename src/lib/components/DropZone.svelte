<script lang="ts">
  import { FolderOpen, Upload } from "@lucide/svelte";

  let {
    isDragging = false,
    compact = false,
    onBrowse,
    onFolder,
  }: {
    isDragging?: boolean;
    compact?: boolean;
    onBrowse: () => void;
    onFolder: () => void;
  } = $props();
</script>

<div
  class={`relative overflow-hidden rounded-2xl border-2 border-dashed transition-colors ${
    isDragging ? "border-accent bg-accent/5" : "border-line bg-panel/60"
  } ${compact ? "px-4 py-4" : "px-6 py-10"}`}
>
  <div
    class="flex items-center gap-4"
    class:flex-col={!compact}
    class:text-center={!compact}
  >
    <div
      class="grid place-items-center rounded-xl border border-line bg-panel-2 text-accent"
      class:h-11={compact}
      class:w-11={compact}
      class:h-14={!compact}
      class:w-14={!compact}
    >
      <Upload size={compact ? 18 : 24} strokeWidth={1.8} />
    </div>

    <div class:min-w-0={compact}>
      <p class="text-sm font-medium">
        {isDragging ? "Suelta los archivos aquí" : "Arrastra archivos aquí"}
      </p>
      <p class="mt-0.5 text-xs text-muted">
        Imágenes, audio, vídeo y documentos. Nada sale de tu equipo.
      </p>
    </div>

    <div
      class="flex flex-wrap items-center gap-2"
      class:ml-auto={compact}
      class:justify-center={!compact}
      class:mt-4={!compact}
    >
      <button
        type="button"
        onclick={onBrowse}
        class="inline-flex h-9 items-center gap-1.5 rounded-lg bg-accent px-3.5 text-xs font-semibold text-stone-950 transition-colors hover:bg-accent-strong"
      >
        <Upload size={14} strokeWidth={2.2} />
        Archivos
      </button>
      <button
        type="button"
        onclick={onFolder}
        class="inline-flex h-9 items-center gap-1.5 rounded-lg border border-line bg-panel px-3.5 text-xs font-medium text-ink transition-colors hover:border-accent/50 hover:text-accent"
      >
        <FolderOpen size={14} strokeWidth={2} />
        Carpeta
      </button>
    </div>
  </div>
</div>
