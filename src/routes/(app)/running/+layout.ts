export const prerender = false
export const ssr = false

export const load = async ({ depends, params: { slug } }) => {
	depends("layout:running")
	const process = Number(slug)
	return { process }
}
