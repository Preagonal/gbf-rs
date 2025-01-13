import { NavigationBar } from '@/components/nav';
import { createQueryBuilder } from '@/data/data-analyzer';
import { FunctionId, FunctionResult, ModuleId, ModuleResult } from '@/data/gbf-suite-result-dao';
import { Container, List, ListItem, ThemeIcon, rem, Anchor } from '@mantine/core';
import { IconCircleCheck, IconExclamationCircle } from '@tabler/icons-react';
import Link from 'next/link';

type versionPromise = Promise<{ version: string, module: string }>;


export default async function FunctionsForModule(props: { params: versionPromise }) {
    const queryBuilder = await createQueryBuilder();

    const versions = queryBuilder.execute() as string[];
    const versionParam = (await props.params).version;

    const moduleParam = (await props.params).module;
    const modules = queryBuilder.version(versionParam).execute() as ModuleResult[];
    const currentModule = modules.find((module: ModuleResult) => module.moduleId.moduleId === moduleParam);

    if (!currentModule) {
        throw new Error(`Module ${moduleParam} not found in version ${versionParam}`);
    }

    const modulesNav: ModuleId[] = modules.map((module: ModuleResult) => module.moduleId);

    const functions = queryBuilder.version(versionParam).module(moduleParam).execute() as FunctionResult[];
    const functionsNav: FunctionId[] = functions.map((func: FunctionResult) => func.functionId);

    return (
        <>
            <NavigationBar versions={versions} modules={modulesNav} functions={functionsNav} version={versionParam} module={currentModule?.moduleId} func={null} />
            <Container size="md">
                <List mt="sm" spacing="xs" withPadding>
                    {functions && functions.map((func: FunctionResult) => (
                        <ListItem
                            key={func.functionId.functionAddress}
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
                            <Anchor component={Link} href={`/${versionParam}/${currentModule?.moduleId.moduleId}/${func.functionId.functionAddress}`}>
                                {func.functionId.functionName || "[entrypoint]"}
                            </Anchor>
                        </ListItem>
                    ))}
                </List>
            </Container>
        </>
    );
}
