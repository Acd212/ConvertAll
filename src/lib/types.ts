export type Category = "image" | "audio" | "video" | "document" | "unknown";

export interface EngineInfo {
  id: string;
  name: string;
  available: boolean;
  version: string | null;
  path: string | null;
}

export interface FormatDef {
  id: string;
  label: string;
  extension: string;
  category: Category;
  canRead: boolean;
  canWrite: boolean;
  engine: string;
  videoHeights: number[];
  imageSides: number[];
  audioBitrates: number[];
}

export interface FileInfo {
  path: string;
  name: string;
  extension: string;
  size: number;
  category: Category;
  formatId: string;
  engine: string;
  engineAvailable: boolean;
  suggestedTarget: string;
}

export interface ConvertRequest {
  jobId: string;
  inputPath: string;
  outputDir: string;
  targetFormat: string;
  quality: number;
  preserveMetadata: boolean;
  overwrite: boolean;
  videoHeight: number | null;
  imageMaxSide: number | null;
  audioBitrate: number | null;
}

export interface ConvertResult {
  outputPath: string;
  outputSize: number;
  durationMs: number;
}

export interface ProgressPayload {
  jobId: string;
  progress: number;
  stage: string;
}
