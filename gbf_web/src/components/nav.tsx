"use client";

import React from 'react';
import { ActionIcon, Box, Button, rem, Select, Stack, Title } from '@mantine/core';
import Link from 'next/link';
import { FunctionId, ModuleId } from '@/data/gbf-suite-result-dao';
import { useRouter } from 'next/navigation';
import { IconBook } from '@tabler/icons-react';

interface NavigationBarProps {
    version: string | null;
    module: ModuleId | null;
    func: FunctionId | null;
    versions: string[];
    modules: ModuleId[] | null;
    functions: FunctionId[] | null;
}

export function NavigationBar({
    version,
    module,
    func,
    versions,
    modules,
    functions,
}: NavigationBarProps) {

    const createNavLink = (newVersion?: string | null, newModule?: string | null, newFunc?: string | null) => {
        let path = '/';
        if (newVersion) {
            path += newVersion;
        }
        if (newModule) {
            path += `/${newModule}`;
        }
        if (newFunc) {
            path += `/${newFunc}`;
        }
        return path;
    };

    const router = useRouter();

    let backLink = '';

    if (func) {
        backLink = createNavLink(version, module?.moduleId, null);
    } else if (module) {
        backLink = createNavLink(version, null, null);
    } else if (version) {
        backLink = '/';
    }

    return (
        <Box style={{ padding: '1rem', backgroundColor: '#444' }}>
            <Stack style={{ flexDirection: 'row', gap: '1rem' }}>
                <Title order={3}>GBF</Title>
                <Select
                    data={versions.map((v) => ({ value: v, label: v }))}
                    placeholder="Select version"
                    value={version}
                    key={version}
                    onChange={(value) => router.push(createNavLink(value, module?.moduleId, func?.functionAddress.toString()))}
                />
                <Select
                    data={modules?.map((m) => ({ value: m.moduleId, label: m.fileName })) || []}
                    placeholder="Select module"
                    value={module?.moduleId || ''}
                    onChange={(value) => router.push(createNavLink(version, value, func?.functionAddress.toString()))}
                    disabled={!modules || modules.length === 0}
                />
                <Select
                    data={functions?.map((f) => ({ value: f.functionAddress.toString(), label: f.functionName || '[entrypoint]' })) || []}
                    placeholder="Select function"
                    value={func?.functionAddress.toString() || ''}
                    onChange={(value) =>
                        router.push(createNavLink(version, module?.moduleId, value))
                    }
                    disabled={!functions || functions.length === 0}
                />
                <Button component={Link} href={`${backLink}`}>Back</Button>
                <ActionIcon color="blue" size={36} radius="xl" component={Link} href="/docs/gbf_core/">
                    <IconBook style={{ width: rem(16), height: rem(16) }} />
                </ActionIcon>
            </Stack>
        </Box>
    );
}