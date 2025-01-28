"use client";

import React, { useEffect, useState } from 'react';
import { instance } from "@viz-js/viz";
import { LoadingOverlay, Paper } from '@mantine/core';
import ZoomableDraggableSVG from './zoomable-panel';

interface ZoomableGraphvizPanelProps {
    /** The DOT string to be rendered. */
    dotUrl: string;
    /** Optional: if the parent wants to pass inline styles for the container (width/height, etc.) */
    containerStyle?: React.CSSProperties;
}

const ZoomableGraphvizPanel: React.FC<ZoomableGraphvizPanelProps> = ({ dotUrl, containerStyle }) => {
    const [svgElement, setSvgElement] = useState<SVGSVGElement | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const renderSvg = async () => {
            try {
                const dotStr = await (await fetch(dotUrl)).text();
                const viz = await instance();
                const svgElement = await viz.renderSVGElement(dotStr);
                setSvgElement(svgElement);
            } catch (err) {
                setError(`Error rendering graph: ${(err as Error).message}`);
            }
        };

        renderSvg();
    }, [dotUrl]);

    return (
        <Paper style={{ overflow: 'auto', ...containerStyle }}>
            {!svgElement ? <LoadingOverlay visible /> : <ZoomableDraggableSVG svg={svgElement} containerStyle={containerStyle} />}
            {error && <div style={{ color: 'red' }}>{error}</div>}
        </Paper>
    );
};

export default ZoomableGraphvizPanel;