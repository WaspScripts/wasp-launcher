<script lang="ts">
	import { goto, invalidate } from "$app/navigation"
	import { channelManager } from "$lib/communication.svelte"
	import { Copy, SearchIcon, Square, X } from "@lucide/svelte"
	import { invoke } from "@tauri-apps/api/core"
	import { onDestroy } from "svelte"

	let { children, data } = $props()

	const { process, channel } = $derived(data)
	let search = $state("")

	$effect.pre(() => {
		process
		search = ""
	})

	const [stopped, running] = $derived.by(() => {
		const query = search.trim().toLowerCase()
		return channelManager.processes.reduce<[number[], number[]]>(
			(acc, idx) => {
				const entry = channelManager.channels[idx]
				if (!entry || (query && !entry.name.toLowerCase().includes(query))) return acc
				entry.stopped ? acc[0].push(idx) : acc[1].push(idx)
				return acc
			},
			[[], []]
		)
	})

	const hasProcesses = $derived(channelManager.processes.length > 0)

	function getRuntime(start: number, finish: number): string {
		const time = finish - start

		const totalSeconds = Math.floor(time / 1000)

		const hours = Math.floor(totalSeconds / 3600)
		const minutes = Math.floor((totalSeconds % 3600) / 60)
		const seconds = totalSeconds % 60

		return `${hours.toString().padStart(2, "0")}:${minutes
			.toString()
			.padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`
	}

	let now = $state(Date.now())

	const runtime = $derived(
		channel ? getRuntime(channel.start, channel.stopped ? channel.finish : now) : "00:00:00"
	)

	const runtimeInterval = setInterval(() => (now = Date.now()), 1000)

	onDestroy(() => clearInterval(runtimeInterval))
</script>

<aside
	class="flex h-full max-w-96 min-w-44 flex-col gap-2 border-r border-surface-500 p-2 text-sm lg:min-w-64"
>
	<div class="input-group grid-cols-[auto_1fr_auto] overflow-visible text-scaling">
		<div class="ig-cell px-1 md:px-2">
			<SearchIcon size={16} />
		</div>
		<input
			type="text"
			placeholder="Search script..."
			class="ig-input h-full placeholder:text-surface-600-400"
			bind:value={search}
		/>
	</div>

	<ul class="h-full w-full overflow-y-scroll pr-2">
		{#each running as entry (entry)}
			<li
				class="flex preset-outlined-surface-200-800 text-sm hover:preset-tonal focus:preset-tonal"
				class:bg-surface-300-700={process === entry}
				class:border-primary-300-700={process === entry}
			>
				<a href={"/running/" + entry} class="flex w-full justify-between px-2 py-2">
					{channelManager.channels[entry].name}
				</a>
			</li>
		{/each}

		{#each stopped as entry (entry)}
			<li
				class="flex preset-outlined-surface-200-800 text-surface-700-300 hover:preset-tonal hover:text-surface-800-200 focus:preset-tonal"
				class:bg-surface-300-700={process === entry}
				class:border-primary-300-700={process === entry}
			>
				<a href={"/running/" + entry} class="flex w-full justify-between px-2 py-2">
					{channelManager.channels[entry].name}
				</a>
			</li>
		{/each}
	</ul>
</aside>

<main class="flex h-full w-full overflow-y-auto">
	<div class="relative flex h-full w-full flex-col overflow-hidden">
		{#if hasProcesses}
			<div class="absolute right-0 mx-4 flex justify-end gap-2 p-4">
				<div class="rounded-lg border border-surface-500 bg-surface-500/65 p-2">
					{runtime}
				</div>
				<button
					class="btn btn-group rounded-lg border border-surface-500 bg-surface-500/65 p-2"
					onclick={async () => {
						if (!channel) return
						const data = channelManager.getLogs(process)
						const lines = data.map((log) => {
							if (log.close) return log.text + "\n"
							return log.text + " "
						})
						await navigator.clipboard.writeText(lines.join(""))
					}}
				>
					<span> Copy </span>
					<Copy size={16} />
				</button>
				{#if channel && !channel.stopped}
					<button
						class="btn btn-group flex gap-2 rounded-lg border border-surface-500 bg-surface-500/70 p-2"
						onclick={async () => {
							const result = await invoke("kill_script", { id: process })
							console.log("kill_script: ", result)
						}}
					>
						<span> Stop </span>
						<Square size={16} />
					</button>
				{:else}
					<button
						class="btn rounded-lg border border-surface-500 bg-surface-500/70 p-2"
						onclick={async () => {
							channelManager.removeChannel(process)
							await Promise.all([invalidate("layout:channel"), invalidate("layout:running")])
							await goto("/running")
						}}
					>
						<span> Close </span>
						<X size={16} />
					</button>
				{/if}
			</div>
		{/if}

		<div
			id="running-container"
			class="block min-h-full w-full min-w-fit gap-2 px-4 text-left wrap-break-word whitespace-break-spaces"
			class:bg-stone-950={hasProcesses}
			class:overflow-y-scroll={hasProcesses}
		>
			{@render children()}
		</div>
	</div>
</main>
