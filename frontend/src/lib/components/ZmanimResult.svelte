<script lang="ts">
	interface Props {
		html: string;
		dataLoaded: boolean;
	}

	let { html, dataLoaded }: Props = $props();

	const message = $derived(
		!dataLoaded
			? 'Loading city data, please wait...'
			: html
				? null
				: 'Search for a city above to calculate Zmanim.'
	);
</script>

<div id="zmanim_result" class:loading={!html}>
	{#if message}
		{message}
	{:else}
		{@html html}
	{/if}
</div>
