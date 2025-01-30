import { getAllVersions } from '@/lib/version';
import { Container, Text, List, ListItem, ThemeIcon, rem, Anchor } from '@mantine/core';
import { IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

export default async function AllVersions() {
    const versions = await getAllVersions();
    return (
        <>
            <Container size="md">
                <Text mt="sm">Versions:</Text>
                {/* TODO: Replace icon with decompiler success if applicable. To do this we need suite result for each module. */}
                <List mt="sm" icon={
                    <ThemeIcon color="yellow" size={24} radius="xl">
                        <IconExclamationCircle style={{ width: rem(16), height: rem(16) }} />
                    </ThemeIcon>
                } withPadding>
                    {versions.map((version) => (
                        <ListItem key={version.gbfVersion}>
                            <Anchor component={Link} href={`/${version.gbfVersion}`}>{version.gbfVersion}</Anchor>
                        </ListItem>
                    ))}
                </List>
            </Container>
        </>

    );
}