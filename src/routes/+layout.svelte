<script lang="ts">
  import "../app.css";
  import { onMount, onDestroy } from "svelte";

  // Global Event Blockers for Production Polish
  function handleKeydown(event: KeyboardEvent) {
    // 1. Block Refresh (F5, Ctrl+R)
    if (event.key === "F5" || (event.ctrlKey && event.key === "r")) {
      event.preventDefault();
      console.log("Blocked Refresh Action");
    }

    // 2. Block Navigation (Alt + Left/Right) - Prevents back/forward history navigation
    if (
      event.altKey &&
      (event.key === "ArrowLeft" || event.key === "ArrowRight")
    ) {
      event.preventDefault();
      console.log("Blocked History Navigation");
    }
  }

  function handleContextMenu(event: MouseEvent) {
    // 3. Block Context Menu (Right Click)
    event.preventDefault();
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("contextmenu", handleContextMenu);
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("keydown", handleKeydown);
      window.removeEventListener("contextmenu", handleContextMenu);
    }
  });
</script>

<main>
  <slot />
</main>
