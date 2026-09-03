import fs from "node:fs";
import path from "node:path";

const zhDictPath = path.resolve(__dirname, "./locales/zh-CN.json");
const missingPath = path.resolve(__dirname, "./diff/missing.json");

function merge() {
  if (!fs.existsSync(missingPath)) {
    console.log("No diff/missing.json found.");
    return;
  }
  const existing = JSON.parse(fs.readFileSync(zhDictPath, "utf-8"));
  const missing = JSON.parse(fs.readFileSync(missingPath, "utf-8"));

  let mergedCount = 0;
  for (const [k, v] of Object.entries(missing)) {
    if (typeof v === "string" && v.trim()) {
      existing[k] = v.trim();
      mergedCount++;
    }
  }

  fs.writeFileSync(zhDictPath, JSON.stringify(existing, null, 2), "utf-8");
  console.log(`✅ Successfully merged ${mergedCount} new translations into locales/zh-CN.json`);
}

merge();
