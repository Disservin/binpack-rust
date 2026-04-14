import init, { parse_binpack } from "./pkg/sfbinpack.js";

const fileInput = document.getElementById("file-input");
const previewLimitInput = document.getElementById("preview-limit");
const parseButton = document.getElementById("parse-button");
const statusEl = document.getElementById("status");
const summaryEl = document.getElementById("summary");
const previewEl = document.getElementById("preview");

function setStatus(message) {
  statusEl.textContent = message;
}

function renderPreview(entries) {
  if (!entries.length) {
    previewEl.textContent = "No preview entries returned.";
    return;
  }

  const table = document.createElement("table");
  const thead = document.createElement("thead");
  const tbody = document.createElement("tbody");

  thead.innerHTML = `
    <tr>
      <th>#</th>
      <th>FEN</th>
      <th>UCI</th>
      <th>Score</th>
      <th>Ply</th>
      <th>Result</th>
      <th>Continuation</th>
    </tr>
  `;

  entries.forEach((entry, index) => {
    const row = document.createElement("tr");
    row.innerHTML = `
      <td>${index + 1}</td>
      <td>${entry.fen}</td>
      <td>${entry.uci}</td>
      <td>${entry.score}</td>
      <td>${entry.ply}</td>
      <td>${entry.result}</td>
      <td>${entry.continuation}</td>
    `;
    tbody.appendChild(row);
  });

  table.appendChild(thead);
  table.appendChild(tbody);
  previewEl.replaceChildren(table);
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
    const previewLimit = Number.parseInt(previewLimitInput.value, 10) || 10;
    const result = parse_binpack(bytes, previewLimit);

    summaryEl.textContent = JSON.stringify(
      {
        byteLength: result.byteLength,
        totalEntries: result.totalEntries,
        previewCount: result.previewCount,
      },
      null,
      2,
    );

    renderPreview(result.preview);
    setStatus(`Parsed ${file.name} successfully.`);
  } catch (error) {
    summaryEl.textContent = "No data available.";
    previewEl.textContent = "No preview available.";
    setStatus(`Parse failed: ${error}`);
  } finally {
    parseButton.disabled = false;
  }
}

async function main() {
  await init();
  setStatus("Wasm module loaded. Select a .binpack file to inspect it.");
  parseButton.addEventListener("click", parseSelectedFile);
}

main().catch((error) => {
  setStatus(`Failed to initialize wasm module: ${error}`);
  parseButton.disabled = true;
});
