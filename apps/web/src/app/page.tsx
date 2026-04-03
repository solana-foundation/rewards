'use client';

import { useState } from 'react';
import { Button } from '@solana/design-system/button';
import { QuickDefaults } from '@/components/QuickDefaults';
import { RecentTransactions } from '@/components/RecentTransactions';
import { ProgramBadge } from '@/components/ProgramBadge';
import { RpcBadge } from '@/components/RpcBadge';
import { WalletButton } from '@/components/WalletButton';
import { AddDirectRecipient } from '@/components/instructions/AddDirectRecipient';
import { ClaimDirect } from '@/components/instructions/ClaimDirect';
import { ClaimMerkle } from '@/components/instructions/ClaimMerkle';
import { CloseDirectDistribution } from '@/components/instructions/CloseDirectDistribution';
import { CloseDirectRecipient } from '@/components/instructions/CloseDirectRecipient';
import { CloseMerkleClaim } from '@/components/instructions/CloseMerkleClaim';
import { CloseMerkleDistribution } from '@/components/instructions/CloseMerkleDistribution';
import { CreateDirectDistribution } from '@/components/instructions/CreateDirectDistribution';
import { CreateMerkleDistribution } from '@/components/instructions/CreateMerkleDistribution';
import { RevokeDirectRecipient } from '@/components/instructions/RevokeDirectRecipient';
import { RevokeMerkleClaim } from '@/components/instructions/RevokeMerkleClaim';

type InstructionId =
    | 'createDirectDistribution'
    | 'addDirectRecipient'
    | 'claimDirect'
    | 'revokeDirectRecipient'
    | 'closeDirectRecipient'
    | 'closeDirectDistribution'
    | 'createMerkleDistribution'
    | 'claimMerkle'
    | 'revokeMerkleClaim'
    | 'closeMerkleClaim'
    | 'closeMerkleDistribution';

const NAV: {
    group: string;
    items: { id: InstructionId; label: string }[];
}[] = [
    {
        group: 'DIRECT',
        items: [
            { id: 'createDirectDistribution', label: 'Create Distribution' },
            { id: 'addDirectRecipient', label: 'Add Recipient' },
            { id: 'claimDirect', label: 'Claim Direct' },
            { id: 'revokeDirectRecipient', label: 'Revoke Recipient' },
            { id: 'closeDirectRecipient', label: 'Close Recipient' },
            { id: 'closeDirectDistribution', label: 'Close Distribution' },
        ],
    },
    {
        group: 'MERKLE',
        items: [
            { id: 'createMerkleDistribution', label: 'Create Distribution' },
            { id: 'claimMerkle', label: 'Claim Merkle' },
            { id: 'revokeMerkleClaim', label: 'Revoke Claim' },
            { id: 'closeMerkleClaim', label: 'Close Claim' },
            { id: 'closeMerkleDistribution', label: 'Close Distribution' },
        ],
    },
];

const PANELS: Record<InstructionId, { title: string; component: React.ComponentType }> = {
    createDirectDistribution: { title: 'Create Direct Distribution', component: CreateDirectDistribution },
    addDirectRecipient: { title: 'Add Direct Recipient', component: AddDirectRecipient },
    claimDirect: { title: 'Claim Direct', component: ClaimDirect },
    revokeDirectRecipient: { title: 'Revoke Direct Recipient', component: RevokeDirectRecipient },
    closeDirectRecipient: { title: 'Close Direct Recipient', component: CloseDirectRecipient },
    closeDirectDistribution: { title: 'Close Direct Distribution', component: CloseDirectDistribution },
    createMerkleDistribution: { title: 'Create Merkle Distribution', component: CreateMerkleDistribution },
    claimMerkle: { title: 'Claim Merkle', component: ClaimMerkle },
    revokeMerkleClaim: { title: 'Revoke Merkle Claim', component: RevokeMerkleClaim },
    closeMerkleClaim: { title: 'Close Merkle Claim', component: CloseMerkleClaim },
    closeMerkleDistribution: { title: 'Close Merkle Distribution', component: CloseMerkleDistribution },
};

export default function HomePage() {
    const [active, setActive] = useState<InstructionId>('createDirectDistribution');
    const panel = PANELS[active];
    const Panel = panel.component;

    return (
        <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
            <header
                style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '12px 24px',
                    borderBottom: '1px solid var(--color-border)',
                    background: 'var(--color-card)',
                    position: 'sticky',
                    top: 0,
                    zIndex: 10,
                }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                    <span style={{ fontWeight: 700, fontSize: '1rem', color: 'var(--color-accent)' }}>
                        Rewards Program
                    </span>
                    <RpcBadge />
                    <ProgramBadge />
                </div>
                <WalletButton />
            </header>

            <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
                <nav
                    style={{
                        width: 230,
                        borderRight: '1px solid var(--color-border)',
                        padding: '16px 0',
                        flexShrink: 0,
                        overflowY: 'auto',
                    }}
                >
                    {NAV.map(({ group, items }) => (
                        <div key={group} style={{ marginBottom: 24 }}>
                            <div
                                style={{
                                    fontSize: '0.6875rem',
                                    fontWeight: 700,
                                    color: 'var(--color-muted)',
                                    letterSpacing: '0.08em',
                                    padding: '0 16px',
                                    marginBottom: 6,
                                }}
                            >
                                {group}
                            </div>
                            {items.map(item => (
                                <Button
                                    key={item.id}
                                    onClick={() => setActive(item.id)}
                                    variant={active === item.id ? 'primary' : 'secondary'}
                                    size="sm"
                                    style={{
                                        width: '100%',
                                        justifyContent: 'flex-start',
                                        borderRadius: 0,
                                    }}
                                >
                                    {item.label}
                                </Button>
                            ))}
                        </div>
                    ))}
                </nav>

                <main style={{ flex: 1, padding: '32px 40px', overflowY: 'auto' }}>
                    <QuickDefaults />
                    <RecentTransactions />
                    <h2
                        style={{
                            fontSize: '1.125rem',
                            fontWeight: 600,
                            marginBottom: 24,
                            paddingBottom: 16,
                            borderBottom: '1px solid var(--color-border)',
                        }}
                    >
                        {panel.title}
                    </h2>
                    <div style={{ maxWidth: 620 }}>
                        <Panel />
                    </div>
                </main>
            </div>
        </div>
    );
}
