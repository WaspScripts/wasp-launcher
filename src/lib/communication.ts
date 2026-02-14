import { Channel } from "@tauri-apps/api/core"

export let running: number[] = []
export const channels: Record<number, Channel<string>> = {}

export async function createChannel(name: string): Promise<Channel<string>> {
	const channel = new Channel<string>()

	channel.onmessage = (message) => {
		console.log(`${message}`)
	}

	running.push(channel.id)
	channels[channel.id] = channel
	return channel
}

export async function removeChannel(id: number) {
	const channel = channels[id]
	if (channel == null) return
	delete channels[id]
	console.log(`Channel "${id}" removed.`)
	running = running.filter((item) => item !== id)
}
