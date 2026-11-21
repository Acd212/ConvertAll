<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    Files,
    Loader2,
    Moon,
    Play,
    RefreshCw,
    ShieldCheck,
    Square,
    Sun,
    Trash2,
    Zap,
  } from "@lucide/svelte";
  import { api } from "./lib/api";
  import type {
    Category,
    EngineInfo,
    FileInfo,
    FormatDef,
    ProgressPayload,
  } from "./lib/types";
  import type { QueueItem } from "./lib/queue";
  import { formatSize, categoryLabel } from "./lib/format";
  import Logo from "./lib/components/Logo.svelte";
  import EngineChips from "./lib/components/EngineChips.svelte";
  import DropZone from "./lib/components/DropZone.svelte";
  import QueueItemCard from "./lib/components/QueueItem.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";

  type Theme = "dark" | "light";

  const emptyTargets = (): Record<Category, FormatDef[]> => ({
    image: [],
    audio: [],
    video: [],
    document: [],
    unknown: [],
  });

  let engines = $state<EngineInfo[]>([]);
  let catalog = $state<FormatDef[]>([]);
  let targets = $state<Record<Category, FormatDef[]>>(emptyTargets());
  let items = $state<QueueItem[]>([]);
  let outputDir = $state("");
  let quality = $state(80);
  let preserveMetadata = $state(true);
  let overwrite = $state(false);
  let openAfter = $state(true);
  let theme = $state<Theme>("dark");
  let isDragging = $state(false);
  let toast = $state<{ message: string; tone: "info" | "error" } | null>(null);
  let busy = $state(false);
  let initialized = $state(false);

  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const pendingCount = $derived(
    items.filter((item) => item.status === "pending").length,
  );
  const convertingCount = $derived(
    items.filter((item) => item.status === "converting").length,
  );
  const doneCount = $derived(
    items.filter((item) => item.status === "done").length,
  );
  const inputFormats = $derived(catalog.filter((format) => format.canRead).length);
  const pandocMissing = $derived(
    engines.length > 0 &&
      !engines.some((engine) => engine.id === "pandoc" && engine.available),
  );
  const canStart = $derived(
    items.some(
      (item) =>
        (item.status === "pending" || item.status === "error") &&
        item.target.length > 0 &&
        (targets[item.category] ?? []).some(
          (format) => format.id === item.target,
        ),
    ),
  );

  $effect(() => {
    document.documentElement.dataset.theme = theme;
  });

  function showToast(message: string, tone: "info" | "error" = "info") {
    toast = { message, tone };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast = null;
    }, 5000);
  }

  function newId(): string {
    if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
      return crypto.randomUUID();
    }
    return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  async function loadInitialData() {
    try {
      const [engineList, catalogList, defaultDir] = await Promise.all([
        api.detectEngines(),
        api.getCatalog(),
        api.defaultOutputDir(),
      ]);

      engines = engineList;
      catalog = catalogList;
      if (!outputDir) outputDir = defaultDir;

      const categories: Category[] = ["image", "audio", "video", "document"];
      const entries = await Promise.all(
        categories.map(async (category) => {
          const formats = await api.getTargets(category);
          return [category, formats] as const;
        }),
      );

      targets = {
        ...emptyTargets(),
        ...(Object.fromEntries(entries) as Record<Category, FormatDef[]>),
      };

      const initialPaths = await api.startupFiles();
      if (initialPaths.length > 0) {
        await addPaths(initialPaths);
      }
    } catch (error) {
      showToast(String(error), "error");
    } finally {
      initialized = true;
    }
  }

  async function refreshEngines() {
    try {
      engines = await api.detectEngines();
      const categories: Category[] = ["image", "audio", "video", "document"];
      const entries = await Promise.all(
        categories.map(async (category) => {
          const formats = await api.getTargets(category);
          return [category, formats] as const;
        }),
      );
      targets = {
        ...emptyTargets(),
        ...(Object.fromEntries(entries) as Record<Category, FormatDef[]>),
      };
      showToast("Estado de los motores actualizado.");
    } catch (error) {
      showToast(String(error), "error");
    }
  }

  function handleProgress(payload: ProgressPayload) {
    const item = items.find((value) => value.id === payload.jobId);
    if (!item) return;
    item.progress = payload.progress;
    if (payload.stage === "completado") item.progress = 100;
  }

  function handleDragDrop(event: {
    payload:
      | { type: "enter" | "drop"; paths: string[] }
      | { type: "over" | "leave" };
  }) {
    if (event.payload.type === "enter" || event.payload.type === "over") {
      isDragging = true;
    } else if (event.payload.type === "leave") {
      isDragging = false;
    } else if (event.payload.type === "drop") {
      isDragging = false;
      void addPaths(event.payload.paths);
    }
  }

  onMount(() => {
    let unlistenDrop: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;

    void loadInitialData();

    void (async () => {
      try {
        unlistenProgress = await api.onProgress(handleProgress);
        unlistenDrop = await getCurrentWebview().onDragDropEvent(handleDragDrop);
      } catch (error) {
        showToast(String(error), "error");
      }
    })();

    return () => {
      unlistenDrop?.();
      unlistenProgress?.();
    };
  });

  async function inspectInBatches(paths: string[]) {
    const results: PromiseSettledResult<FileInfo>[] = [];
    for (let index = 0; index < paths.length; index += 8) {
      const chunk = paths.slice(index, index + 8);
      results.push(
        ...(await Promise.allSettled(chunk.map((path) => api.inspectFile(path)))),
      );
    }
    return results;
  }

  async function addPaths(paths: string[]) {
    const seen = new Set(items.map((item) => item.path));
    const unique = paths.filter((path) => {
      if (seen.has(path)) return false;
      seen.add(path);
      return true;
    });

    if (!unique.length) return;

    const results = await inspectInBatches(unique);
    const next: QueueItem[] = [];
    let skipped = 0;

    for (const result of results) {
      if (result.status === "rejected") {
        skipped += 1;
        continue;
      }

      const info = result.value;
      const availableTargets = targets[info.category] ?? [];

      if (info.category === "unknown") {
        skipped += 1;
        continue;
      }

      const target = availableTargets.some(
        (format) => format.id === info.suggestedTarget,
      )
        ? info.suggestedTarget
        : (availableTargets[0]?.id ?? "");

      if (!target) {
        skipped += 1;
        continue;
      }

      next.push({
        ...info,
        id: newId(),
        target,
        status: "pending",
        progress: 0,
      });
    }

    if (next.length) {
      items = [...items, ...next];
    }

    if (skipped > 0) {
      showToast(
        `${skipped} archivo(s) omitido(s): formato no compatible o motor no disponible.`,
      );
    }
  }

  async function browseFiles() {
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Selecciona archivos para convertir",
    });

    if (!selected) return;
    await addPaths(Array.isArray(selected) ? selected : [selected]);
  }

  async function browseFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Selecciona una carpeta",
    });

    if (typeof selected !== "string") return;

    try {
      const paths = await api.scanFolder(selected);
      if (paths.length === 0) {
        showToast("La carpeta no contiene archivos.");
        return;
      }
      if (paths.length >= 1000) {
        showToast("Se añadieron los primeros 1000 archivos de la carpeta.");
      }
      await addPaths(paths);
    } catch (error) {
      showToast(String(error), "error");
    }
  }

  async function pickOutputDir() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Selecciona la carpeta de salida",
      defaultPath: outputDir || undefined,
    });

    if (typeof selected === "string") {
      outputDir = selected;
    }
  }

  function isCancelled(item: QueueItem): boolean {
    return item.status === "cancelled";
  }

  function onTargetChange(id: string, target: string) {
    const item = items.find((value) => value.id === id);
    if (item) item.target = target;
  }

  async function convertItem(item: QueueItem) {
    if (item.status === "converting" || !item.target) return;

    item.status = "converting";
    item.progress = 0;
    item.error = undefined;

    try {
      const result = await api.convertFile({
        jobId: item.id,
        inputPath: item.path,
        outputDir,
        targetFormat: item.target,
        quality,
        preserveMetadata,
        overwrite,
      });

      if (isCancelled(item)) return;

      item.status = "done";
      item.progress = 100;
      item.outputPath = result.outputPath;
    } catch (error) {
      if (isCancelled(item)) return;
      item.status = "error";
      item.error = typeof error === "string" ? error : String(error);
    }
  }

  async function runQueue(queue: QueueItem[], concurrency: number) {
    let index = 0;

    const workers = Array.from(
      { length: Math.min(concurrency, queue.length) },
      async () => {
        while (index < queue.length) {
          const item = queue[index++];
          if (!item || item.status === "done" || item.status === "converting") {
            continue;
          }
          await convertItem(item);
        }
      },
    );

    await Promise.all(workers);
  }

  async function startAll() {
    if (busy) return;

    const queue = items.filter(
      (item) =>
        (item.status === "pending" || item.status === "error") &&
        item.target.length > 0 &&
        (targets[item.category] ?? []).some(
          (format) => format.id === item.target,
        ),
    );

    if (!queue.length) return;

    busy = true;
    try {
      await runQueue(queue, 2);
    } finally {
      busy = false;
      if (openAfter && queue.some((item) => item.status === "done")) {
        const directory = outputDir.trim();
        if (directory) {
          await api.openInFileManager(directory).catch(() => undefined);
        }
      }
    }
  }

  function convertOne(id: string) {
    const item = items.find((value) => value.id === id);
    if (item) void convertItem(item);
  }

  async function cancelItem(id: string) {
    const item = items.find((value) => value.id === id);
    if (!item) return;
    item.status = "cancelled";
    await api.cancelJob(id).catch(() => undefined);
  }

  async function cancelAll() {
    const active = items.filter((item) => item.status === "converting");
    await Promise.all(active.map((item) => cancelItem(item.id)));
  }

  async function removeItem(id: string) {
    const item = items.find((value) => value.id === id);
    if (item?.status === "converting") {
      await api.cancelJob(id).catch(() => undefined);
    }
    items = items.filter((value) => value.id !== id);
  }

  function clearFinished() {
    items = items.filter(
      (item) => item.status === "pending" || item.status === "converting",
    );
  }

  async function clearAll() {
    await cancelAll();
    items = [];
  }

  async function openOutput(path: string) {
    await api.openInFileManager(path).catch((error) => {
      showToast(String(error), "error");
    });
  }

  function targetsFor(item: QueueItem): FormatDef[] {
    return targets[item.category] ?? [];
  }
</script>

<div class="min-h-screen bg-canvas text-ink">
  <header class="sticky top-0 z-20 border-b border-line bg-canvas/85 backdrop-blur">
    <div class="mx-auto flex max-w-7xl items-center gap-4 px-6 py-3">
      <Logo size={34} />
      <div class="min-w-0">
        <h1 class="text-base font-semibold tracking-tight">ConvertAll</h1>
        <p class="hidden text-[11px] text-muted sm:block">
          Conversión local. Tus archivos no salen de tu equipo.
        </p>
      </div>

      <div class="ml-auto flex items-center gap-2">
        <div class="hidden lg:block">
          <EngineChips {engines} />
        </div>
        <button
          type="button"
          onclick={refreshEngines}
          class="grid h-8 w-8 place-items-center rounded-lg border border-line bg-panel text-muted transition-colors hover:border-accent/50 hover:text-accent"
          title="Actualizar estado de los motores"
          aria-label="Actualizar estado de los motores"
        >
          <RefreshCw size={14} strokeWidth={2} />
        </button>
        <button
          type="button"
          onclick={() => (theme = theme === "dark" ? "light" : "dark")}
          class="grid h-8 w-8 place-items-center rounded-lg border border-line bg-panel text-muted transition-colors hover:border-accent/50 hover:text-accent"
          title={theme === "dark" ? "Cambiar a tema claro" : "Cambiar a tema oscuro"}
          aria-label={theme === "dark" ? "Cambiar a tema claro" : "Cambiar a tema oscuro"}
        >
          {#if theme === "dark"}
            <Sun size={14} strokeWidth={2} />
          {:else}
            <Moon size={14} strokeWidth={2} />
          {/if}
        </button>
      </div>
    </div>
  </header>

  <main class="mx-auto max-w-7xl px-6 py-6 pb-28">
    {#if !initialized}
      <div class="flex items-center gap-2 py-16 text-sm text-muted">
        <Loader2 size={16} class="animate-spin" />
        Detectando motores y formatos…
      </div>
    {:else}
      <div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_340px]">
        <div class="min-w-0 space-y-4">
          <DropZone
            {isDragging}
            compact={items.length > 0}
            onBrowse={browseFiles}
            onFolder={browseFolder}
          />

          {#if items.length === 0}
            <section class="grid gap-3 sm:grid-cols-3">
              <article class="rounded-xl border border-line bg-panel p-4">
                <ShieldCheck size={18} class="text-success" strokeWidth={1.9} />
                <h2 class="mt-3 text-xs font-semibold">Privado por diseño</h2>
                <p class="mt-1 text-[11px] leading-relaxed text-muted">
                  Sin subidas, sin cuentas y sin telemetría. Todo ocurre en tu equipo.
                </p>
              </article>
              <article class="rounded-xl border border-line bg-panel p-4">
                <Zap size={18} class="text-accent" strokeWidth={1.9} />
                <h2 class="mt-3 text-xs font-semibold">Motores nativos</h2>
                <p class="mt-1 text-[11px] leading-relaxed text-muted">
                  FFmpeg, ImageMagick, libvips y Pandoc, sin las limitaciones de un navegador.
                </p>
              </article>
              <article class="rounded-xl border border-line bg-panel p-4">
                <Files size={18} class="text-accent" strokeWidth={1.9} />
                <h2 class="mt-3 text-xs font-semibold">
                  {inputFormats > 0 ? `${inputFormats}+ formatos` : "Muchos formatos"}
                </h2>
                <p class="mt-1 text-[11px] leading-relaxed text-muted">
                  Imágenes, audio, vídeo y documentos en una sola cola.
                </p>
              </article>
            </section>

            <section class="rounded-xl border border-line bg-panel p-4">
              <h2 class="text-xs font-semibold uppercase tracking-wider text-muted">
                Antes de empezar
              </h2>
              <div class="mt-3 grid gap-2 text-[11px] text-muted sm:grid-cols-2">
                <p>· Arrastra archivos o una carpeta completa.</p>
                <p>· Elige el formato de salida por archivo.</p>
                <p>· Ajusta la calidad y la carpeta de destino.</p>
                <p>· Pulsa «Convertir todo» y revisa el progreso.</p>
              </div>
            </section>
          {:else}
            <section class="space-y-2">
              <div class="flex items-center justify-between gap-3">
                <h2 class="text-sm font-semibold">
                  Cola <span class="font-normal text-muted">({items.length})</span>
                </h2>
                <button
                  type="button"
                  onclick={clearFinished}
                  class="inline-flex items-center gap-1.5 text-[11px] text-muted transition-colors hover:text-ink"
                >
                  <Trash2 size={13} strokeWidth={2} />
                  Limpiar terminados
                </button>
              </div>

              <ul class="space-y-2">
                {#each items as item (item.id)}
                  <QueueItemCard
                    {item}
                    targets={targetsFor(item)}
                    onRemove={removeItem}
                    onConvert={convertOne}
                    onCancel={cancelItem}
                    onOpen={openOutput}
                    onTargetChange={onTargetChange}
                  />
                {/each}
              </ul>
            </section>
          {/if}
        </div>

        <SettingsPanel
          {engines}
          bind:outputDir
          bind:quality
          bind:preserveMetadata
          bind:overwrite
          bind:openAfter
          {pandocMissing}
          onPickOutput={pickOutputDir}
        />
      </div>
    {/if}
  </main>

  {#if items.length > 0}
    <footer class="fixed inset-x-0 bottom-0 z-20 border-t border-line bg-canvas/90 backdrop-blur">
      <div class="mx-auto flex max-w-7xl flex-wrap items-center gap-3 px-6 py-3">
        <p class="text-[11px] text-muted">
          {pendingCount} pendiente(s)
          {#if convertingCount > 0}
            · <span class="text-accent">{convertingCount} convirtiendo</span>
          {/if}
          · {doneCount} completado(s)
        </p>

        <div class="ml-auto flex items-center gap-2">
          {#if convertingCount > 0}
            <button
              type="button"
              onclick={cancelAll}
              class="inline-flex h-9 items-center gap-1.5 rounded-lg border border-danger/30 px-3 text-xs font-medium text-danger transition-colors hover:bg-danger/10"
            >
              <Square size={12} strokeWidth={2.2} />
              Cancelar todo
            </button>
          {/if}
          <button
            type="button"
            onclick={clearAll}
            class="inline-flex h-9 items-center gap-1.5 rounded-lg border border-line bg-panel px-3 text-xs font-medium text-muted transition-colors hover:text-ink"
          >
            <Trash2 size={13} strokeWidth={2} />
            Vaciar
          </button>
          <button
            type="button"
            onclick={startAll}
            disabled={!canStart || busy}
            class="inline-flex h-9 items-center gap-1.5 rounded-lg bg-accent px-4 text-xs font-semibold text-stone-950 transition-colors hover:bg-accent-strong disabled:cursor-not-allowed disabled:opacity-40"
          >
            {#if busy}
              <Loader2 size={13} class="animate-spin" strokeWidth={2.4} />
              Convirtiendo…
            {:else}
              <Play size={13} strokeWidth={2.4} />
              Convertir todo
            {/if}
          </button>
        </div>
      </div>
    </footer>
  {/if}

  {#if toast}
    <div
      class={`fixed bottom-20 left-1/2 z-40 max-w-[min(90vw,560px)] -translate-x-1/2 rounded-lg border bg-panel px-3.5 py-2 text-xs shadow-2xl shadow-black/30 ${
        toast.tone === "error"
          ? "border-danger/30 text-danger"
          : "border-line text-ink"
      }`}
      role="status"
      aria-live="polite"
    >
      {toast.message}
    </div>
  {/if}
</div>
