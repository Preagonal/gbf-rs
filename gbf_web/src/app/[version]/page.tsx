import { getAllModules } from '@/lib/module';
import { List, ListItem, ThemeIcon, rem, Anchor, Alert, Box, Code } from '@mantine/core';
import { IconAlertCircle, IconCircleCheck, IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

type versionPromise = Promise<{ version: string }>;


export default async function Version(props: { params: versionPromise }) {
    const paramVersion = (await props.params).version;
    const modules = await getAllModules(paramVersion);

    if (modules.length === 0) {
        return (
            <Box mt="sm">
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="No Modules"
                    color="yellow"
                    mt="sm"
                >
                    There are no modules with the version ID <Code>{paramVersion}</Code>.
                </Alert>
            </Box>
        )
    }

    return (
        <List mt="sm" withPadding>
            {modules.map((module) => (
                <ListItem icon={
                    module.decompileSuccess ? (
                        <ThemeIcon color="teal" size={24} radius="xl">
                            <IconCircleCheck style={{ width: rem(16), height: rem(16) }} />
                        </ThemeIcon>
                    ) : (
                        <ThemeIcon color="yellow" size={24} radius="xl">
                            <IconExclamationCircle style={{ width: rem(16), height: rem(16) }} />
                        </ThemeIcon>
                    )
                } key={module.moduleId}>
                    <Anchor component={Link} href={`/${paramVersion}/${module.moduleId}`}>{module.fileName}</Anchor>
                </ListItem>
            ))}
        </List>

    );
}
