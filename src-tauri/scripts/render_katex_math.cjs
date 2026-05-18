const puppeteer = require("puppeteer");
async function main() {
  const tex = Buffer.from(process.argv[2] || "", "base64").toString("utf8");
  const displayMode = process.argv[3] === "1";

  const browser = await puppeteer.launch({ headless: "new" });
  try {
    const page = await browser.newPage();
    await page.setViewport({ width: 1600, height: 800, deviceScaleFactor: 2 });
    await page.setContent(
      '<!doctype html><html><head><meta charset="utf-8"></head><body><div id="math"></div></body></html>'
    );
    await page.addStyleTag({ path: require.resolve("katex/dist/katex.min.css") });
    await page.addScriptTag({ path: require.resolve("katex/dist/katex.min.js") });

    await page.evaluate(
      ({ tex, displayMode }) => {
        const el = document.getElementById("math");
        document.body.style.margin = "0";
        document.body.style.padding = "0";
        document.body.style.background = "#fff";
        el.style.display = "inline-block";
        el.style.margin = "0";
        el.style.padding = "0";
        el.style.lineHeight = "1";
        el.style.whiteSpace = "nowrap";
        window.katex.render(tex, el, {
          displayMode,
          throwOnError: false,
          strict: "ignore",
        });
        const display = el.querySelector(".katex-display");
        if (display) {
          display.style.margin = "0";
        }
        const marker = document.createElement("span");
        marker.id = "baseline-marker";
        marker.style.display = "inline-block";
        marker.style.width = "0";
        marker.style.height = "0";
        marker.style.padding = "0";
        marker.style.margin = "0";
        marker.style.verticalAlign = "baseline";
        el.appendChild(marker);
      },
      { tex, displayMode }
    );

    await page.evaluate(async () => {
      if (document.fonts && document.fonts.ready) {
        await document.fonts.ready;
      }
    });
    await page.evaluate(() => {
      document.documentElement.style.background = "transparent";
      document.body.style.background = "transparent";
    });
    await new Promise((resolve) => setTimeout(resolve, 20));

    const rect = await page.evaluate(() => {
      const el = document.getElementById("math");
      const r = el.getBoundingClientRect();
      const marker = document.getElementById("baseline-marker");
      const baseline = marker ? marker.getBoundingClientRect() : null;
      return {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
        baselineOffset: baseline ? baseline.y - r.y : 0,
      };
    });

    const clip = {
      x: Math.max(0, rect.x),
      y: Math.max(0, rect.y),
      width: Math.max(1, Math.ceil(rect.width)),
      height: Math.max(1, Math.ceil(rect.height)),
    };

    const png = await page.screenshot({
      clip,
      type: "png",
      omitBackground: true,
    });

    process.stdout.write(
        JSON.stringify({
          width: rect.width,
          height: rect.height,
          baselineOffset: rect.baselineOffset,
          devicePixelRatio: await page.evaluate(() => window.devicePixelRatio || 1),
          png: png.toString("base64"),
        })
    );
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : String(err));
  process.exit(1);
});
