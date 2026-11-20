import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Category,
  ConvertRequest,
  ConvertResult,
  EngineInfo,
  FileInfo,
  FormatDef,
  ProgressPayload,
} from "./types";

export const api = {
  detectEngines: () => invoke<EngineInfo[]>("detect_engines"),
  getCatalog: () => invoke<FormatDef[]>("get_catalog"),
  getTargets: (category: Category) =>
    invoke<FormatDef[]>("get_targets", { category }),
  inspectFile: (path: string) => invoke<FileInfo>("inspect_file", { path }),
  convertFile: (request: ConvertRequest) =>
    invoke<ConvertResult>("convert_file", { request }),
  cancelJob: (jobId: string) => invoke<void>("cancel_job", { jobId }),
  defaultOutputDir: () => invoke<string>("default_output_dir"),
  scanFolder: (path: string) => invoke<string[]>("scan_folder", { path }),
  openInFileManager: (path: string) =>
    invoke<void>("open_in_file_manager", { path }),
  onProgress: (handler: (payload: ProgressPayload) => void): Promise<UnlistenFn> =>
    listen<ProgressPayload>("job-progress", (event) => handler(event.payload)),
};
