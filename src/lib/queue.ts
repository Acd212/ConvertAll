import type { FileInfo } from "./types";

export type JobStatus = "pending" | "converting" | "done" | "error" | "cancelled";

export interface QueueItem extends FileInfo {
  id: string;
  target: string;
  status: JobStatus;
  progress: number;
  outputPath?: string;
  error?: string;
  videoHeight?: number;
  imageMaxSide?: number;
  audioBitrate?: number;
}

export type QueueOptionPatch = Pick<
  QueueItem,
  "videoHeight" | "imageMaxSide" | "audioBitrate"
>;
