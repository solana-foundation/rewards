import type { Metadata } from 'next';
import './globals.css';
import '@solana/wallet-adapter-react-ui/styles.css';
import { Providers } from '@/components/Providers';

export const metadata: Metadata = {
    title: 'Rewards Program',
    description: 'Solana rewards program frontend',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en" className="dark">
            <body>
                <Providers>{children}</Providers>
            </body>
        </html>
    );
}
