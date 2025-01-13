import { createQueryBuilder } from '@/data/data-analyzer';
import ZoomablePanableSVG from '@/components/zoomable-panel';
import { DecompilerError, FunctionId, FunctionResult, ModuleId, ModuleResult } from '@/data/gbf-suite-result-dao';
import { CodeHighlight } from '@mantine/code-highlight';
import { Box, Container, Paper, Text, Title, } from '@mantine/core';
import { S3_BUCKET_URL_BASE } from '@/consts';
import { NavigationBar } from '@/components/nav';

type versionPromise = Promise<{ version: string, module: string, function: string }>;


export default async function Versions(props: { params: versionPromise }) {
    const queryBuilder = await createQueryBuilder();
    const versionParam = (await props.params).version;
    const moduleParam = (await props.params).module;
    const functionParam: number = parseInt((await props.params).function, 10);

    const versions = queryBuilder.execute() as string[];
    const modules = queryBuilder.version(versionParam).execute() as ModuleResult[];
    const currentModule = modules.find((module: ModuleResult) => module.moduleId.moduleId === moduleParam);

    if (!currentModule) {
        throw new Error(`Module ${moduleParam} not found in version ${versionParam}`);
    }

    const modulesNav: ModuleId[] = modules.map((module: ModuleResult) => module.moduleId);

    const functions = queryBuilder.version(versionParam).module(moduleParam).execute() as FunctionResult[];
    const functionsNav: FunctionId[] = functions.map((func: FunctionResult) => func.functionId);

    const funcObj = queryBuilder.version(versionParam).module(moduleParam).function(functionParam).execute() as FunctionResult;

    let decompilerOutput = null;
    if ((funcObj.decompiled as DecompilerError).obj) {
        const errorAsJson = JSON.stringify((funcObj.decompiled as DecompilerError).obj, null, 2);
        decompilerOutput = (
            <Box mt="sm">
                <Title order={2}>Decompiler Error</Title>
                <Text mt="sm">
                    The decompiler failed to decompile this function in this version.
                </Text>
                <CodeHighlight mt="sm" language="json" code={errorAsJson} />
            </Box>
        );
    } else {
        decompilerOutput = (
            <Box mt="sm">
                <Title order={2}>Decompiler Output</Title>
                <CodeHighlight mt="sm" language="js" code={(funcObj.decompiled as string)} />
            </Box>
        );
    }

    return (
        <>
            <NavigationBar versions={versions} modules={modulesNav} functions={functionsNav} version={versionParam} module={currentModule?.moduleId} func={funcObj.functionId} />

            <Container size="md">
                {decompilerOutput}
                <Box mt="sm">
                    <Title order={2}>CFG</Title>
                    <Text mt="sm">Use the mouse wheel + shift to zoom in and out, and click and drag to pan.</Text>
                    <Paper mt="sm" withBorder>
                        <ZoomablePanableSVG containerStyle={{ height: '1000px' }} svgUrl={`${S3_BUCKET_URL_BASE}/${funcObj.svgCfg.Key}`} />
                    </Paper >
                </Box>
            </Container>
        </>
    );
}
