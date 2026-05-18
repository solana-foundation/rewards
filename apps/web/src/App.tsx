import { Navigate, Route, Routes } from 'react-router';

import { AppLayout } from '@/components/app-layout';
import { AppProviders } from '@/components/app-providers';
import { ClaimRewards } from '@/routes/claim-rewards';
import { CreateRewards } from '@/routes/create-rewards';
import { Dashboard } from '@/routes/dashboard';
import { ManageRewards } from '@/routes/manage-rewards';

export function App() {
    return (
        <AppProviders>
            <Routes>
                <Route
                    path="/"
                    element={
                        <AppLayout>
                            <Dashboard />
                        </AppLayout>
                    }
                />
                <Route
                    path="/create"
                    element={
                        <AppLayout>
                            <CreateRewards />
                        </AppLayout>
                    }
                />
                <Route
                    path="/manage"
                    element={
                        <AppLayout>
                            <ManageRewards />
                        </AppLayout>
                    }
                />
                <Route
                    path="/claim"
                    element={
                        <AppLayout>
                            <ClaimRewards />
                        </AppLayout>
                    }
                />
                <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
        </AppProviders>
    );
}
