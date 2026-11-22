import type { EngineInfo } from "./types";

/**
 * Motores que pueden sustituirse entre sí para la misma categoría.
 * El backend usa ImageMagick para imágenes y recurre a libvips si falta.
 */
const REPLACEMENTS: Record<string, string> = {
  magick: "vips",
  vips: "magick",
  libreoffice: "msoffice",
  msoffice: "libreoffice",
};

/**
 * Oculta un motor ausente cuando su reemplazo disponible está en uso,
 * para que la interfaz muestre el motor que realmente convierte.
 */
export function visibleEngines(engines: EngineInfo[]): EngineInfo[] {
  const isAvailable = (id: string) =>
    engines.some((engine) => engine.id === id && engine.available);

  return engines.filter((engine) => {
    if (engine.available) return true;
    const replacement = REPLACEMENTS[engine.id];
    return !replacement || !isAvailable(replacement);
  });
}
