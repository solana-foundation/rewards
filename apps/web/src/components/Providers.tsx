'use client';

import { RecentTransactionsProvider } from '@/contexts/RecentTransactionsContext';
import { RpcProvider } from '@/contexts/RpcContext';
import { SavedValuesProvider } from '@/contexts/SavedValuesContext';
import { WalletProvider } from '@/contexts/WalletContext';

export function Providers({ children }: { children: React.ReactNode }) {
    return (
        <RpcProvider>
            <WalletProvider>
                <RecentTransactionsProvider>
                    <SavedValuesProvider>{children}</SavedValuesProvider>
                </RecentTransactionsProvider>
            </WalletProvider>
        </RpcProvider>
    );
}
