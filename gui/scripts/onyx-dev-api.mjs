import { execFile } from "node:child_process";
import { createServer } from "node:http";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const port = 5174;

function cleanOnyxOutput(output) {
  return output
    .replace(/^Onyx Brain .+ chat\r?\nMode: .+\r?\n/, "")
    .trim();
}

function sendJson(res, status, body) {
  res.writeHead(status, {
    "Access-Control-Allow-Origin": "http://127.0.0.1:5173",
    "Access-Control-Allow-Headers": "Content-Type",
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Content-Type": "application/json",
  });
  res.end(JSON.stringify(body));
}

createServer((req, res) => {
  if (req.method === "OPTIONS") {
    sendJson(res, 204, {});
    return;
  }

  if (req.url !== "/api/chat" || req.method !== "POST") {
    sendJson(res, 404, { error: "Not found" });
    return;
  }

  let body = "";
  req.on("data", (chunk) => {
    body += chunk;
  });
  req.on("end", () => {
    let input = "";
    try {
      input = String(JSON.parse(body || "{}").input || "").trim();
    } catch {
      sendJson(res, 400, { error: "Invalid JSON body" });
      return;
    }

    if (!input) {
      sendJson(res, 400, { error: "Message is required" });
      return;
    }

    execFile(
      "cargo",
      ["run", "--quiet", "--", "chat", input],
      { cwd: projectRoot, windowsHide: true, timeout: 120000 },
      (error, stdout, stderr) => {
        if (error) {
          sendJson(res, 500, { error: stderr || error.message });
          return;
        }
        sendJson(res, 200, { response: cleanOnyxOutput(stdout) });
      },
    );
  });
}).listen(port, "127.0.0.1", () => {
  console.log(`Onyx dev API listening on http://127.0.0.1:${port}`);
});
