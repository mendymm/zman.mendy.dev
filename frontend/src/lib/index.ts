// place files you want to import through the `$lib` alias in this folder.

export { getCurrentTime, printZman } from "./wasm-gen-output/wasm_funcs";
import init from "./wasm-gen-output/wasm_funcs";

let initPromise: Promise<void> | null = null;

/** Call once on mount to start loading the WASM binary. */
export function warmup(): Promise<void> {
  if (typeof window === "undefined") return Promise.resolve();
  initPromise ??= init().then(() => undefined);
  return initPromise;
}
