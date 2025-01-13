import { NavigationBar } from '@/components/nav';
import { createQueryBuilder } from '@/data/data-analyzer';
import { ModuleId, ModuleResult } from '@/data/gbf-suite-result-dao';
import { Container, List, ListItem, ThemeIcon, rem, Anchor } from '@mantine/core';
import { IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

type versionPromise = Promise<{ version: string }>;


export default async function ModulesForVersion(props: { params: versionPromise }) {
    const queryBuilder = await createQueryBuilder();
    const versions = await queryBuilder.execute() as string[];
    const paramVersion = (await props.params).version;
    const modules = (await queryBuilder.version(paramVersion).execute()) as ModuleResult[];

    const modulesNav: ModuleId[] = modules.map((module: ModuleResult) => module.moduleId);
    return (
        <>
            <NavigationBar versions={versions} modules={modulesNav} functions={[]} version={paramVersion} module={null} func={null} />
            <Container size="md">
                <List mt="sm" icon={
                    <ThemeIcon color="yellow" size={24} radius="xl">
                        <IconExclamationCircle style={{ width: rem(16), height: rem(16) }} />
                    </ThemeIcon>
                } withPadding>
                    {modules && modules.map((module: ModuleResult) => (
                        <ListItem key={module.moduleId.moduleId}>
                            <Anchor component={Link} href={`/${paramVersion}/${module.moduleId.moduleId}`}>{module.moduleId.fileName}</Anchor>
                        </ListItem>
                    ))}
                </List>
            </Container >
        </>

    );
}
