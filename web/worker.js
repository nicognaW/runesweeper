// Generates fields off the page's thread: the page posts this very wasm module, then one job
// at a time ([mode, columns, rows, mines, seed, first click]); each answer is the field as one
// byte per cell, 1 for a mine. Anything else means it failed, and the page lays it itself.
import init, { lay_mines } from './app.js';

let ready;
onmessage = async ({ data }) => {
  if (data instanceof WebAssembly.Module) {
    ready = init({ module_or_path: data });
    return;
  }
  try {
    await ready;
    const [mode, w, h, mines, seed, first] = data;
    const field = lay_mines(mode, w, h, mines, seed, first);
    postMessage(field, [field.buffer]);
  } catch (error) {
    postMessage({ error: String(error) });
  }
};
