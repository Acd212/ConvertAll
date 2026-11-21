<script lang="ts">
  import { AlertTriangle, FolderOpen } from "@lucide/svelte";
  import type { EngineInfo } from "../types";
  import { visibleEngines } from "../engines";

  let {
    engines,
    outputDir = $bindable(""),
    quality = $bindable(80),
    preserveMetadata = $bindable(true),
    overwrite = $bindable(false),
    openAfter = $bindable(true),
    pandocMissing = false,
    onPickOutput,
  }: {
    engines: EngineInfo[];
    outputDir?: string;
    quality?: number;
    preserveMetadata?: boolean;
    overwrite?: boolean;
    openAfter?: boolean;
    pandocMissing?: boolean;
    onPickOutput: () => void;
  } = $props();

  const displayed = $derived(visibleEngines(engines));
</script>

<aside class="space-y-3">
  <section class="rounded-xl border border-line bg-panel p-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-muted">Salida</h2>
    <div class="mt-3 flex gap-2">
      <input
        type="text"
        bind:value={outputDir}
        spellcheck="false"
        aria-label="Carpeta de salida"
        class="h-9 min-w-0 flex-1 rounded-lg border border-line bg-panel-2 px-3 text-[11px] text-ink outline-none transition-colors placeholder:text-muted focus:border-accent/60"
        placeholder="Carpeta de destino"
      />
      <button
        type="button"
        onclick={onPickOutput}
        class="grid h-9 w-9 shrink-0 place-items-center rounded-lg border border-line bg-panel-2 text-muted transition-colors hover:border-accent/50 hover:text-accent"
        title="Elegir carpeta de salida"
        aria-label="Elegir carpeta de salida"
      >
        <FolderOpen size={15} strokeWidth={1.9} />
      </button>
    </div>
    <p class="mt-2 text-[10px] leading-relaxed text-muted">
      Si la carpeta está vacía, se usará la carpeta del archivo original.
    </p>
  </section>

  <section class="rounded-xl border border-line bg-panel p-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-muted">Calidad</h2>
      <span class="text-xs font-semibold text-accent">{quality}</span>
    </div>
    <input
      type="range"
      min="1"
      max="100"
      step="1"
      bind:value={quality}
      aria-label="Calidad de conversión"
      class="mt-3 w-full"
      style="accent-color: var(--color-accent)"
    />
    <div class="mt-1 flex justify-between text-[10px] text-muted">
      <span>Menor tamaño</span>
      <span>Mayor calidad</span>
    </div>
    <p class="mt-2 text-[10px] leading-relaxed text-muted">
      Se aplica a imágenes y a la codificación de audio/vídeo. Los documentos no se ven afectados.
    </p>
  </section>

  <section class="rounded-xl border border-line bg-panel p-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-muted">Opciones</h2>
    <div class="mt-3 space-y-2.5">
      <label class="flex cursor-pointer items-center gap-2.5 text-xs text-ink">
        <input
          type="checkbox"
          bind:checked={preserveMetadata}
          class="h-3.5 w-3.5 rounded border-line bg-panel-2"
          style="accent-color: var(--color-accent)"
        />
        Conservar metadatos
      </label>
      <label class="flex cursor-pointer items-center gap-2.5 text-xs text-ink">
        <input
          type="checkbox"
          bind:checked={overwrite}
          class="h-3.5 w-3.5 rounded border-line bg-panel-2"
          style="accent-color: var(--color-accent)"
        />
        Sobrescribir si existe
      </label>
      <label class="flex cursor-pointer items-center gap-2.5 text-xs text-ink">
        <input
          type="checkbox"
          bind:checked={openAfter}
          class="h-3.5 w-3.5 rounded border-line bg-panel-2"
          style="accent-color: var(--color-accent)"
        />
        Abrir carpeta al terminar
      </label>
    </div>
  </section>

  <section class="rounded-xl border border-line bg-panel p-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-muted">Motores</h2>
    <div class="mt-3 space-y-2">
      {#each displayed as engine (engine.id)}
        <div class="flex items-center justify-between gap-2">
          <span class="flex items-center gap-2 text-[11px] text-ink">
            <span
              class={`h-1.5 w-1.5 rounded-full ${
                engine.available ? "bg-success" : "bg-danger"
              }`}
            ></span>
            {engine.name}
          </span>
          <span
            class="max-w-[150px] truncate text-[10px] text-muted"
            title={engine.available
              ? (engine.version ?? engine.path ?? "")
              : "No disponible"}
          >
            {engine.available ? (engine.version ?? "Disponible") : "No disponible"}
          </span>
        </div>
      {/each}
    </div>

    {#if pandocMissing}
      <div class="mt-3 flex gap-2 rounded-lg border border-amber-500/25 bg-amber-500/10 p-2.5 text-[10px] leading-relaxed text-ink">
        <AlertTriangle size={14} class="mt-0.5 shrink-0 text-amber-500" strokeWidth={2} />
        <span>
          Pandoc no está instalado. Instálalo con el gestor de paquetes de tu
          distribución (por ejemplo,
          <code class="mx-1 rounded bg-panel-2 px-1 py-0.5 font-mono text-[10px]">sudo apt install pandoc</code>
          o
          <code class="mx-1 rounded bg-panel-2 px-1 py-0.5 font-mono text-[10px]">sudo pacman -S pandoc</code>).
          En Windows viene incluido en el paquete portable.
        </span>
      </div>
    {/if}
  </section>
</aside>
