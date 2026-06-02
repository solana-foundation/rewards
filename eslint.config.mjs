import solanaConfig from '@solana/eslint-config-solana';

export default [
    ...solanaConfig,
    {
        ignores: [
            '**/.next/**',
            '**/dist/**',
            '**/node_modules/**',
            '**/target/**',
            '**/generated/**',
            'clients/typescript/src/generated/**',
            '**/playwright-report/**',
            '**/test-results/**',
            '**/e2e/**',
            'apps/web/playwright.config.ts',
            'apps/web/postcss.config.mjs',
            'eslint.config.mjs',
            '.coverage/**',
            '**/.remember/**',
        ],
    },
];
