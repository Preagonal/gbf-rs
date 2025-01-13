import { NavigationBar } from '@/components/nav';
import { createQueryBuilder } from '@/data/data-analyzer';
import { Container, Text, Title, List, ListItem, ThemeIcon, rem, Anchor } from '@mantine/core';
import { IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

export default async function Versions() {
    const queryBuilder = await createQueryBuilder();
    const versions = await queryBuilder.execute() as string[];

    return (
        <>
            <NavigationBar versions={versions} modules={[]} functions={[]} version={null} module={null} func={null} />
            <Container size="md">
                <Title order={1}>Welcome to the GBF Test Portal</Title>
                <Text mt="sm">
                    This was built to track decompiler and CFG progress.
                </Text>
                <Text mt="sm">Versions:</Text>
                <List mt="sm" icon={
                    <ThemeIcon color="yellow" size={24} radius="xl">
                        <IconExclamationCircle style={{ width: rem(16), height: rem(16) }} />
                    </ThemeIcon>
                } withPadding>
                    {versions && versions.map((version: string) => (
                        <ListItem key={version}>
                            <Anchor component={Link} href={`/${version}`}>{version}</Anchor>
                        </ListItem>
                    ))}
                </List>
            </Container>
        </>

    );
}
