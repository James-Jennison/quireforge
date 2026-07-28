import { readFileSync, readdirSync, statSync } from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath } from "node:url";
import limits from "./bundle-budget.json" with { type: "json" };

const appRoot = fileURLToPath(new URL("..", import.meta.url));
const distRoot = join(appRoot, "dist");
const assetsRoot = join(distRoot, "assets");
const indexHtml = readFileSync(join(distRoot, "index.html"), "utf8");
const entryMatch = indexHtml.match(
  /<script[^>]+type="module"[^>]+src="\/assets\/([^"]+\.js)"/u,
);

if (!entryMatch) {
  throw new Error("desktop dist is missing its local module entry");
}

const entryName = basename(entryMatch[1]);
const assetNames = readdirSync(assetsRoot);
const javascript = assetNames.filter((name) => name.endsWith(".js"));
const stylesheets = assetNames.filter((name) => name.endsWith(".css"));
const bytes = (name) => statSync(join(assetsRoot, name)).size;
const entryBytes = bytes(entryName);
const appName = javascript.find((name) => name.startsWith("App-"));
const appBytes = appName ? bytes(appName) : null;
const javascriptBytes = javascript.reduce(
  (total, name) => total + bytes(name),
  0,
);
const stylesheetBytes = stylesheets.reduce(
  (total, name) => total + bytes(name),
  0,
);

const failures = [];
if (entryBytes > limits.entryBytes) {
  failures.push(
    `initial JavaScript is ${entryBytes} bytes; limit is ${limits.entryBytes}`,
  );
}
if (appBytes === null) {
  failures.push(
    "application shell must remain outside the startup entry for native WebKit rendering",
  );
} else if (appBytes > limits.appShellBytes) {
  failures.push(
    `application shell is ${appBytes} bytes; limit is ${limits.appShellBytes}`,
  );
}
if (javascriptBytes > limits.totalJavaScriptBytes) {
  failures.push(
    `total JavaScript is ${javascriptBytes} bytes; limit is ${limits.totalJavaScriptBytes}`,
  );
}
if (stylesheetBytes > limits.stylesheetsBytes) {
  failures.push(
    `total CSS is ${stylesheetBytes} bytes; limit is ${limits.stylesheetsBytes}`,
  );
}
if (!javascript.some((name) => name.startsWith("TerminalWorkspace-"))) {
  failures.push("terminal renderer must remain outside the initial bundle");
}
if (indexHtml.includes("http://") || indexHtml.includes("https://")) {
  failures.push("desktop dist must not load an external origin");
}

if (failures.length > 0) {
  throw new Error(
    `desktop dist validation failed:\n- ${failures.join("\n- ")}`,
  );
}

const kib = (value) => (value / 1024).toFixed(2);
console.log(
  `Desktop dist passed: entry ${kib(entryBytes)} KiB, app ${kib(
    appBytes,
  )} KiB, total JS ${kib(javascriptBytes)} KiB, total CSS ${kib(
    stylesheetBytes,
  )} KiB.`,
);
for (const [label, assets] of [
  ["JavaScript", javascript],
  ["CSS", stylesheets],
]) {
  const largest = assets
    .map((name) => [name, bytes(name)])
    .sort((left, right) => right[1] - left[1])
    .slice(0, 5)
    .map(([name, size]) => `${name} (${size} bytes)`)
    .join(", ");
  console.log(`Largest ${label} assets: ${largest}.`);
}
