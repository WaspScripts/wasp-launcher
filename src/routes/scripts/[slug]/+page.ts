export const load = async ({ parent, params: { slug } }) => {
	const { scripts } = await parent()

	return { script: scripts![0] }
}
