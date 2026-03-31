import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'gest',
  description: 'Track AI-agent-generated tasks and artifacts alongside your project',

  head: [
    ['meta', { name: 'theme-color', content: '#4EA8E0' }],
  ],

  themeConfig: {
    nav: [
      { text: 'Docs', link: '/getting-started/installation' },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Installation', link: '/getting-started/installation' },
          { text: 'Quick Start', link: '/getting-started/quick-start' },
          { text: 'Core Concepts', link: '/getting-started/concepts' },
          { text: 'Agent Usage', link: '/getting-started/agents' },
        ],
      },
      {
        text: 'CLI Reference',
        collapsed: false,
        items: [
          { text: 'init', link: '/cli/init' },
          { text: 'task', link: '/cli/task' },
          { text: 'artifact', link: '/cli/artifact' },
          { text: 'iteration', link: '/cli/iteration' },
          { text: 'search', link: '/cli/search' },
          { text: 'serve', link: '/cli/serve' },
          { text: 'config', link: '/cli/config' },
          { text: 'generate', link: '/cli/generate' },
          { text: 'self-update', link: '/cli/self-update' },
          { text: 'version', link: '/cli/version' },
        ],
      },
      {
        text: 'Configuration',
        items: [
          { text: 'Config Reference', link: '/configuration/' },
        ],
      },
      {
        text: 'Why gest?',
        link: '/why-gest',
      },
      {
        text: 'FAQ',
        link: '/faq',
      },
    ],

    search: {
      provider: 'local',
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/aaronmallen/gest' },
    ],

    editLink: {
      pattern: 'https://github.com/aaronmallen/gest/edit/main/docs/site/:path',
      text: 'Edit this page on GitHub',
    },

    footer: {
      message: 'Released under the MIT License.',
    },
  },
})
