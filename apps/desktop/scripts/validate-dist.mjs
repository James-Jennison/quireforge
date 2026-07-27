import { readFileSync, readdirSync, statSync } from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath } from "node:url";

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

const limits = {
  entry: 256 * 1024,
  app: 300 * 1024,
  // M27's native Chat/Codex capability contracts, the eight closed M26
  // palettes, M28's explicitly confirmed safe snapshot boundary, and M29's
  // bounded managed Advisor conversation, its digest-bound approval controller,
  // one-time B2 handoff, bounded B3 status return, the managed-initializer
  // compatibility hotfix, M33's bounded PDF projection contract, and the
  // beta.27 chat-first Advisor layout are included in the shipped desktop
  // bundle. Keep a
  // narrow, measured ceiling instead of weakening the check or excluding chunks.
  javascript: 896 * 1024,
  stylesheets: 105 * 1024,
};

const failures = [];
if (entryBytes > limits.entry) {
  failures.push(
    `initial JavaScript is ${entryBytes} bytes; limit is ${limits.entry}`,
  );
}
if (appBytes === null) {
  failures.push(
    "application shell must remain outside the startup entry for native WebKit rendering",
  );
} else if (appBytes > limits.app) {
  failures.push(
    `application shell is ${appBytes} bytes; limit is ${limits.app}`,
  );
}
if (javascriptBytes > limits.javascript) {
  failures.push(
    `total JavaScript is ${javascriptBytes} bytes; limit is ${limits.javascript}`,
  );
}
if (stylesheetBytes > limits.stylesheets) {
  failures.push(
    `total CSS is ${stylesheetBytes} bytes; limit is ${limits.stylesheets}`,
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
