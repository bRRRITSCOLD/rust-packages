// core-react's column of `cargo xtask captures compare`: one 2x PNG per job from the published
// Storybook, cropped to the story's root element plus 4px, like the GPUI captures.
//
// usage: node storybook.cjs <storybook-url> <jobs.json> <out-dir>
// jobs.json: [{ "story": "components-button--default", "args": { "variant": "outline" },
//               "theme": "light" | "dark", "file": "<png name>" }]
//
// URL `args` are ignored by the published Storybook, so args and the theme go through the
// preview channel. Transitions are disabled and focus blurred, so a story's `play` click leaves
// no focus ring or half-finished hover in the shot.
const fs = require('node:fs')
const path = require('node:path')
const { chromium } = require('playwright')

const [, , base, jobsFile, out] = process.argv
const jobs = JSON.parse(fs.readFileSync(jobsFile, 'utf8'))
const MARGIN = 4

async function main() {
	const browser = await chromium.launch()
	const page = await browser.newPage({ deviceScaleFactor: 2, viewport: { width: 640, height: 320 } })
	for (const job of jobs) {
		await page.goto(`${base}/iframe.html?id=${job.story}&viewMode=story`)
		await page.waitForFunction(() => window.__STORYBOOK_ADDONS_CHANNEL__ && document.querySelector('#storybook-root > *'))
		await page.addStyleTag({ content: '*,*::before,*::after{transition:none!important;animation:none!important}' })
		await page.evaluate(async ({ story, args, theme }) => {
			const channel = window.__STORYBOOK_ADDONS_CHANNEL__
			const rendered = () =>
				Promise.race([new Promise((done) => channel.once('storyRendered', done)), new Promise((done) => setTimeout(done, 1500))])
			let next = rendered()
			channel.emit('updateGlobals', { globals: { theme } })
			await next
			next = rendered()
			channel.emit('updateStoryArgs', { storyId: story, updatedArgs: args })
			await next
		}, job)
		const dark = await page.evaluate(() => document.documentElement.classList.contains('dark'))
		if (dark !== (job.theme === 'dark')) throw new Error(`theme ${job.theme} not applied to ${job.story}`)
		await page.evaluate(() => document.activeElement?.blur())
		await page.mouse.move(0, 0)
		const box = await page.locator('#storybook-root > *').first().boundingBox()
		await page.screenshot({
			path: path.join(out, job.file),
			clip: { x: box.x - MARGIN, y: box.y - MARGIN, width: box.width + 2 * MARGIN, height: box.height + 2 * MARGIN },
		})
		process.stdout.write('.')
	}
	process.stdout.write('\n')
	await browser.close()
}

main().catch((err) => {
	console.error(err.message)
	process.exit(1)
})
