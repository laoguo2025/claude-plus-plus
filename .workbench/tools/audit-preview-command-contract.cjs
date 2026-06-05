const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..", "..");
const previewPath = path.join(root, "src", "previewCommands.ts");
const text = fs.readFileSync(previewPath, "utf8");
const errors = [];

if (/return\s+undefined\s+as\s+T\s*;/.test(text)) {
  errors.push("previewCommand must not return `undefined as T` for noop commands.");
}

const stateFunctionStart = text.indexOf("function applyPreviewCommandState(");
if (stateFunctionStart === -1) {
  errors.push("applyPreviewCommandState function not found.");
} else {
  const body = text.slice(stateFunctionStart, text.indexOf("\n}\n\nfunction previewLogs", stateFunctionStart));
  if (!/\bswitch\s*\(\s*cmd\s*\)/.test(body)) {
    errors.push("applyPreviewCommandState should use switch or mutually exclusive branches.");
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}
