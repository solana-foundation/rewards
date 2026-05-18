import { useState } from 'react';
import { Link } from 'react-router';
import { useWallet } from '@solana/connector/react';
import { Badge, Button } from '@solana/design-system';
import { GitBranch, Plus, RefreshCw, Trash2, Undo2, Users } from 'lucide-react';

import { AddDirectRecipientForm } from '@/components/rewards/manage/add-direct-recipient-form';
import { CloseDirectRewardForm } from '@/components/rewards/manage/close-direct-reward-form';
import { CloseMerkleDropForm } from '@/components/rewards/manage/close-merkle-drop-form';
import { RevokeDirectRecipientForm } from '@/components/rewards/manage/revoke-direct-recipient-form';
import { RevokeMerkleClaimForm } from '@/components/rewards/manage/revoke-merkle-claim-form';
import { WalletButton } from '@/components/solana/solana-provider';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { useCreatedRewards } from '@/hooks/use-rewards';
import { formatAddress, formatTokenAmount, rewardStatus, type RewardCampaign } from '@/lib/rewards-model';

type ManageAction = 'add-recipient' | 'close' | 'revoke';
type DialogStep = 'confirm' | 'form';

function statusBadge(campaign: RewardCampaign) {
    const status = rewardStatus(campaign);
    if (status === 'complete') return <Badge variant="success">Complete</Badge>;
    if (status === 'clawback-ready') return <Badge variant="warning">Ready to close</Badge>;
    return <Badge variant="info">Open</Badge>;
}

function canCloseReward(campaign: RewardCampaign) {
    return campaign.clawbackTs === 0n || BigInt(Math.floor(Date.now() / 1000)) >= campaign.clawbackTs;
}

function formatClawbackTs(value: bigint) {
    if (value === 0n) return 'Anytime';
    return new Intl.DateTimeFormat('en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
    }).format(new Date(Number(value) * 1000));
}

function ActionSummary({ campaign }: { campaign: RewardCampaign }) {
    const outstanding =
        campaign.totalAmount > campaign.claimedAmount ? campaign.totalAmount - campaign.claimedAmount : 0n;

    return (
        <div className="grid gap-3 rounded-lg border border-sand-300 bg-sand-100 p-3 text-sm">
            <div>
                <p className="text-xs text-muted-foreground">Reward</p>
                <p className="break-all font-mono text-xs text-foreground">{campaign.address}</p>
            </div>
            <div>
                <p className="text-xs text-muted-foreground">Mint</p>
                <p className="truncate font-mono text-xs text-foreground">{campaign.mint}</p>
            </div>
            <div className="grid grid-cols-2 gap-3">
                <div>
                    <p className="text-xs text-muted-foreground">Outstanding</p>
                    <p className="font-semibold">{formatTokenAmount(outstanding)}</p>
                </div>
                <div>
                    <p className="text-xs text-muted-foreground">Close Time</p>
                    <p className="font-semibold">{formatClawbackTs(campaign.clawbackTs)}</p>
                </div>
            </div>
        </div>
    );
}

function ActionConfirmation({
    action,
    campaign,
    onCancel,
    onContinue,
}: {
    action: Exclude<ManageAction, 'add-recipient'>;
    campaign: RewardCampaign;
    onCancel: () => void;
    onContinue: () => void;
}) {
    const isClose = action === 'close';
    const canContinue = isClose ? canCloseReward(campaign) : campaign.revocable;
    const title = isClose ? 'Close Reward' : 'Revoke Reward';
    const description = isClose
        ? 'Closing returns any remaining reward tokens to your wallet and closes the reward account.'
        : 'Revoking changes the recipient allocation and may return unvested rewards to your wallet.';
    const blockedDescription = isClose
        ? `This reward cannot be closed until ${formatClawbackTs(campaign.clawbackTs)}.`
        : 'This reward was created without revocation enabled.';

    return (
        <div className="space-y-4">
            <DialogHeader>
                <DialogTitle className="text-destructive">{title}</DialogTitle>
                <DialogDescription>{canContinue ? description : blockedDescription}</DialogDescription>
            </DialogHeader>
            <ActionSummary campaign={campaign} />
            <DialogFooter>
                <Button type="button" variant="secondary" onClick={onCancel}>
                    Cancel
                </Button>
                <Button
                    type="button"
                    className="bg-destructive text-white hover:bg-destructive/90"
                    disabled={!canContinue}
                    onClick={onContinue}
                >
                    Continue
                </Button>
            </DialogFooter>
        </div>
    );
}

function ManageDialog({
    action,
    campaign,
    onOpenChange,
}: {
    action: ManageAction | null;
    campaign: RewardCampaign;
    onOpenChange: (open: boolean) => void;
}) {
    const open = action !== null;
    const initialDistribution = campaign.address;
    const initialMint = campaign.mint;
    const [step, setStep] = useState<DialogStep>('confirm');

    const closeDialog = (nextOpen: boolean) => {
        if (!nextOpen) setStep('confirm');
        onOpenChange(nextOpen);
    };

    const formTitle =
        action === 'add-recipient' ? 'Add Recipient' : action === 'revoke' ? 'Configure Revocation' : 'Close Reward';

    return (
        <Dialog open={open} onOpenChange={closeDialog}>
            <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
                {action !== null && action !== 'add-recipient' && step === 'confirm' ? (
                    <ActionConfirmation
                        action={action}
                        campaign={campaign}
                        onCancel={() => closeDialog(false)}
                        onContinue={() => setStep('form')}
                    />
                ) : (
                    <DialogHeader>
                        <DialogTitle>{formTitle}</DialogTitle>
                        {action !== 'add-recipient' && (
                            <DialogDescription>
                                Reward address and mint are already selected from the card.
                            </DialogDescription>
                        )}
                    </DialogHeader>
                )}
                {action === 'add-recipient' && (
                    <AddDirectRecipientForm
                        key={`${campaign.address}-add`}
                        initialDistribution={initialDistribution}
                        initialMint={initialMint}
                        onSuccess={() => closeDialog(false)}
                        submitLabel="Add Recipient"
                    />
                )}
                {action === 'revoke' &&
                    step === 'form' &&
                    (campaign.kind === 'direct' ? (
                        <RevokeDirectRecipientForm
                            key={`${campaign.address}-revoke-direct`}
                            hideKnownFields
                            initialDistribution={initialDistribution}
                            initialMint={initialMint}
                            onSuccess={() => closeDialog(false)}
                            submitLabel="Revoke Reward"
                        />
                    ) : (
                        <RevokeMerkleClaimForm
                            key={`${campaign.address}-revoke-merkle`}
                            hideKnownFields
                            initialDistribution={initialDistribution}
                            initialMint={initialMint}
                            onSuccess={() => closeDialog(false)}
                            submitLabel="Revoke Reward"
                        />
                    ))}
                {action === 'close' &&
                    step === 'form' &&
                    (campaign.kind === 'direct' ? (
                        <CloseDirectRewardForm
                            key={`${campaign.address}-close-direct`}
                            hideKnownFields
                            initialDistribution={initialDistribution}
                            initialMint={initialMint}
                            onSuccess={() => closeDialog(false)}
                            submitLabel="Close Reward"
                        />
                    ) : (
                        <CloseMerkleDropForm
                            key={`${campaign.address}-close-merkle`}
                            hideKnownFields
                            initialDistribution={initialDistribution}
                            initialMint={initialMint}
                            onSuccess={() => closeDialog(false)}
                            submitLabel="Close Reward"
                        />
                    ))}
            </DialogContent>
        </Dialog>
    );
}

function RewardCard({ campaign }: { campaign: RewardCampaign }) {
    const [action, setAction] = useState<ManageAction | null>(null);
    const outstanding =
        campaign.totalAmount > campaign.claimedAmount ? campaign.totalAmount - campaign.claimedAmount : 0n;
    const closeable = canCloseReward(campaign);

    return (
        <>
            <Card className="border-0 border-all-dashed-medium bg-card transition-colors hover:bg-sand-100">
                <CardContent className="space-y-4 p-4">
                    <div className="flex items-start justify-between gap-3">
                        <div className="flex items-center gap-2">
                            {campaign.kind === 'direct' ? (
                                <Users className="h-4 w-4 text-sand-1100" />
                            ) : (
                                <GitBranch className="h-4 w-4 text-sand-1100" />
                            )}
                            <div>
                                <p className="font-semibold text-foreground">
                                    {campaign.kind === 'direct' ? 'Recipient Rewards' : 'Proof-Based Drop'}
                                </p>
                                <p className="font-mono text-xs text-muted-foreground">
                                    {formatAddress(campaign.address)}
                                </p>
                            </div>
                        </div>
                        {statusBadge(campaign)}
                    </div>

                    <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                            <p className="text-xs text-muted-foreground">Allocated</p>
                            <p className="font-semibold">{formatTokenAmount(campaign.totalAmount)}</p>
                        </div>
                        <div>
                            <p className="text-xs text-muted-foreground">Outstanding</p>
                            <p className="font-semibold">{formatTokenAmount(outstanding)}</p>
                        </div>
                        <div>
                            <p className="text-xs text-muted-foreground">Revocation</p>
                            <p className="font-semibold">{campaign.revocable ? 'Enabled' : 'Disabled'}</p>
                        </div>
                        <div>
                            <p className="text-xs text-muted-foreground">Close Time</p>
                            <p className="font-semibold">{formatClawbackTs(campaign.clawbackTs)}</p>
                        </div>
                        <div className="col-span-2">
                            <p className="text-xs text-muted-foreground">Mint</p>
                            <p className="truncate font-mono text-xs">{campaign.mint}</p>
                        </div>
                    </div>

                    <div className="flex flex-wrap gap-2">
                        {campaign.kind === 'direct' && (
                            <Button
                                type="button"
                                size="sm"
                                variant="secondary"
                                iconLeft={<Plus />}
                                onClick={() => setAction('add-recipient')}
                            >
                                Add Recipient
                            </Button>
                        )}
                        <Button
                            type="button"
                            size="sm"
                            variant="secondary"
                            iconLeft={<Undo2 />}
                            onClick={() => setAction('revoke')}
                            disabled={!campaign.revocable}
                        >
                            Revoke Reward
                        </Button>
                        <Button
                            type="button"
                            size="sm"
                            variant="secondary"
                            iconLeft={<Trash2 />}
                            onClick={() => setAction('close')}
                            disabled={!closeable}
                        >
                            Close Reward
                        </Button>
                    </div>
                </CardContent>
            </Card>
            <ManageDialog action={action} campaign={campaign} onOpenChange={open => !open && setAction(null)} />
        </>
    );
}

export function ManageRewards() {
    const { account } = useWallet();
    const connected = Boolean(account);
    const { data, isFetching, isLoading, refetch } = useCreatedRewards();
    const campaigns = data ?? [];

    if (!connected) {
        return (
            <div className="mx-auto max-w-3xl space-y-6">
                <h1 className="text-3xl font-semibold tracking-tight text-foreground">Manage Rewards</h1>
                <Card className="border-0 border-all-dashed-medium bg-card">
                    <CardContent className="flex flex-col items-center justify-center gap-4 py-14 text-center">
                        <p className="text-sm text-muted-foreground">Connect wallet to load created rewards.</p>
                        <WalletButton />
                    </CardContent>
                </Card>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <h1 className="text-3xl font-semibold tracking-tight text-foreground">Manage Rewards</h1>
            <Card className="relative overflow-hidden border-0 border-all-dashed-medium bg-card">
                <CardHeader className="pb-4">
                    <div className="flex items-center justify-between gap-4">
                        <div className="flex items-center gap-2">
                            <GiftIcon />
                            <CardTitle>My Rewards</CardTitle>
                        </div>
                        <div className="flex items-center gap-2">
                            {campaigns.length > 0 && <Badge variant="success">{campaigns.length}</Badge>}
                            <Button
                                type="button"
                                size="sm"
                                variant="secondary"
                                iconOnly
                                iconLeft={<RefreshCw className={isFetching ? 'animate-spin' : ''} />}
                                aria-label="Refresh rewards"
                                onClick={() => void refetch()}
                                disabled={isFetching}
                            />
                        </div>
                    </div>
                </CardHeader>
                <CardContent>
                    {isLoading ? (
                        <div className="flex items-center justify-center py-12 text-muted-foreground">
                            Loading rewards...
                        </div>
                    ) : campaigns.length > 0 ? (
                        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                            {campaigns.map(campaign => (
                                <RewardCard key={campaign.address} campaign={campaign} />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col items-center justify-center gap-3 py-10 text-center">
                            <GiftIcon className="h-8 w-8 text-sand-1000" />
                            <div className="space-y-1">
                                <h2 className="text-lg font-semibold text-foreground">No created rewards</h2>
                                <p className="text-sm text-muted-foreground">
                                    Create rewards first, then manage recipients and closeout here.
                                </p>
                            </div>
                            <div className="flex flex-wrap justify-center gap-2">
                                <Button asChild iconLeft={<Plus />} radius="round">
                                    <Link to="/create">Create Rewards</Link>
                                </Button>
                                <Button asChild variant="secondary">
                                    <Link to="/claim">Claim Rewards</Link>
                                </Button>
                            </div>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}

function GiftIcon({ className = 'h-5 w-5 text-foreground' }: { className?: string }) {
    return <Users className={className} />;
}
