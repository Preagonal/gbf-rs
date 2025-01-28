import { getAllFunctions } from '@/lib/function';
import { List, ListItem, ThemeIcon, rem, Anchor, Box, Alert, Code } from '@mantine/core';
import { IconAlertCircle, IconCircleCheck, IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

type versionPromise = Promise<{ version: string, module: string }>;


export default async function Module(props: { params: versionPromise }) {
    const versionParam = (await props.params).version;
    const moduleParam = (await props.params).module;

    const functions = await getAllFunctions(versionParam, moduleParam);

    if (functions.length === 0) {
        return (
            <Box mt="sm">
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="No Functions"
                    color="yellow"
                    mt="sm"
                >
                    There are no functions with the module ID <Code>{moduleParam}</Code>.
                </Alert>
            </Box>
        )
    }

    return (
        <List mt="sm" spacing="xs" withPadding>
            {functions && functions.map((func) => (
                <ListItem
                    key={func.functionAddress}
                    icon={
                        func.decompileSuccess ? (
                            <ThemeIcon color="teal" size={24} radius="xl">
                                <IconCircleCheck style={{ width: rem(16), height: rem(16) }} />
                            </ThemeIcon>
                        ) : (
                            <ThemeIcon color="yellow" size={24} radius="xl">
                                <IconExclamationCircle style={{ width: rem(16), height: rem(16) }} />
                            </ThemeIcon>
                        )
                    }
                >
                    <Anchor component={Link} href={`/${versionParam}/${moduleParam}/${func.functionAddress}`}>
                        {func.functionName || "[entrypoint]"}
                    </Anchor>
                </ListItem>
            ))}
        </List>
    );
}
