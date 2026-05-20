import { useEffect, useRef, useState } from 'react';

import { TOKEN_PROGRAM_ID } from '@/lib/program';

import { useClusterConfig } from './use-cluster-config';

export const DEVNET_USDC_MINT = '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU';
export const MAINNET_USDC_MINT = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';

function getClusterUsdcMint(clusterId: string) {
    if (clusterId === 'solana:devnet') return DEVNET_USDC_MINT;
    if (clusterId === 'solana:mainnet') return MAINNET_USDC_MINT;
    return '';
}

export function useTokenFormDefaults(initialMint = '') {
    const { id } = useClusterConfig();
    const clusterMint = getClusterUsdcMint(id);
    const previousClusterMintRef = useRef(clusterMint);
    const [mint, setMint] = useState(initialMint || clusterMint);
    const [tokenProgram, setTokenProgram] = useState<string>(TOKEN_PROGRAM_ID);

    useEffect(() => {
        const previousClusterMint = previousClusterMintRef.current;
        previousClusterMintRef.current = clusterMint;

        if (initialMint) return;

        setMint(current => {
            if (!current) return clusterMint;
            if (current === previousClusterMint) return clusterMint;
            return current;
        });
    }, [clusterMint, initialMint]);

    return {
        clusterMint,
        mint,
        setMint,
        setTokenProgram,
        tokenProgram,
    };
}
