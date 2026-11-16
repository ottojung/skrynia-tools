import { spawn } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";
import { firefox, webkit } from "playwright";

const engines = [["Firefox", firefox], ["WebKit", webkit]];
for (const [index, [name, engine]] of engines.entries()) {
  const port = 18810 + index;
  const origin = `http://127.0.0.1:${port}`;
  const server = spawn("python3", ["tests/browser/server.py", "--port", String(port)], {
    cwd: new URL("../../", import.meta.url),
    stdio: ["ignore", "pipe", "pipe"],
  });
  let browser;
  const diagnostics = [];
  server.stdout.on("data", (chunk) => diagnostics.push(chunk.toString()));
  server.stderr.on("data", (chunk) => diagnostics.push(chunk.toString()));
  try {
    let ready = false;
    for (let i = 0; i < 80; i += 1) {
      if (server.exitCode !== null) throw new Error("test HTTP server exited");
      try {
        const response = await fetch(`${origin}/health`, { signal: AbortSignal.timeout(500) });
        if (response.ok) { ready = true; break; }
      } catch { /* wait until the local fixture is ready */ }
      await delay(100);
    }
    if (!ready) throw new Error("local browser fixture did not become ready");
    browser = await engine.launch({ headless: true });
    const page = await browser.newPage();
    page.on("pageerror", (err) => diagnostics.push(`Page: ${err.message}`));
    page.on("console", (line) => {
      if (line.type() === "error") diagnostics.push(`Console: ${line.text()}`);
    });
    await page.goto(`${origin}/tests/browser/index.html`, { waitUntil: "load" });
    await page.waitForFunction(() =>
      document.body.dataset.result === "pass" || document.body.dataset.result === "fail",
      null, { timeout: 45000 });
    const state = await page.locator("body").getAttribute("data-result");
    if (state !== "pass") {
      throw new Error(`browser protocol failed: ${await page.locator("body").innerText()}`);
    }
    console.log(`${name} Wasm HTTP, CAS and concurrent log integration passed`);
  } catch (error) {
    console.error(`${name}: ${error.stack || error}`);
    console.error(diagnostics.join("").slice(-8000));
    process.exitCode = 1;
  } finally {
    if (browser) await browser.close();
    server.kill("SIGTERM");
    await Promise.race([
      new Promise((resolve) => server.once("exit", resolve)),
      delay(3000),
    ]);
    if (server.exitCode === null && server.signalCode === null) server.kill("SIGKILL");
  }
}
