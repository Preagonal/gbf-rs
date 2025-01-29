"use client";
import '@mantine/code-highlight/styles.css';
import React from "react";
import { Box, Tabs, Title, Text, Paper, Alert } from "@mantine/core";
import { IconAlertCircle } from "@tabler/icons-react";
import { CodeHighlight } from "@mantine/code-highlight";

import StructureAnalysisStepper from "./structure-analysis-stepper";
import ZoomableGraphvizPanel from "./zoomable-graphviz-panel";
import { GbfSimplifiedBacktrace } from "@/dao/gbf-function-error-dao";

interface FunctionInterface {
    decompileResult?: string;
    decompileSuccess: boolean;
    dotUrl: string;
}
interface FunctionError {
    errorType: string;
    message: string;
    backtrace: GbfSimplifiedBacktrace;
    errorContext: string;
}

export interface Structure {
    gbfVersion: string;
    moduleId: string;
    functionAddress: number;
    structureAnalysisStep: number;
    dotUrl: string;
}

interface TabbedFunctionViewProps {
    structures: Structure[];
    func: FunctionInterface;
    err: FunctionError | null;
}

const TabbedFunctionView: React.FC<TabbedFunctionViewProps> = ({
    structures,
    func,
    err,
}) => {
    return (
        <Tabs defaultValue="decompiler-output">
            <Tabs.List grow justify="space-between">
                <Tabs.Tab value="decompiler-output">Decompiler Output</Tabs.Tab>
                <Tabs.Tab value="cfg">BasicBlock CFG</Tabs.Tab>
                <Tabs.Tab value="structure-analysis">Structure Analysis</Tabs.Tab>
            </Tabs.List>

            {/* Decompiler Output Tab */}
            <Tabs.Panel value="decompiler-output" pt="md">
                <Box mt="sm">
                    <Title order={2}>Decompiler Output</Title>

                    {/* If there's an error object, display a red alert + backtrace */}
                    {err && (
                        <>
                            <Alert
                                icon={<IconAlertCircle size={16} />}
                                title={`Error: ${err.errorType}`}
                                color="red"
                                mt="sm"
                            >
                                <Text>{err.message}</Text>
                            </Alert>
                            <Title order={3} mt="sm">Backtrace</Title>
                            <CodeHighlight
                                mt="sm"
                                language="json"
                                code={JSON.stringify(err.backtrace, null, 2)}
                            />
                            <Title order={3} mt="sm">Context</Title>
                            <CodeHighlight
                                mt="sm"
                                language="json"
                                code={err.errorContext}
                            />
                        </>
                    )}

                    {/* If function was successfully decompiled, show the output;
              Otherwise, show an error message (in addition to the alert if present). */}
                    {func.decompileSuccess ? (
                        <CodeHighlight
                            mt="sm"
                            language="js"
                            code={func.decompileResult || ""}
                        />
                    ) : (
                        <></>
                    )}
                </Box>
            </Tabs.Panel>

            {/* BasicBlock CFG Tab */}
            <Tabs.Panel value="cfg" pt="md">
                <Box mt="sm">
                    <Title order={2}>BasicBlock CFG</Title>
                    <Text mt="sm">
                        Use the mouse wheel + shift to zoom in and out, and click and drag to
                        pan.
                    </Text>
                    <Paper mt="sm" withBorder>
                        <ZoomableGraphvizPanel
                            containerStyle={{ height: "1000px" }}
                            dotUrl={func.dotUrl}
                        />
                    </Paper>
                </Box>
            </Tabs.Panel>

            {/* Structure Analysis Tab */}
            <Tabs.Panel value="structure-analysis" pt="md">
                <Box mt="sm">
                    <Title order={2}>Structure Analysis</Title>
                    <Text mt="sm">
                        Use the mouse wheel + shift to zoom in and out, and click and drag to
                        pan.
                    </Text>
                    <StructureAnalysisStepper structures={structures} />
                </Box>
            </Tabs.Panel>
        </Tabs>
    );
};

export default TabbedFunctionView;
