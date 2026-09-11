// Real browser regression: jsdom/mocked Motion cannot detect layout scale distortion.
// Start Vite, then run with H_CAPSULE_TEST_URL=http://127.0.0.1:1432
// and H_PLAYWRIGHT_MODULE=/path/to/playwright (or install it in your test environment).
const assert = require('node:assert/strict')
const { chromium } = require(process.env.H_PLAYWRIGHT_MODULE || 'playwright')

;(async () => {
  const browser = await chromium.launch({ channel: 'chrome', headless: true })
  try {
    const cases = [
      { name: 'preview to transcribing', preview: true, menu: false, next: 'transcribing' },
      { name: 'preview to polishing', preview: true, menu: false, next: 'polishing' },
      { name: 'style menu to transcribing', preview: false, menu: true, next: 'transcribing' },
      { name: 'preview and menu to polishing', preview: true, menu: true, next: 'polishing' },
      { name: 'recording to outputting', preview: false, menu: false, next: 'outputting' },
      { name: 'interrupted processing', preview: true, menu: true, next: 'transcribing', rapid: true },
    ]
    for (const dpr of [1, 2]) {
      const page = await browser.newPage({ viewport: { width: 384, height: 188 }, deviceScaleFactor: dpr })
      await page.goto(`${process.env.H_CAPSULE_TEST_URL || 'http://127.0.0.1:1432'}/scripts/tests/browser/capsule-layout.html`)
      await page.waitForFunction(() => window.capsuleStore)
      for (const test of cases) {
        await page.evaluate(test => window.capsuleStore.setState({
          pipelineState: 'recording', partialTranscript: test.preview ? '역할 분담을 확인해 주세요.' : '',
          polishStyleMenuOpen: test.menu,
        }), test)
        await page.waitForTimeout(250)
        const frames = await page.evaluate(async test => {
          window.capsuleStore.setState({ pipelineState: test.next })
          const frames = []
          for (let i = 0; i < 24; i++) {
            await new Promise(requestAnimationFrame)
            const shell = document.querySelector('.jelly-capsule-active')
            const rect = shell.getBoundingClientRect()
            const text = shell.querySelector('p')
            frames.push({ x: rect.width / shell.offsetWidth, y: rect.height / shell.offsetHeight,
              textScale: text ? text.getBoundingClientRect().height / text.offsetHeight : 1 })
            if (test.rapid && i === 1) window.capsuleStore.setState({ pipelineState: 'polishing', partialTranscript: '' })
            if (test.rapid && i === 3) window.capsuleStore.setState({ pipelineState: 'outputting' })
          }
          return frames
        }, test)
        for (const frame of frames) {
          assert.ok(Math.abs(frame.x - 1) < 0.01 && Math.abs(frame.y - 1) < 0.01,
            `${test.name}, DPR ${dpr}: shell distorted ${JSON.stringify(frame)}`)
          assert.ok(!Number.isFinite(frame.textScale) || Math.abs(frame.textScale - 1) < 0.08,
            `${test.name}, DPR ${dpr}: status text distorted ${JSON.stringify(frame)}`)
        }
        console.log(`PASS DPR ${dpr}: ${test.name}, ${frames.length} rendered frames`)
      }
      await page.close()
    }
  } finally { await browser.close() }
})().catch(error => { console.error(error); process.exitCode = 1 })
