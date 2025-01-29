import '@mantine/core/styles.css';
import {
    ColorSchemeScript,
    MantineProvider,
    Title,
    mantineHtmlProps,
    Text,
    Container
} from '@mantine/core';
import { ReactNode } from 'react';
import { NavigationBar } from '@/components/nav';

export const metadata = {
    title: 'GBF Web',
    description: 'The GBF decomplier and CFG tracker',
};

interface RootLayoutProps {
    children: ReactNode;
}

export default async function RootLayout({ children }: RootLayoutProps) {
    return (
        <html lang="en" {...mantineHtmlProps}>
            <head>
                <ColorSchemeScript />
            </head>
            <body>
                <MantineProvider defaultColorScheme="dark">
                    <NavigationBar />
                    <Container size="md" mb="sm">
                        <Title order={1}>Welcome to the GBF Test Portal</Title>
                        <Text mt="sm">
                            This was built to track decompiler and CFG progress.
                        </Text>
                        {children}
                    </Container>
                </MantineProvider>
            </body>
        </html>
    );
}

