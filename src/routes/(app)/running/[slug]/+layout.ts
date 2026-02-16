import { channelManager } from "$lib/communication.svelte"
import { redirect } from "@sveltejs/kit"

export const prerender = false
export const ssr = false

export const load = async ({ parent, depends }) => {
	const { process } = await parent()
	depends("layout:channel")
	const channel = channelManager.channels[process]
	if (!channel) redirect(303, "/running")
	return
}
