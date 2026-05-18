import { useState } from 'react';
import { Button } from '@solana/design-system';
import { GitBranch, Send, Users } from 'lucide-react';

import { CreateDirectRewardForm } from '@/components/rewards/create/create-direct-reward-form';
import { CreateMerkleDropForm } from '@/components/rewards/create/create-merkle-drop-form';
import { AddDirectRecipientForm } from '@/components/rewards/manage/add-direct-recipient-form';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';

type CreateFlow = 'proof-drop' | 'recipient-list';

export function CreateRewards() {
    const [flow, setFlow] = useState<CreateFlow>('recipient-list');

    return (
        <div className="space-y-6">
            <div className="flex flex-wrap items-center justify-between gap-4">
                <h1 className="text-3xl font-semibold tracking-tight text-foreground">Create Rewards</h1>
                <div className="grid w-full grid-cols-2 rounded-full border bg-card p-1 sm:w-auto">
                    <Button
                        type="button"
                        size="sm"
                        variant={flow === 'recipient-list' ? 'primary' : 'secondary'}
                        iconLeft={<Users />}
                        onClick={() => setFlow('recipient-list')}
                        className={cn('justify-center', flow !== 'recipient-list' && 'shadow-none')}
                    >
                        Recipients
                    </Button>
                    <Button
                        type="button"
                        size="sm"
                        variant={flow === 'proof-drop' ? 'primary' : 'secondary'}
                        iconLeft={<GitBranch />}
                        onClick={() => setFlow('proof-drop')}
                        className={cn('justify-center', flow !== 'proof-drop' && 'shadow-none')}
                    >
                        Proof Drop
                    </Button>
                </div>
            </div>

            {flow === 'recipient-list' ? (
                <div className="grid gap-6 lg:grid-cols-2">
                    <Card className="border-0 border-all-dashed-medium bg-card">
                        <CardHeader className="flex-row items-center gap-2">
                            <Send className="h-5 w-5 text-foreground" />
                            <CardTitle>Create Reward</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <CreateDirectRewardForm />
                        </CardContent>
                    </Card>
                    <Card className="border-0 border-all-dashed-medium bg-card">
                        <CardHeader className="flex-row items-center gap-2">
                            <Users className="h-5 w-5 text-foreground" />
                            <CardTitle>Add Recipients</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <AddDirectRecipientForm submitLabel="Add Recipient" />
                        </CardContent>
                    </Card>
                </div>
            ) : (
                <Card className="max-w-3xl border-0 border-all-dashed-medium bg-card">
                    <CardHeader className="flex-row items-center gap-2">
                        <GitBranch className="h-5 w-5 text-foreground" />
                        <CardTitle>Proof-Based Drop</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <CreateMerkleDropForm />
                    </CardContent>
                </Card>
            )}
        </div>
    );
}
