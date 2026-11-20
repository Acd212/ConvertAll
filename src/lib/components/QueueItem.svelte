<script lang="ts">
  import {
    AlertTriangle,
    CheckCircle2,
    FileAudio,
    FileImage,
    FileText,
    FileVideo,
    FolderOpen,
    Loader2,
    Play,
    Square,
    X,
  } from "@lucide/svelte";
  import type { FormatDef } from "../types";
  import type { QueueItem } from "../queue";
  import { categoryLabel, formatSize, statusLabel } from "../format";

  let {
    item,
    targets,
    onRemove,
    onConvert,
    onCancel,
    onOpen,
    onTargetChange,
  }: {
    item: QueueItem;
    targets: FormatDef[];
    onRemove: (id: string) => void;
    onConvert: (id: string) => void;
    onCancel: (id: string) => void;
    onOpen: (path: string) => void;
    onTargetChange: (id: string, target: string) => void;
  } = $props();

  const isBusy = $derived(item.status === "converting");
  const canConvert = $derived(
    !isBusy &&
      item.status !== "done" &&
      item.engineAvailable &&
      targets.some((format) => format.id === item.target),
  );

  function statusToneClass(status: QueueItem["status"]): string {
    switch (status) {
      case "done":
        return "border-success/25 bg-success/10 text-success";
      case "converting":
        return "border-accent/25 bg-accent/10 text-accent";
      case "error":
        return "border-danger/25 bg-danger/10 text-danger";
      default:
        return "border-line bg-panel-2 text-muted";
    }
  }
</script>

<li class="rounded-xl border border-line bg-panel p-3 transition-colors hover:border-line/80">
  <div class="flex gap-3">
    <div
      class="mt-0.5 grid h-10 w-10 shrink-0 place-items-center rounded-lg border border-line bg-panel-2 text-muted"
    >
      {#if item.category === "image"}
        <FileImage size={19} strokeWidth={1.7} />
      {:else if item.category === "audio"}
        <FileAudio size={19} strokeWidth={1.7} />
      {:else if item.category === "video"}
        <FileVideo size={19} strokeWidth={1.7} />
      {:else}
        <FileText size={19} strokeWidth={1.7} />
      {/if}
    </div>

    <div class="min-w-0 flex-1">
      <div class="flex items-start gap-3">
        <div class="min-w-0 flex-1">
          <p class="truncate text-sm font-medium" title={item.path}>{item.name}</p>
          <p class="mt-0.5 text-[11px] text-muted">
            {formatSize(item.size)} · {categoryLabel(item.category)}
            {#if !item.engineAvailable}
              <span class="text-danger"> · motor no disponible</span>
            {/if}
          </p>
        </div>

        <div class="flex shrink-0 items-center gap-1">
          {#if item.status === "done" && item.outputPath}
            <button
              type="button"
              onclick={() => onOpen(item.outputPath || "")}
              class="grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-panel-2 hover:text-ink"
              title="Abrir carpeta de salida"
              aria-label="Abrir carpeta de salida"
            >
              <FolderOpen size={15} strokeWidth={1.9} />
            </button>
          {/if}
          <button
            type="button"
            onclick={() => onRemove(item.id)}
            disabled={isBusy}
            class="grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-panel-2 hover:text-danger disabled:cursor-not-allowed disabled:opacity-40"
            title="Quitar de la cola"
            aria-label="Quitar de la cola"
          >
            <X size={15} strokeWidth={2} />
          </button>
        </div>
      </div>

      <div class="mt-3 flex flex-wrap items-center gap-2">
        <label class="sr-only" for={`target-${item.id}`}>Formato de salida</label>
        <select
          id={`target-${item.id}`}
          value={item.target}
          onchange={(event) =>
            onTargetChange(item.id, event.currentTarget.value)}
          disabled={isBusy || targets.length === 0}
          class="h-8 min-w-0 rounded-lg border border-line bg-panel-2 px-2 text-xs text-ink transition-colors hover:border-accent/40 disabled:cursor-not-allowed disabled:opacity-60"
        >
          {#if targets.length === 0}
            <option value="">Sin formatos disponibles</option>
          {:else}
            {#each targets as format (format.id)}
              <option value={format.id}>{format.label} · .{format.extension}</option>
            {/each}
          {/if}
        </select>

        <span
          class={`inline-flex h-7 items-center gap-1 rounded-full border px-2 text-[10px] font-medium ${statusToneClass(item.status)}`}
        >
          {#if item.status === "done"}
            <CheckCircle2 size={11} strokeWidth={2.2} />
          {:else if item.status === "error"}
            <AlertTriangle size={11} strokeWidth={2.2} />
          {:else if item.status === "converting"}
            <Loader2 size={11} strokeWidth={2.2} class="animate-spin" />
          {/if}
          {statusLabel(item.status)}
        </span>

        {#if isBusy}
          <button
            type="button"
            onclick={() => onCancel(item.id)}
            class="ml-auto inline-flex h-8 items-center gap-1.5 rounded-lg border border-danger/30 px-2.5 text-[11px] font-medium text-danger transition-colors hover:bg-danger/10"
          >
            <Square size={11} strokeWidth={2.2} />
            Cancelar
          </button>
        {:else if item.status !== "done"}
          <button
            type="button"
            onclick={() => onConvert(item.id)}
            disabled={!canConvert}
            class="ml-auto inline-flex h-8 items-center gap-1.5 rounded-lg bg-accent px-2.5 text-[11px] font-semibold text-stone-950 transition-colors hover:bg-accent-strong disabled:cursor-not-allowed disabled:opacity-40"
          >
            <Play size={11} strokeWidth={2.4} />
            {item.status === "error" ? "Reintentar" : "Convertir"}
          </button>
        {/if}
      </div>

      {#if item.status === "converting"}
        <div
          class="mt-2.5 h-1.5 overflow-hidden rounded-full bg-panel-2"
          role="progressbar"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={Math.max(0, Math.round(item.progress))}
        >
          <div
            class="h-full rounded-full bg-accent transition-[width] duration-200"
            style:width={`${Math.max(item.progress, 4)}%`}
          ></div>
        </div>
      {/if}

      {#if item.error}
        <p class="mt-2 whitespace-pre-wrap rounded-lg bg-danger/5 px-2 py-1.5 text-[11px] leading-relaxed text-danger">
          {item.error}
        </p>
      {/if}
    </div>
  </div>
</li>
