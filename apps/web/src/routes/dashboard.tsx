import { Link } from 'react-router';
import { useMemo } from 'react';
import { useWallet } from '@solana/connector/react';
import { Badge, Button } from '@solana/design-system';
import { ArrowRight, GitBranch, Gift, Inbox, Plus, WalletCards } from 'lucide-react';

import { RecentTransactions } from '@/components/RecentTransactions';
import { WalletButton } from '@/components/solana/solana-provider';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { toProofDropClaimInput, useProofDropClaims } from '@/hooks/use-proof-drop-claims';
import { useClaimableRewards, useCreatedRewards } from '@/hooks/use-rewards';
import { formatTokenAmount, rewardStatus, type ProofDropClaimInput } from '@/lib/rewards-model';

function SummaryLinkCard({
    icon: Icon,
    rows,
    title,
    to,
}: {
    icon: React.ComponentType<{ className?: string }>;
    rows: Array<{ label: string; value: string }>;
    title: string;
    to: string;
}) {
    return (
        <Link
            to={to}
            className="group relative flex flex-col overflow-hidden rounded-xl border-0 border-all-dashed-medium bg-card transition-colors hover:bg-sand-100"
        >
            <div className="flex-grow p-5">
                <div className="mb-6 flex items-center justify-between gap-3">
                    <div className="flex items-center gap-2">
                        <Icon className="h-5 w-5 text-sand-1100" />
                        <h2 className="text-[17px] font-semibold tracking-tight text-foreground">{title}</h2>
                    </div>
                    <ArrowRight className="h-4 w-4 text-sand-1000 transition-transform group-hover:translate-x-0.5" />
                </div>

                <div className="space-y-4">
                    {rows.map((row, index) => (
                        <div key={row.label} className="space-y-4">
                            {index > 0 && <div className="h-px w-full bg-sand-100" />}
                            <div className="flex items-center justify-between gap-3 text-sm">
                                <span className="text-sand-1100">{row.label}</span>
                                <span className="truncate text-base font-bold text-foreground">{row.value}</span>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </Link>
    );
}

function isProofDropClaimInput(value: ProofDropClaimInput | null): value is ProofDropClaimInput {
    return value !== null;
}

export function Dashboard() {
    const { account } = useWallet();
    const connected = Boolean(account);
    const { proofDrops } = useProofDropClaims();
    const proofDropClaims = useMemo(
        () => proofDrops.map(toProofDropClaimInput).filter(isProofDropClaimInput),
        [proofDrops],
    );
    const createdRewards = useCreatedRewards();
    const claimableRewards = useClaimableRewards(proofDropClaims);

    const campaigns = createdRewards.data ?? [];
    const allocations = claimableRewards.data ?? [];
    const totalAllocated = campaigns.reduce((sum, item) => sum + item.totalAmount, 0n);
    const totalOutstanding = campaigns.reduce(
        (sum, item) => sum + (item.totalAmount > item.claimedAmount ? item.totalAmount - item.claimedAmount : 0n),
        0n,
    );
    const totalClaimable = allocations.reduce((sum, item) => sum + item.allocation.remainingAmount, 0n);
    const activeCampaigns = campaigns.filter(campaign => rewardStatus(campaign) !== 'complete').length;
    const proofAllocations = allocations.filter(allocation => allocation.kind === 'proof');
    const claimableProofDrops = proofAllocations.filter(
        allocation => allocation.allocation.remainingAmount > 0n,
    ).length;
    const nextAction =
        totalClaimable > 0n
            ? {
                  description: `${allocations.length} reward${allocations.length === 1 ? '' : 's'} ready to claim.`,
                  label: 'Claim Rewards',
                  secondaryHref: '/manage',
                  secondaryLabel: 'Manage Rewards',
                  to: '/claim',
              }
            : campaigns.length > 0
              ? {
                    description:
                        activeCampaigns > 0
                            ? `${activeCampaigns} active reward${activeCampaigns === 1 ? '' : 's'} to review.`
                            : 'All created rewards are complete.',
                    label: 'Manage Rewards',
                    secondaryHref: '/create',
                    secondaryLabel: 'Create Rewards',
                    to: '/manage',
                }
              : proofDropClaims.length > 0
                ? {
                      description: `${proofDropClaims.length} proof drop${proofDropClaims.length === 1 ? '' : 's'} imported.`,
                      label: 'Review Proof Drops',
                      secondaryHref: '/create',
                      secondaryLabel: 'Create Rewards',
                      to: '/claim',
                  }
                : {
                      description: 'Create rewards or add a proof-based drop to claim.',
                      label: 'Create Rewards',
                      secondaryHref: '/claim',
                      secondaryLabel: 'Claim Rewards',
                      to: '/create',
                  };

    if (!connected) {
        return (
            <div className="mx-auto max-w-3xl space-y-6">
                <h1 className="text-3xl font-semibold tracking-tight text-foreground">Rewards</h1>
                <Card className="border-0 border-all-dashed-medium bg-card">
                    <CardContent className="flex flex-col items-center justify-center gap-4 py-14 text-center">
                        <WalletCards className="h-8 w-8 text-sand-1000" />
                        <div className="space-y-1">
                            <h2 className="text-lg font-semibold text-foreground">Connect wallet</h2>
                            <p className="text-sm text-muted-foreground">Rewards data loads from your wallet.</p>
                        </div>
                        <WalletButton />
                    </CardContent>
                </Card>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <div className="flex flex-wrap items-center justify-between gap-4">
                <div className="space-y-2">
                    <h1 className="text-3xl font-semibold tracking-tight text-foreground">Dashboard</h1>
                    <div className="flex items-center gap-2">
                        <Badge variant="info">
                            {createdRewards.isFetching || claimableRewards.isFetching ? 'Syncing' : 'Live'}
                        </Badge>
                    </div>
                </div>
                <Button asChild iconLeft={<Plus />} radius="round">
                    <Link to="/create">Create Rewards</Link>
                </Button>
            </div>

            <div className="grid gap-4 md:grid-cols-3">
                <SummaryLinkCard
                    icon={Gift}
                    title="Created Rewards"
                    to="/manage"
                    rows={[
                        { label: 'Total', value: campaigns.length.toString() },
                        { label: 'Outstanding', value: formatTokenAmount(totalOutstanding) },
                    ]}
                />
                <SummaryLinkCard
                    icon={Inbox}
                    title="Claim Rewards"
                    to="/claim"
                    rows={[
                        { label: 'Claimable', value: allocations.length.toString() },
                        { label: 'Amount', value: formatTokenAmount(totalClaimable) },
                    ]}
                />
                <SummaryLinkCard
                    icon={GitBranch}
                    title="Proof Drops"
                    to="/claim"
                    rows={[
                        { label: 'Imported', value: proofDropClaims.length.toString() },
                        { label: 'Claimable', value: claimableProofDrops.toString() },
                    ]}
                />
            </div>

            <div className="grid gap-4 lg:grid-cols-[1fr_360px]">
                <Card className="border-0 border-all-dashed-medium bg-card">
                    <CardHeader>
                        <CardTitle>Total Allocated</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-4xl font-semibold tracking-tight text-foreground">
                            {formatTokenAmount(totalAllocated)}
                        </p>
                    </CardContent>
                </Card>
                <Card className="border-0 border-all-dashed-medium bg-card">
                    <CardHeader>
                        <CardTitle>Next Action</CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-3">
                        <p className="text-sm text-muted-foreground">{nextAction.description}</p>
                        <Button asChild iconRight={<ArrowRight />}>
                            <Link to={nextAction.to}>{nextAction.label}</Link>
                        </Button>
                        <Button asChild variant="secondary">
                            <Link to={nextAction.secondaryHref}>{nextAction.secondaryLabel}</Link>
                        </Button>
                    </CardContent>
                </Card>
            </div>

            <RecentTransactions />
        </div>
    );
}
