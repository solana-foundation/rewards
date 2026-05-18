import { useMemo, useState } from 'react';
import { Link } from 'react-router';
import { useWallet } from '@solana/connector/react';
import { Badge, Button } from '@solana/design-system';
import { GitBranch, Plus, RefreshCw, Trash2, WalletCards } from 'lucide-react';

import { ClaimDirectRewardForm } from '@/components/rewards/claim/claim-direct-reward-form';
import { ClaimMerkleRewardForm } from '@/components/rewards/claim/claim-merkle-reward-form';
import { ImportProofDropForm } from '@/components/rewards/claim/import-proof-drop-form';
import { WalletButton } from '@/components/solana/solana-provider';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { toProofDropClaimInput, useProofDropClaims, type ProofDropClaimDraft } from '@/hooks/use-proof-drop-claims';
import { useClaimableRewards } from '@/hooks/use-rewards';
import {
    formatAddress,
    formatTokenAmount,
    type ClaimableReward,
    type ProofDropClaimInput,
    vestingLabel,
} from '@/lib/rewards-model';

function ClaimDialog({
    onOpenChange,
    open,
    reward,
}: {
    onOpenChange: (open: boolean) => void;
    open: boolean;
    reward: ClaimableReward;
}) {
    const title = reward.kind === 'proof' ? 'Claim Proof-Based Drop' : 'Claim Rewards';

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                </DialogHeader>
                {reward.kind === 'recipient' ? (
                    <ClaimDirectRewardForm
                        key={reward.allocation.address}
                        initialDistribution={reward.allocation.distribution}
                        initialMint={reward.allocation.mint ?? ''}
                        initialAmount="0"
                        onSuccess={() => onOpenChange(false)}
                        submitLabel="Claim Rewards"
                    />
                ) : (
                    <ClaimMerkleRewardForm
                        key={reward.allocation.id}
                        hideKnownFields
                        initialDistribution={reward.allocation.distribution}
                        initialMint={reward.allocation.mint ?? ''}
                        initialProof={reward.allocation.proofText}
                        initialSchedule={reward.allocation.scheduleState}
                        initialTotalAmount={String(reward.allocation.totalAmount)}
                        onSuccess={() => onOpenChange(false)}
                        submitLabel="Claim Rewards"
                    />
                )}
            </DialogContent>
        </Dialog>
    );
}

function ClaimableRewardCard({
    onRemoveProofDrop,
    reward,
}: {
    onRemoveProofDrop: (id: string) => void;
    reward: ClaimableReward;
}) {
    const [open, setOpen] = useState(false);
    const allocation = reward.allocation;
    const isProofDrop = reward.kind === 'proof';
    const addressValue = reward.kind === 'proof' ? reward.allocation.distribution : reward.allocation.address;
    const title = isProofDrop ? 'Proof-Based Drop' : 'Reward Allocation';
    const canClaim =
        Boolean(allocation.mint) &&
        allocation.remainingAmount > 0n &&
        (reward.kind === 'recipient' || reward.allocation.dropExists);
    const badgeLabel =
        reward.kind === 'proof' && !reward.allocation.dropExists
            ? 'Unavailable'
            : allocation.remainingAmount > 0n
              ? 'Claimable'
              : 'Claimed';
    const badgeVariant =
        reward.kind === 'proof' && !reward.allocation.dropExists
            ? 'warning'
            : allocation.remainingAmount > 0n
              ? 'info'
              : 'success';

    return (
        <>
            <Card className="border-0 border-all-dashed-medium bg-card transition-colors hover:bg-sand-100">
                <CardContent className="space-y-4 p-4">
                    <div className="flex items-start justify-between gap-3">
                        <div className="flex min-w-0 items-center gap-2">
                            {isProofDrop ? (
                                <GitBranch className="h-4 w-4 shrink-0 text-sand-1100" />
                            ) : (
                                <WalletCards className="h-4 w-4 shrink-0 text-sand-1100" />
                            )}
                            <div className="min-w-0">
                                <p className="font-semibold text-foreground">{title}</p>
                                <p className="truncate font-mono text-xs text-muted-foreground">
                                    {formatAddress(addressValue)}
                                </p>
                            </div>
                        </div>
                        <Badge variant={badgeVariant}>{badgeLabel}</Badge>
                    </div>

                    <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                            <p className="text-xs text-muted-foreground">Remaining</p>
                            <p className="font-semibold">{formatTokenAmount(allocation.remainingAmount)}</p>
                        </div>
                        <div>
                            <p className="text-xs text-muted-foreground">Total</p>
                            <p className="font-semibold">{formatTokenAmount(allocation.totalAmount)}</p>
                        </div>
                        <div className="col-span-2">
                            <p className="text-xs text-muted-foreground">Schedule</p>
                            <p className="text-sm">{vestingLabel(allocation.schedule)}</p>
                        </div>
                        {allocation.mint && (
                            <div className="col-span-2">
                                <p className="text-xs text-muted-foreground">Mint</p>
                                <p className="truncate font-mono text-xs">{allocation.mint}</p>
                            </div>
                        )}
                    </div>

                    <div className="flex flex-wrap gap-2">
                        <Button type="button" size="sm" disabled={!canClaim} onClick={() => setOpen(true)}>
                            Claim
                        </Button>
                        {reward.kind === 'proof' && (
                            <Button
                                type="button"
                                size="sm"
                                variant="secondary"
                                iconLeft={<Trash2 />}
                                onClick={() => onRemoveProofDrop(reward.allocation.id)}
                            >
                                Remove
                            </Button>
                        )}
                    </div>
                </CardContent>
            </Card>
            <ClaimDialog reward={reward} open={open} onOpenChange={setOpen} />
        </>
    );
}

function AddProofDropDialog({
    onImport,
    onOpenChange,
    open,
}: {
    onImport: (claim: ProofDropClaimDraft) => void;
    onOpenChange: (open: boolean) => void;
    open: boolean;
}) {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>Add Proof-Based Drop</DialogTitle>
                </DialogHeader>
                <ImportProofDropForm
                    onImport={claim => {
                        onImport(claim);
                        onOpenChange(false);
                    }}
                />
            </DialogContent>
        </Dialog>
    );
}

function isProofDropClaimInput(value: ProofDropClaimInput | null): value is ProofDropClaimInput {
    return value !== null;
}

export function ClaimRewards() {
    const { account } = useWallet();
    const connected = Boolean(account);
    const { addProofDropClaim, hydrated, proofDrops, removeProofDropClaim } = useProofDropClaims();
    const proofDropClaims = useMemo(
        () => proofDrops.map(toProofDropClaimInput).filter(isProofDropClaimInput),
        [proofDrops],
    );
    const { data, isFetching, isLoading, refetch } = useClaimableRewards(proofDropClaims);
    const rewards = data ?? [];
    const loading = isLoading || !hydrated;
    const [proofDropOpen, setProofDropOpen] = useState(false);

    if (!connected) {
        return (
            <div className="mx-auto max-w-3xl space-y-6">
                <h1 className="text-3xl font-semibold tracking-tight text-foreground">Claim Rewards</h1>
                <Card className="border-0 border-all-dashed-medium bg-card">
                    <CardContent className="flex flex-col items-center justify-center gap-4 py-14 text-center">
                        <WalletCards className="h-8 w-8 text-sand-1000" />
                        <p className="text-sm text-muted-foreground">Connect wallet to load rewards.</p>
                        <WalletButton />
                    </CardContent>
                </Card>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <h1 className="text-3xl font-semibold tracking-tight text-foreground">Claim Rewards</h1>
            <Card className="border-0 border-all-dashed-medium bg-card">
                <CardHeader className="pb-4">
                    <div className="flex flex-wrap items-center justify-between gap-3">
                        <div className="flex items-center gap-2">
                            <WalletCards className="h-5 w-5 text-foreground" />
                            <CardTitle>My Rewards</CardTitle>
                        </div>
                        <div className="flex items-center gap-2">
                            {rewards.length > 0 && <Badge variant="success">{rewards.length}</Badge>}
                            <Button
                                type="button"
                                size="sm"
                                variant="secondary"
                                iconLeft={<Plus />}
                                onClick={() => setProofDropOpen(true)}
                            >
                                Add Proof-Based Drop
                            </Button>
                            <Button
                                type="button"
                                size="sm"
                                variant="secondary"
                                iconOnly
                                iconLeft={<RefreshCw className={isFetching ? 'animate-spin' : ''} />}
                                aria-label="Refresh claimable rewards"
                                onClick={() => void refetch()}
                                disabled={isFetching}
                            />
                        </div>
                    </div>
                </CardHeader>
                <CardContent>
                    {loading ? (
                        <div className="flex items-center justify-center py-12 text-muted-foreground">
                            Loading rewards...
                        </div>
                    ) : rewards.length > 0 ? (
                        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                            {rewards.map(reward => (
                                <ClaimableRewardCard
                                    key={reward.id}
                                    reward={reward}
                                    onRemoveProofDrop={removeProofDropClaim}
                                />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col items-center justify-center gap-3 py-10 text-center">
                            <WalletCards className="h-8 w-8 text-sand-1000" />
                            <div className="space-y-1">
                                <h2 className="text-lg font-semibold text-foreground">No claimable rewards</h2>
                                <p className="text-sm text-muted-foreground">
                                    Add a proof-based drop or create rewards for recipients.
                                </p>
                            </div>
                            <div className="flex flex-wrap justify-center gap-2">
                                <Button
                                    type="button"
                                    variant="secondary"
                                    iconLeft={<Plus />}
                                    onClick={() => setProofDropOpen(true)}
                                >
                                    Add Proof-Based Drop
                                </Button>
                                <Button asChild variant="secondary">
                                    <Link to="/create">Create Rewards</Link>
                                </Button>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>

            <AddProofDropDialog open={proofDropOpen} onOpenChange={setProofDropOpen} onImport={addProofDropClaim} />
        </div>
    );
}
