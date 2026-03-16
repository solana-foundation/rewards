import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
    env: {
        NEXT_PUBLIC_PROGRAM_ID:
            process.env.NEXT_PUBLIC_PROGRAM_ID ?? 'REWArDioXgQJ2fZKkfu9LCLjQfRwYWVVfsvcsR5hoXi',
        NEXT_PUBLIC_RPC_URL: process.env.NEXT_PUBLIC_RPC_URL ?? 'https://api.devnet.solana.com',
    },
    transpilePackages: ['@solana/rewards-client'],
};

export default nextConfig;
