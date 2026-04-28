<script lang="ts">
	import { onMount } from "svelte";
	import { warmup } from "$lib";
	import "../app.css";

	let { children } = $props();

	onMount(() => {
		void warmup()
			.catch((error) => {
				console.error("Failed to warm up WASM:", error);
			})
			.then(() => {
				console.log("wasm warmed up");
			});
	});
</script>

{@render children()}
