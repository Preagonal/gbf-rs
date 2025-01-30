import { Alert, Box } from '@mantine/core';
import { fetchAllGraphvizStructureAnalysis } from '@/lib/dynamodb/function-graphviz-structure-analysis-repo';
import TabbedFunctionView from '@/components/tabbed-view';
import { getFunction } from '@/lib/function';
import { IconAlertCircle } from '@tabler/icons-react';
import { getFunctionError } from '@/lib/error';
import { GbfFunctionDao } from '@/dao/gbf-function-dao';
import { GbfFunctionErrorDao } from '@/dao/gbf-function-error-dao';
import { GbfGraphvizStructureAnalysisDao } from '@/dao/gbf-graphviz-structure-analysis-dao';

type versionPromise = Promise<{ version: string; module: string; function: string }>;

export default async function Function(props: { params: versionPromise }) {
    const versionParam = (await props.params).version;
    const moduleParam = (await props.params).module;
    const functionParam = (await props.params).function;
    const functionParamNumber = parseInt(functionParam, 10);

    if (isNaN(functionParamNumber)) {
        return (
            <Box mt="sm">
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="Function Address Invalid"
                    color="red"
                    mt="sm"
                >
                    Function address must be a number.
                </Alert>
            </Box>
        )
    }

    const func = await getFunction(versionParam, moduleParam, functionParamNumber);
    const structures = await fetchAllGraphvizStructureAnalysis(versionParam, moduleParam, functionParamNumber);

    if (!func) {
        return (
            <Box mt="sm">
                <Alert
                    icon={<IconAlertCircle size="1rem" />}
                    title="Function Not Found"
                    color="red"
                    mt="sm"
                >
                    The function with address {functionParam} was not found in this version.
                </Alert>
            </Box>
        );
    }

    const error = await getFunctionError(versionParam, moduleParam, functionParamNumber);
    return (
        <Box mt="sm">
            <TabbedFunctionView err={error ? GbfFunctionErrorDao.toPlainObject(error) : null} func={GbfFunctionDao.toPlainObject(func)} structures={structures.map((st) => GbfGraphvizStructureAnalysisDao.toPlainObject(st))} />
        </Box>
    );
}