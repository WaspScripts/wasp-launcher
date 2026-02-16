import { Channel } from "@tauri-apps/api/core"

interface ChannelEntry {
	name: string
	channel: Channel<string>
	logs: string[]
	stopped: boolean
}

class ChannelManager {
	processes = $state<number[]>([])
	channels = $state<Record<number, ChannelEntry>>({})

	async createChannel(name: string): Promise<Channel<string>> {
		const channel = new Channel<string>()
		const id = channel.id

		this.processes.push(id)
		this.channels[id] = {
			name,
			channel,
			logs: [],
			stopped: false
		}

		this.channels[id].channel.onmessage = (msg: string) => {
			const entry = this.channels[id]
			if (entry && !entry.stopped) {
				entry.logs.push(msg)
			}
		}

		return this.channels[id].channel
	}

	stopChannel(id: number) {
		if (this.channels[id]) {
			this.channels[id].stopped = true
		}
	}

	removeChannel(id: number) {
		if (!this.channels[id]) return

		delete this.channels[id]
		this.processes = this.processes.filter((item) => item !== id)
		console.log(`Channel "${id}" removed.`)
	}
}

export const channelManager = new ChannelManager()
