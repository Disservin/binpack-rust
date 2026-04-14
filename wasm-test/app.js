import init, { parse_binpack } from "./pkg/sfbinpack.js";

const fileInput = document.getElementById("file-input");
const fileNameDisplay = document.getElementById("file-name-display");
const previewLimitInput = document.getElementById("preview-limit");
const parseButton = document.getElementById("parse-button");
const statusEl = document.getElementById("status");
const metricsRow = document.getElementById("metrics-row");
const previewTable = document.getElementById("preview-table");
const previewEmpty = document.getElementById("preview-empty");
const previewTbody = document.getElementById("preview-tbody");

fileInput.addEventListener("change", () => {
  fileNameDisplay.textContent =
    fileInput.files?.[0]?.name ?? "no file selected";
});

function setStatus(message, type = "") {
  statusEl.textContent = message;
  statusEl.className = "status-bar" + (type ? ` ${type}` : "");
}

function renderPreview(entries) {
  previewTbody.innerHTML = "";
  if (!entries.length) {
    previewTable.style.display = "none";
    previewEmpty.textContent = "No entries returned.";
    previewEmpty.style.display = "";
    return;
  }
  entries.forEach((entry, i) => {
    const row = document.createElement("tr");
    row.innerHTML = `
      <td>${i + 1}</td>
      <td class="fen-cell" title="${entry.fen}">${entry.fen}</td>
      <td>${entry.uci}</td>
      <td class="score-cell">${entry.score}</td>
      <td>${entry.ply}</td>
      <td class="result-cell">${entry.result}</td>
      <td>${entry.continuation}</td>
    `;
    previewTbody.appendChild(row);
  });
  previewEmpty.style.display = "none";
  previewTable.style.display = "";
}

async function parseSelectedFile() {
  const file = fileInput.files?.[0];
  if (!file) {
    setStatus("Select a .binpack file first.");
    return;
  }

  parseButton.disabled = true;
  setStatus(`Reading ${file.name}...`);

  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const previewLimit = parseInt(previewLimitInput.value, 10) || 10;
    const result = parse_binpack(bytes, previewLimit);

    document.getElementById("m-bytes").textContent =
      result.byteLength.toLocaleString();
    document.getElementById("m-total").textContent =
      result.totalEntries.toLocaleString();
    document.getElementById("m-preview").textContent =
      result.previewCount.toLocaleString();
    metricsRow.style.display = "";

    renderPreview(result.preview);
    setStatus(`Parsed ${file.name} successfully.`, "ok");
  } catch (err) {
    metricsRow.style.display = "none";
    previewTable.style.display = "none";
    previewEmpty.style.display = "";
    previewEmpty.textContent = "No preview available.";
    setStatus(`Parse failed: ${err}`, "err");
  } finally {
    parseButton.disabled = false;
  }
}

async function main() {
  await init();
  setStatus("Wasm loaded — select a .binpack file to inspect.");
  parseButton.addEventListener("click", parseSelectedFile);
}

main().catch((err) => {
  setStatus(`Failed to initialize wasm: ${err}`, "err");
  parseButton.disabled = true;
});
