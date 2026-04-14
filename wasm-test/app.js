import init, { parse_binpack_chunk } from "./pkg/sfbinpack.js";

const fileInput = document.getElementById("file-input");
const fileNameDisplay = document.getElementById("file-name-display");
const previewLimitInput = document.getElementById("preview-limit");
const parseButton = document.getElementById("parse-button");
const exampleButton = document.getElementById("example-button");
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

function parseChunkSize(header) {
  if (header.length !== 8) {
    throw new Error("Short BINP header");
  }

  if (
    header[0] !== 0x42 ||
    header[1] !== 0x49 ||
    header[2] !== 0x4e ||
    header[3] !== 0x50
  ) {
    throw new Error("Invalid BINP magic");
  }

  return (
    header[4] |
    (header[5] << 8) |
    (header[6] << 16) |
    (header[7] << 24)
  ) >>> 0;
}

async function readSlice(file, start, end) {
  return new Uint8Array(await file.slice(start, end).arrayBuffer());
}

async function parseBinpackSource({ name, size, readRange }) {
  parseButton.disabled = true;
  exampleButton.disabled = true;
  setStatus(`Reading ${name}...`);

  try {
    const previewLimit = parseInt(previewLimitInput.value, 10) || 10;
    let offset = 0;
    let chunkIndex = 0;
    let entriesRead = 0;
    let bytesRead = 0;
    const preview = [];

    while (offset < size && preview.length < previewLimit) {
      setStatus(`Parsing ${name} chunk ${chunkIndex + 1}...`);

      const header = await readRange(offset, offset + 8);
      const chunkSize = parseChunkSize(header);
      const payloadStart = offset + 8;
      const payloadEnd = payloadStart + chunkSize;

      if (payloadEnd > size) {
        throw new Error(`Chunk ${chunkIndex + 1} exceeds file size`);
      }

      const payload = await readRange(payloadStart, payloadEnd);
      const remainingPreview = Math.max(previewLimit - preview.length, 0);
      const result = parse_binpack_chunk(payload, remainingPreview);

      entriesRead += result.entriesRead;
      bytesRead += payloadEnd - offset;
      preview.push(...result.preview);

      offset = payloadEnd;
      chunkIndex += 1;
    }

    document.getElementById("m-bytes").textContent =
      bytesRead.toLocaleString();
    document.getElementById("m-total").textContent =
      entriesRead.toLocaleString();
    document.getElementById("m-preview").textContent =
      preview.length.toLocaleString();
    metricsRow.style.display = "";

    renderPreview(preview);
    const stoppedEarly = preview.length >= previewLimit && offset < size;
    const message = stoppedEarly
      ? `Loaded ${preview.length} preview rows from ${chunkIndex} chunk${chunkIndex === 1 ? "" : "s"} without scanning the rest of the file.`
      : `Parsed ${name} successfully across ${chunkIndex} chunk${chunkIndex === 1 ? "" : "s"}.`;
    setStatus(message, "ok");
  } catch (err) {
    metricsRow.style.display = "none";
    previewTable.style.display = "none";
    previewEmpty.style.display = "";
    previewEmpty.textContent = "No preview available.";
    setStatus(`Parse failed: ${err}`, "err");
  } finally {
    parseButton.disabled = false;
    exampleButton.disabled = false;
  }
}

async function parseSelectedFile() {
  const file = fileInput.files?.[0];
  if (!file) {
    setStatus("Select a .binpack file first.");
    return;
  }

  await parseBinpackSource({
    name: file.name,
    size: file.size,
    readRange: (start, end) => readSlice(file, start, end),
  });
}

async function parseExampleFile() {
  const response = await fetch("./examples/ep1.binpack");
  if (!response.ok) {
    throw new Error(`Failed to fetch example binpack: ${response.status}`);
  }

  const blob = await response.blob();
  fileNameDisplay.textContent = "example: ep1.binpack";

  await parseBinpackSource({
    name: "example ep1.binpack",
    size: blob.size,
    readRange: async (start, end) =>
      new Uint8Array(await blob.slice(start, end).arrayBuffer()),
  });
}

async function handleExampleClick() {
  try {
    await parseExampleFile();
  } catch (err) {
    parseButton.disabled = false;
    exampleButton.disabled = false;
    setStatus(`Example load failed: ${err}`, "err");
  }
}

async function main() {
  await init();
  setStatus("Wasm loaded. The page stops reading once it has enough preview rows.");
  parseButton.addEventListener("click", parseSelectedFile);
  exampleButton.addEventListener("click", handleExampleClick);
}

main().catch((err) => {
  setStatus(`Failed to initialize wasm: ${err}`, "err");
  parseButton.disabled = true;
  exampleButton.disabled = true;
});
