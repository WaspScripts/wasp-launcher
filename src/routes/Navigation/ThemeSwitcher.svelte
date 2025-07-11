<script lang="ts">
	import { page } from "$app/state"
	import { Popover } from "@skeletonlabs/skeleton-svelte"
	import { ChevronDown, Palette, X } from "@lucide/svelte"

	const themesData = [
		{ label: "Cerberus", value: "cerberus" },
		{ label: "Concord", value: "concord" },
		{ label: "Fennec", value: "fennec" },
		{ label: "Wasp", value: "wasp" }
	]

	const { settings, theme } = $derived(page.data)

	// svelte-ignore state_referenced_locally
	let current = $state(theme) as string
	let open = $state(false)

	async function updateTheme(value: string) {
		current = value
		document.body.setAttribute("data-theme", current)
		await settings.set("theme", current)
	}
</script>

<div class="input-group hover:preset-tonal my-auto flex">
	<Popover
		{open}
		onOpenChange={(e) => (open = e.open)}
		positioning={{ placement: "bottom" }}
		triggerBase="btn hover:preset-tonal h-full"
		contentBase="card bg-surface-200-800 p-4 space-y-4 max-w-[320px]"
		zIndex="50"
	>
		{#snippet trigger()}
			<Palette size="16" />
			<span class="mx-4 my-auto flex w-16">{current}</span>
			<ChevronDown size="16" />{/snippet}
		{#snippet content()}
			<div class="card w-52">
				<header class="flex justify-between">
					<p class="text-xl font-bold">Themes</p>
					<button class="btn-icon hover:preset-tonal" onclick={() => (open = false)}><X /></button>
				</header>
				<div class="my-4 flex flex-col">
					{#each themesData as entry (entry.value)}
						<button
							type="submit"
							class="btn preset-outlined-surface-500 hover:border-primary-500 my-2"
							onclick={async () => await updateTheme(entry.value)}
						>
							{entry.label}
						</button>
					{/each}
				</div>
			</div>
		{/snippet}
	</Popover>
</div>
