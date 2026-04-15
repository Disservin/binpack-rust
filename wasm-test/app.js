import init, { parse_binpack_chunk } from "./pkg/sfbinpack.js";

const fileInput = document.getElementById("file-input");
const fileNameDisplay = document.getElementById("file-name-display");
const parseButton = document.getElementById("parse-button");
const exampleButton = document.getElementById("example-button");
const openFileButton = document.getElementById("open-file-button");
const statusEl = document.getElementById("status");
const metricsRow = document.getElementById("metrics-row");
const previewTable = document.getElementById("preview-table");
const previewEmpty = document.getElementById("preview-empty");
const previewTbody = document.getElementById("preview-tbody");

let currentFileHandle = null;
let currentFile = null;
let currentFileSize = 0;
let currentChunkOffset = 0;
let currentChunkIndex = 0;
let visitedOffsets = [];
let currentChunkPayload = null;
let currentChunkNextOffset = 0;
let currentEntryOffset = 0;
let totalEntriesInChunk = 0;
let hasMoreEntriesInChunk = true;
const ENTRIES_PER_PAGE = 1000;

fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];
  if (file) {
    fileNameDisplay.textContent = file.name;
    currentFileHandle = null;
    currentFile = file;
    currentFileSize = file.size;
  }
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
      <td>${entry.offset !== undefined ? entry.offset : i + 1}</td>
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

async function readSliceFromHandle(fileHandle, start, end) {
  const file = await fileHandle.getFile();
  const blob = file.slice(start, end);
  const arrayBuffer = await blob.arrayBuffer();
  return new Uint8Array(arrayBuffer);
}

async function readChunkAt(offset, readRange) {
  const header = await readRange(offset, offset + 8);
  const chunkSize = parseChunkSize(header);
  const payload = await readRange(offset + 8, offset + 8 + chunkSize);
  return { payload, nextOffset: offset + 8 + chunkSize };
}

function parseEntriesFromPayload(payload, startEntryIndex, count) {
  const chunkResult = parse_binpack_chunk(payload, count, startEntryIndex);
  const totalEntries = chunkResult.totalEntries;
  const entriesRead = chunkResult.entriesRead;
  
  return { 
    entries: chunkResult.preview, 
    totalParsed: totalEntries,
    hasMore: entriesRead === count && startEntryIndex + entriesRead < totalEntries
  };
}

async function navigateForward(readRange) {
  parseButton.disabled = true;
  exampleButton.disabled = true;

  try {
    if (!currentChunkPayload) {
      setStatus("No chunk loaded", "err");
      parseButton.disabled = false;
      exampleButton.disabled = false;
      return;
    }

    const nextEntryOffset = currentEntryOffset + ENTRIES_PER_PAGE;
    const result = parseEntriesFromPayload(currentChunkPayload, nextEntryOffset, ENTRIES_PER_PAGE);
    
    if (result.entries.length > 0 || result.hasMore) {
      currentEntryOffset = nextEntryOffset;
      if (!result.hasMore) {
        totalEntriesInChunk = result.totalParsed;
      }
      
      document.getElementById("m-bytes").textContent = result.entries.length.toLocaleString();
      document.getElementById("m-total").textContent = totalEntriesInChunk.toLocaleString();
      document.getElementById("m-preview").textContent = result.entries.length.toLocaleString();
      metricsRow.style.display = "";
      
      renderPreview(result.entries);
      setStatus(`Chunk ${currentChunkIndex + 1}, entries ${currentEntryOffset + 1}-${currentEntryOffset + result.entries.length} of ${totalEntriesInChunk}`, "ok");
    } else {
      if (currentChunkOffset >= currentFileSize) {
        setStatus("Already at last chunk", "err");
      } else {
        visitedOffsets.push(currentChunkOffset);
        const chunkResult = await readChunkAt(currentChunkOffset, readRange);
        currentChunkPayload = chunkResult.payload;
        currentChunkNextOffset = chunkResult.nextOffset;
        
        const parseResult = parseEntriesFromPayload(chunkResult.payload, 0, ENTRIES_PER_PAGE);
        totalEntriesInChunk = parseResult.totalParsed;
        hasMoreEntriesInChunk = parseResult.hasMore;
        currentEntryOffset = 0;
        currentChunkOffset = currentChunkNextOffset;
        currentChunkIndex++;
        
        document.getElementById("m-bytes").textContent = chunkResult.payload.length.toLocaleString();
        document.getElementById("m-total").textContent = totalEntriesInChunk.toLocaleString();
        document.getElementById("m-preview").textContent = parseResult.entries.length.toLocaleString();
        metricsRow.style.display = "";
        
        renderPreview(parseResult.entries);
        setStatus(`Chunk ${currentChunkIndex + 1}, entries 1-${parseResult.entries.length} of ${totalEntriesInChunk}`, "ok");
      }
    }
  } catch (err) {
    setStatus(`Navigation failed: ${err}`, "err");
  } finally {
    parseButton.disabled = false;
    exampleButton.disabled = false;
  }
}

async function navigateBackward(readRange) {
  parseButton.disabled = true;
  exampleButton.disabled = true;

  try {
    const prevEntryOffset = currentEntryOffset - ENTRIES_PER_PAGE;
    
    if (currentChunkPayload && prevEntryOffset >= 0) {
      currentEntryOffset = prevEntryOffset;
      const result = parseEntriesFromPayload(currentChunkPayload, currentEntryOffset, ENTRIES_PER_PAGE);
      
      document.getElementById("m-bytes").textContent = result.entries.length.toLocaleString();
      document.getElementById("m-total").textContent = totalEntriesInChunk.toLocaleString();
      document.getElementById("m-preview").textContent = result.entries.length.toLocaleString();
      metricsRow.style.display = "";
      
      renderPreview(result.entries);
      setStatus(`Chunk ${currentChunkIndex + 1}, entries ${currentEntryOffset + 1}-${currentEntryOffset + result.entries.length} of ${totalEntriesInChunk}`, "ok");
    } else if (currentChunkIndex > 0) {
      const prevOffset = visitedOffsets[currentChunkIndex - 1];
      if (prevOffset !== undefined) {
        const result = await readChunkAt(prevOffset, readRange);
        currentChunkPayload = result.payload;
        currentChunkNextOffset = result.nextOffset;
        
        const parseResult = parseEntriesFromPayload(result.payload, 0, ENTRIES_PER_PAGE);
        totalEntriesInChunk = parseResult.totalParsed;
        hasMoreEntriesInChunk = parseResult.hasMore;
        
        currentChunkOffset = prevOffset;
        currentChunkIndex--;
        visitedOffsets.pop();
        
        const lastPageStart = Math.max(0, totalEntriesInChunk - ENTRIES_PER_PAGE);
        currentEntryOffset = lastPageStart;
        
        const finalResult = parseEntriesFromPayload(result.payload, lastPageStart, ENTRIES_PER_PAGE);
        
        document.getElementById("m-bytes").textContent = result.payload.length.toLocaleString();
        document.getElementById("m-total").textContent = totalEntriesInChunk.toLocaleString();
        document.getElementById("m-preview").textContent = finalResult.entries.length.toLocaleString();
        metricsRow.style.display = "";
        
        renderPreview(finalResult.entries);
        setStatus(`Chunk ${currentChunkIndex + 1}, entries ${lastPageStart + 1}-${totalEntriesInChunk} of ${totalEntriesInChunk}`, "ok");
      } else {
        setStatus("Already at first chunk", "err");
      }
    } else {
      setStatus("Already at first chunk", "err");
    }
  } catch (err) {
    setStatus(`Navigation failed: ${err}`, "err");
  } finally {
    parseButton.disabled = false;
    exampleButton.disabled = false;
  }
}

async function handleKeyDown(event) {
  if (!currentFile) {
    return;
  }

  const readRange = currentFileHandle
    ? (start, end) => readSliceFromHandle(currentFileHandle, start, end)
    : (start, end) => readSlice(currentFile, start, end);

  if (event.key === "ArrowRight" || event.key === "ArrowDown") {
    event.preventDefault();
    await navigateForward(readRange);
  } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
    event.preventDefault();
    await navigateBackward(readRange);
  }
}

async function parseBinpackSource({ name, size, readRange }) {
  parseButton.disabled = true;
  exampleButton.disabled = true;
  setStatus(`Reading ${name}...`);

  try {
    currentFileSize = size;
    currentChunkOffset = 0;
    currentChunkIndex = 0;
    visitedOffsets = [];
    currentChunkPayload = null;
    currentEntryOffset = 0;
    totalEntriesInChunk = 0;
    hasMoreEntriesInChunk = true;

    const result = await readChunkAt(0, readRange);
    currentChunkPayload = result.payload;
    currentChunkNextOffset = result.nextOffset;
    
    const parseResult = parseEntriesFromPayload(result.payload, 0, ENTRIES_PER_PAGE);
    totalEntriesInChunk = parseResult.totalParsed;
    hasMoreEntriesInChunk = parseResult.hasMore;
    currentEntryOffset = 0;
    currentChunkOffset = currentChunkNextOffset;

    document.getElementById("m-bytes").textContent = result.payload.length.toLocaleString();
    document.getElementById("m-total").textContent = totalEntriesInChunk.toLocaleString();
    document.getElementById("m-preview").textContent = parseResult.entries.length.toLocaleString();
    metricsRow.style.display = "";

    renderPreview(parseResult.entries);
    document.getElementById("navigation-hint").style.display = "block";
    setStatus(`Chunk 1, entries 1-${parseResult.entries.length} of ${totalEntriesInChunk}. Use arrow keys to navigate.`, "ok");
  } catch (err) {
    metricsRow.style.display = "none";
    previewTable.style.display = "none";
    previewEmpty.style.display = "";
    previewEmpty.textContent = "No preview available.";
    document.getElementById("navigation-hint").style.display = "none";
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

async function openWithFilePicker() {
  if (!("showOpenFilePicker" in window)) {
    setStatus("File System Access API not supported in this browser", "err");
    return;
  }

  try {
    const [fileHandle] = await window.showOpenFilePicker({
      types: [
        {
          description: "BINP Pack Files",
          accept: {
            "application/octet-stream": [".binpack", ".bin"],
          },
        },
      ],
      multiple: false,
    });

    const file = await fileHandle.getFile();
    fileNameDisplay.textContent = file.name;
    currentFileHandle = fileHandle;
    currentFile = file;

    await parseBinpackSource({
      name: file.name,
      size: file.size,
      readRange: (start, end) => readSliceFromHandle(fileHandle, start, end),
    });
  } catch (err) {
    if (err.name !== "AbortError") {
      setStatus(`File picker error: ${err}`, "err");
    }
  }
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
  setStatus("Wasm loaded. Use arrow keys to navigate chunks.");
  parseButton.addEventListener("click", parseSelectedFile);
  exampleButton.addEventListener("click", handleExampleClick);
  if (openFileButton) {
    openFileButton.addEventListener("click", openWithFilePicker);
  }
  if (!("showOpenFilePicker" in window)) {
    if (openFileButton) {
      openFileButton.style.display = "none";
    }
  }
  document.addEventListener("keydown", handleKeyDown);
}

main().catch((err) => {
  setStatus(`Failed to initialize wasm: ${err}`, "err");
  parseButton.disabled = true;
  exampleButton.disabled = true;
});
