"use client";

import { useState, useEffect, useCallback } from 'react';
import { Box, Title, Paper, Button, Group, Text, Divider } from '@mantine/core';
import { IconArrowLeft, IconArrowRight } from '@tabler/icons-react';
import ZoomableGraphvizPanel from './zoomable-graphviz-panel';
import { Structure } from './tabbed-view';

interface StructureAnalysisStepperProps {
    structures: Structure[];
}

const StructureAnalysisStepper: React.FC<StructureAnalysisStepperProps> = ({ structures }) => {
    const [currentIndex, setCurrentIndex] = useState(0);

    const handlePrevious = useCallback(() => {
        setCurrentIndex((prev) => Math.max(0, prev - 1));
    }, []);

    const handleNext = useCallback(() => {
        setCurrentIndex((prev) => Math.min(structures.length - 1, prev + 1));
    }, [structures.length]);

    const handleKeyDown = useCallback((event: KeyboardEvent) => {
        if (event.key === 'ArrowLeft') handlePrevious();
        if (event.key === 'ArrowRight') handleNext();
    }, [handlePrevious, handleNext]);

    useEffect(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, [handleKeyDown]);

    if (structures.length === 0) {
        return (
            <Box mt="sm">
                <Title order={2}>Structure Analysis</Title>
                <Divider my="sm" />
                <Paper withBorder shadow="sm" p="md">
                    <Text>No structures available.</Text>
                </Paper>
            </Box>
        );
    }

    const currentStructure = structures[currentIndex];

    return (
        <Box mt="sm">
            <Paper withBorder shadow="sm" p="md">
                <Group align="apart" mb="sm">
                    <Text>
                        Step {currentIndex + 1} of {structures.length}
                    </Text>
                    <Group>
                        <Button
                            onClick={handlePrevious}
                            disabled={currentIndex === 0}
                            variant="default"
                        >
                            <IconArrowLeft size={16} style={{ marginRight: 8 }} />
                            Previous
                        </Button>
                        <Button
                            onClick={handleNext}
                            disabled={currentIndex === structures.length - 1}
                            variant="default"
                        >
                            Next
                            <IconArrowRight size={16} style={{ marginLeft: 8 }} />
                        </Button>
                    </Group>
                </Group>
                <ZoomableGraphvizPanel containerStyle={{ height: '500px' }} dotUrl={currentStructure.dotUrl} />
            </Paper>
        </Box>
    );
};

export default StructureAnalysisStepper;
