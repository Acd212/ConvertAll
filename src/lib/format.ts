import type { JobStatus } from "./queue";
import type { Category } from "./types";

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** index;
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

export function categoryLabel(category: Category): string {
  switch (category) {
    case "image":
      return "Imagen";
    case "audio":
      return "Audio";
    case "video":
      return "Vídeo";
    case "document":
      return "Documento";
    default:
      return "Desconocido";
  }
}

export function statusLabel(status: JobStatus): string {
  switch (status) {
    case "pending":
      return "Pendiente";
    case "converting":
      return "Convirtiendo";
    case "done":
      return "Completado";
    case "error":
      return "Error";
    case "cancelled":
      return "Cancelado";
  }
}

export function formatDuration(milliseconds: number): string {
  if (milliseconds < 1000) return `${milliseconds} ms`;
  return `${(milliseconds / 1000).toFixed(1)} s`;
}
