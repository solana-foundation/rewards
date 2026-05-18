import { Gift, LayoutDashboard, PlusCircle, WalletCards } from 'lucide-react';
import { type LucideIcon } from 'lucide-react';

export interface NavItem {
    icon: LucideIcon;
    label: string;
    path: string;
}

export const NAV_ITEMS: NavItem[] = [
    { icon: LayoutDashboard, label: 'Dashboard', path: '/' },
    { icon: PlusCircle, label: 'Create Rewards', path: '/create' },
    { icon: Gift, label: 'Manage Rewards', path: '/manage' },
    { icon: WalletCards, label: 'Claim Rewards', path: '/claim' },
];
