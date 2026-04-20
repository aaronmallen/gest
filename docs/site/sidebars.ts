import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/how-to-use-gest',
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/concepts',
        'getting-started/agents',
      ],
    },
    {
      type: 'category',
      label: 'CLI Reference',
      items: [
        'cli/init',
        'cli/task',
        'cli/artifact',
        'cli/iteration',
        'cli/tag',
        'cli/search',
        'cli/project',
        'cli/purge',
        'cli/migrate',
        'cli/undo',
        'cli/serve',
        'cli/config',
        'cli/generate',
        'cli/self-update',
        'cli/version',
        'cli/exit-codes',
      ],
    },
    {
      type: 'category',
      label: 'Configuration',
      items: [
        'configuration/index',
        'configuration/theming',
      ],
    },
    {
      type: 'category',
      label: 'Migration',
      items: [
        'migration/v0-5-to-v0-6',
      ],
    },
    'why-gest',
    'faq',
    'changelog',
  ],
};

export default sidebars;
