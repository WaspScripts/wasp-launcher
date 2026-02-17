import { Channel } from "@tauri-apps/api/core"

interface ChannelEntry {
	name: string
	version: number
	stopped: boolean
}

const MAX_LOGS = 1000

class ChannelManager {
	private _logsBuffer: Record<number, string[]> = {}

	processes = $state<number[]>([])
	channels = $state<Record<number, ChannelEntry>>({})

	async createChannel(name: string): Promise<Channel<string>> {
		const channel = new Channel<string>()
		const id = channel.id

		this._logsBuffer[id] = []
		this.channels[id] = { name, version: 0, stopped: false }
		this.processes.push(id)

		channel.onmessage = (msg: string) => {
			const entry = this.channels[id]
			if (!entry || entry.stopped) return

			const buffer = this._logsBuffer[id]
			buffer.push(msg)

			if (buffer.length > MAX_LOGS) {
				buffer.shift()
			}

			entry.version++
		}

		return channel
	}

	getLogs(id: number): string[] {
		const entry = this.channels[id]
		if (!entry) return []
		entry.version
		return this._logsBuffer[id]
	}

	stopChannel(id: number) {
		if (this.channels[id]) this.channels[id].stopped = true
	}

	removeChannel(id: number) {
		if (!this.channels[id]) return
		delete this._logsBuffer[id]
		delete this.channels[id]
		this.processes = this.processes.filter((item) => item !== id)
	}
}

export const channelManager = new ChannelManager()
