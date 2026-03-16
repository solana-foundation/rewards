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
            'apps/web/postcss.config.mjs',
            'eslint.config.mjs',
            '.coverage/**',
        ],
    },
];
