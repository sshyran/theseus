<script context="module" lang="ts">
  import { init, waitLocale, t, getLocaleFromAcceptLanguageHeader } from 'svelte-intl-precompile';
  import { registerAll, availableLocales } from '$locales';

  registerAll();

  export const load: import('@sveltejs/kit').Load = async ({ session }) => {
    init({
      fallbackLocale: 'en',
      initialLocale: getLocaleFromAcceptLanguageHeader(session.acceptLanguage, availableLocales)
    });
    await waitLocale();

    return {};
  };
</script>

<script lang="ts">
  import '@fontsource/inter';
  import 'omorphia/styles.postcss';
  import '$styles/global.postcss';
  import Sidebar from '$layout/Sidebar.svelte';
  import StatusBar from '$layout/StatusBar.svelte';
  import Page from '$layout/Page.svelte';
</script>

<div class="app base theme-dark">
  <Sidebar />
  <StatusBar />
  <Page>
    <slot />
  </Page>
</div>

<style lang="postcss">
  .app {
    height: 100%;
    width: 100%;
    display: grid;
    grid-template-areas:
      'sidebar status-bar'
      'sidebar page';
    grid-template-rows: 2.5rem 1fr;
    grid-template-columns: 14rem 1fr;
  }

  :global(.page) {
    grid-area: page;
  }

  :global(.sidebar) {
    grid-area: sidebar;
  }

  :global(.status-bar) {
    grid-area: status-bar;
  }
</style>
