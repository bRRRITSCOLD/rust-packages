// core-react's column of `cargo xtask captures compare`: one 2x PNG per job from the published
// Storybook, cropped to the story's root element plus 4px, like the GPUI captures.
//
// usage: node storybook.cjs <storybook-url> <jobs.json> <out-dir>
// jobs.json: [{ "story": "components-button--default", "component": "Button",
//               "args": { "variant": "outline" }, "theme": "light" | "dark", "file": "<png name>" }]
//
// URL `args` are ignored by the published Storybook, so args and the theme go through the
// preview channel. Transitions are disabled and focus blurred, so a story's `play` click leaves
// no focus ring or half-finished hover in the shot.
//
// An overlay part renders outside the story root (a portal) or inside a full-screen layout div,
// so the shot crops to the component's own `data-slot` (core-react's kebab-case export name, or
// the one `SLOTS` names) when it is portalled or the root fills the viewport. A part that isn't showing is opened from
// its story's trigger first (hovered for a tooltip), and a submenu from its sub-trigger. Modal
// backdrops are hidden: GPUI captures the surface alone.
const fs = require('node:fs')
const path = require('node:path')
const { chromium } = require('playwright')

const [, , base, jobsFile, out] = process.argv
const jobs = JSON.parse(fs.readFileSync(jobsFile, 'utf8'))
const MARGIN = 4
const VIEWPORT = { width: 640, height: 320 }

/** Exports whose root element carries another component's `data-slot`. */
const SLOTS = {
	// ComboboxInput renders its InputGroup.
	ComboboxInput: 'input-group',
}

/** core-react's `data-slot` for an export: `AlertDialogContent` → `alert-dialog-content`. */
const slotOf = (component) => SLOTS[component] ?? component.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()

/** The first visible element with this `data-slot`, or null. */
const findSlot = (page, slot) =>
	page.evaluateHandle((slot) => {
		for (const el of document.querySelectorAll(`[data-slot="${slot}"]`)) {
			const box = el.getBoundingClientRect()
			if (box.width > 0 && box.height > 0 && getComputedStyle(el).visibility !== 'hidden') return el
		}
		return null
	}, slot)

/** Waits up to `ms` for the slot to show; resolves to it or null. */
async function waitForSlot(page, slot, ms) {
	const deadline = Date.now() + ms
	for (;;) {
		const handle = await findSlot(page, slot)
		const el = handle.asElement()
		if (el || Date.now() > deadline) return el
		await page.waitForTimeout(100)
	}
}

/** Opens the overlay that shows `slot`: its own trigger, the story's first trigger, or its first button. */
async function open(page, slot) {
	const own = slot.replace(/-content$/, '-trigger')
	const selectors = [`[data-slot="${own}"]`, '#storybook-root [data-slot$="-trigger"]', '#storybook-root button']
	for (const selector of selectors) {
		const trigger = page.locator(selector).first()
		if ((await trigger.count()) === 0) continue
		if (slot.startsWith('tooltip')) {
			// Off and back on, so a pointer already resting on the trigger enters it again.
			await page.mouse.move(0, 0)
			await trigger.hover()
		} else await trigger.click()
		break
	}
	let el = await waitForSlot(page, slot, 2000)
	if (!el) {
		const sub = page.locator('[data-slot$="-sub-trigger"]').first()
		if ((await sub.count()) > 0) {
			await sub.click()
			el = await waitForSlot(page, slot, 2000)
		}
	}
	return el
}

/** The element to crop to: the story root, unless the component's own slot is portalled or the root fills the viewport. */
async function target(page, component) {
	const root = page.locator('#storybook-root > *').first()
	const slot = slotOf(component)
	if ((await root.getAttribute('data-slot')) === slot) return root
	// Only an overlay's part is worth opening a trigger for; anything else is shot as before.
	const opens =
		slot.startsWith('toast') ||
		(await page.locator('#storybook-root [data-slot$="-trigger"], #storybook-root [role="combobox"]').count()) > 0
	if (!opens && !(await waitForSlot(page, slot, 0))) return root
	// The story's `play` may still be opening or closing it, so it must stay up for a moment.
	let el = null
	for (let attempt = 0; attempt < 4 && !el; attempt++) {
		el = (await waitForSlot(page, slot, 0)) ?? (await open(page, slot))
		if (!el) continue
		await page.waitForTimeout(400)
		el = await waitForSlot(page, slot, 0)
	}
	if (!el) throw new Error(`${component}: no [data-slot="${slot}"] showed, even from its trigger`)
	const portalled = await el.evaluate((el) => !document.querySelector('#storybook-root').contains(el))
	// The page under a modal surface would show in its margin; GPUI draws the surface alone.
	if (portalled && !slot.startsWith('tooltip')) await page.addStyleTag({ content: '#storybook-root{visibility:hidden}' })
	const rootBox = await root.boundingBox()
	// A locator, not the handle: the overlay may re-render before the shot.
	const shown = page.locator(`[data-slot="${slot}"]`).filter({ visible: true }).first()
	return portalled || rootBox.height >= VIEWPORT.height ? shown : root
}

async function main() {
	const browser = await chromium.launch()
	const page = await browser.newPage({ deviceScaleFactor: 2, viewport: VIEWPORT })
	let failed = 0
	for (const job of jobs) {
		try {
			await shoot(page, job)
			process.stdout.write('.')
		} catch (err) {
			// One story that won't show its part leaves that cell empty, not the whole column.
			failed++
			process.stdout.write('x')
			console.error(`\n${job.file}: ${err.message.split('\n')[0]}`)
		}
	}
	process.stdout.write('\n')
	if (failed) console.error(`${failed} of ${jobs.length} core-react shots failed`)
	await browser.close()
}

/** Opens a job's story with its args and theme, and shoots it. */
async function shoot(page, job) {
	await page.goto(`${base}/iframe.html?id=${job.story}&viewMode=story`)
	await page.waitForFunction(() => window.__STORYBOOK_ADDONS_CHANNEL__ && document.querySelector('#storybook-root > *'))
	await page.addStyleTag({
		content: '*,*::before,*::after{transition:none!important;animation:none!important} [data-slot$="-overlay"]{display:none!important}',
	})
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
	const shot = await target(page, job.component)
	if (!job.component.startsWith('Tooltip')) {
		await page.evaluate(() => document.activeElement?.blur())
		await page.mouse.move(0, 0)
	}
	const box = await shot.boundingBox()
	await page.screenshot({
		path: path.join(out, job.file),
		clip: { x: box.x - MARGIN, y: box.y - MARGIN, width: box.width + 2 * MARGIN, height: box.height + 2 * MARGIN },
	})
}

main().catch((err) => {
	console.error(err.message)
	process.exit(1)
})
