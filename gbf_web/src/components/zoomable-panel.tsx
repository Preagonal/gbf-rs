"use client";

import React, {
    useRef,
    useEffect,
    useState,
    useCallback,
} from "react";
import gsap from "gsap";
import { Draggable } from "gsap/Draggable";
gsap.registerPlugin(Draggable);

const SCROLL_RESISTANCE = 15;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 1;

interface ZoomableDraggableSVGProps {
    /** A pre-loaded SVG element (e.g., from Graphviz), never changes once passed. */
    svg: SVGSVGElement;
    /** Optional inline style for the container (width, height, etc.) */
    containerStyle?: React.CSSProperties;
}

const ZoomableDraggableSVG: React.FC<ZoomableDraggableSVGProps> = ({
    svg,
    containerStyle = {},
}) => {
    // State for the parsed width/height
    const [svgSize, setSvgSize] = useState({ width: 0, height: 0 });

    // Refs
    const containerRef = useRef<HTMLDivElement | null>(null);
    const imageWrapperRef = useRef<HTMLDivElement | null>(null);

    // Store the container's measured size in refs
    const containerWidthRef = useRef<number>(0);
    const containerHeightRef = useRef<number>(0);

    // Draggable ref
    const draggableRef = useRef<Draggable | null>(null);

    // Transform/pan/zoom
    const xRef = useRef(0);
    const yRef = useRef(0);
    const zoomRef = useRef(1);

    // Pinch tracking
    const pinchRef = useRef<{
        initialDistance: number;
        initialZoom: number;
    } | null>(null);

    // ------------------------------------------------------------------
    // 1) Parse the <svg> size from width/height or viewBox (once)
    // ------------------------------------------------------------------
    useEffect(() => {
        if (!svg) return;

        let width = svg.width?.baseVal?.value ?? 0;
        let height = svg.height?.baseVal?.value ?? 0;

        // If no explicit width/height, fallback to viewBox
        if ((!width || !height) && svg.hasAttribute("viewBox")) {
            const vb = svg.getAttribute("viewBox");
            if (vb) {
                const parts = vb.split(/[\s,]+/);
                if (parts.length === 4) {
                    const vbW = parseFloat(parts[2]) || 0;
                    const vbH = parseFloat(parts[3]) || 0;
                    if (vbW && vbH) {
                        width = vbW;
                        height = vbH;
                    }
                }
            }
        }

        // Fallback
        if (!width || !height) {
            width = 800;
            height = 600;
        }

        setSvgSize({ width, height });
    }, [svg]);

    // ------------------------------------------------------------------
    // 2) Observe container size with ResizeObserver
    // ------------------------------------------------------------------
    useEffect(() => {
        if (!containerRef.current) return;

        const ro = new ResizeObserver((entries) => {
            for (const entry of entries) {
                if (entry.target === containerRef.current && entry.contentRect) {
                    containerWidthRef.current = entry.contentRect.width;
                    containerHeightRef.current = entry.contentRect.height;
                }
            }
        });

        ro.observe(containerRef.current);

        // Initial measure
        const rect = containerRef.current.getBoundingClientRect();
        containerWidthRef.current = rect.width;
        containerHeightRef.current = rect.height;

        return () => {
            ro.disconnect();
        };
    }, []);

    // ------------------------------------------------------------------
    // 3) clampToBounds
    // ------------------------------------------------------------------
    const clampToBounds = useCallback(() => {
        const containerW = containerWidthRef.current;
        const containerH = containerHeightRef.current;
        const scaledW = svgSize.width * zoomRef.current;
        const scaledH = svgSize.height * zoomRef.current;

        // If bigger, typical bounding.
        // If smaller, allow from 0 to leftover space.
        let minX: number;
        let maxX: number;
        if (scaledW > containerW) {
            minX = containerW - scaledW;
            maxX = 0;
        } else {
            minX = 0;
            maxX = containerW - scaledW;
        }
        xRef.current = clamp(xRef.current, minX, maxX);

        let minY: number;
        let maxY: number;
        if (scaledH > containerH) {
            minY = containerH - scaledH;
            maxY = 0;
        } else {
            minY = 0;
            maxY = containerH - scaledH;
        }
        yRef.current = clamp(yRef.current, minY, maxY);
    }, [svgSize.width, svgSize.height]);

    // ------------------------------------------------------------------
    // 4) applyTransform
    // ------------------------------------------------------------------
    function applyTransform() {
        if (!imageWrapperRef.current) return;
        gsap.set(imageWrapperRef.current, {
            x: xRef.current,
            y: yRef.current,
            scale: zoomRef.current,
            transformOrigin: "0 0",
            force3D: true,
        });
    }

    // ------------------------------------------------------------------
    // 5) Create (or re-create) Draggable AFTER we know sizes
    // ------------------------------------------------------------------
    useEffect(() => {
        // If we haven't measured container or parsed svg size, skip Draggable creation for now.
        if (!imageWrapperRef.current) return;
        if (svgSize.width === 0 || svgSize.height === 0) return;
        if (containerWidthRef.current === 0 || containerHeightRef.current === 0) return;

        // Kill old Draggable
        draggableRef.current?.kill();

        // Create new Draggable
        draggableRef.current = Draggable.create(imageWrapperRef.current, {
            type: "x,y",
            allowEventDefault: false,
            dragClickables: false,
            onDrag() {
                xRef.current = this.x;
                yRef.current = this.y;
                clampToBounds();
                applyTransform();
            },
        })[0];

        // Optionally clamp once, in case x,y is out of bounds
        clampToBounds();
        applyTransform();

        return () => {
            draggableRef.current?.kill();
            draggableRef.current = null;
        };
    }, [
        svgSize.width,
        svgSize.height,
        clampToBounds,
    ]);

    // ------------------------------------------------------------------
    // 6) Wheel Zoom (Desktop)
    // ------------------------------------------------------------------
    const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
        if (!e.shiftKey) return;

        const oldZoom = zoomRef.current;
        const direction = Math.sign(e.deltaY);
        let newZoom = oldZoom - direction / SCROLL_RESISTANCE;
        newZoom = clamp(newZoom, MIN_ZOOM, MAX_ZOOM);
        if (newZoom === oldZoom) return;

        const containerRect = containerRef.current?.getBoundingClientRect();
        if (!containerRect) return;

        const pointerX = e.clientX - containerRect.left;
        const pointerY = e.clientY - containerRect.top;

        const pointerInImageX = pointerX - xRef.current;
        const pointerInImageY = pointerY - yRef.current;
        const ratio = newZoom / oldZoom;

        const newPointerInImageX = pointerInImageX * ratio;
        const newPointerInImageY = pointerInImageY * ratio;

        const newX = pointerX - newPointerInImageX;
        const newY = pointerY - newPointerInImageY;

        zoomRef.current = newZoom;
        xRef.current = newX;
        yRef.current = newY;

        clampToBounds();
        applyTransform();
    };

    // ------------------------------------------------------------------
    // 7) Touch Pinch Zoom
    // ------------------------------------------------------------------
    const handleTouchStart = (e: React.TouchEvent<HTMLDivElement>) => {
        if (e.touches.length === 2) {
            e.preventDefault();
            const distance = getDistance(e.touches[0], e.touches[1]);
            pinchRef.current = {
                initialDistance: distance,
                initialZoom: zoomRef.current,
            };
        }
    };

    const handleTouchMove = (e: React.TouchEvent<HTMLDivElement>) => {
        if (e.touches.length === 2 && pinchRef.current) {
            e.preventDefault();
            const containerRect = containerRef.current?.getBoundingClientRect();
            if (!containerRect) return;

            const t1 = e.touches[0];
            const t2 = e.touches[1];

            const midpointNow = {
                x: (t1.clientX + t2.clientX) / 2 - containerRect.left,
                y: (t1.clientY + t2.clientY) / 2 - containerRect.top,
            };

            const newDistance = getDistance(t1, t2);
            const { initialDistance, initialZoom } = pinchRef.current;

            let newZoom = initialZoom * (newDistance / initialDistance);
            newZoom = clamp(newZoom, MIN_ZOOM, MAX_ZOOM);

            const oldZoom = zoomRef.current;
            const ratio = newZoom / oldZoom;

            const pointerInImageX = midpointNow.x - xRef.current;
            const pointerInImageY = midpointNow.y - yRef.current;

            const newPointerInImageX = pointerInImageX * ratio;
            const newPointerInImageY = pointerInImageY * ratio;

            const newX = midpointNow.x - newPointerInImageX;
            const newY = midpointNow.y - newPointerInImageY;

            zoomRef.current = newZoom;
            xRef.current = newX;
            yRef.current = newY;

            clampToBounds();
            applyTransform();
        }
    };

    const handleTouchEnd = (e: React.TouchEvent<HTMLDivElement>) => {
        if (e.touches.length < 2) {
            pinchRef.current = null;
        }
    };

    // ------------------------------------------------------------------
    // 8) Utility
    // ------------------------------------------------------------------
    function clamp(value: number, minVal: number, maxVal: number) {
        return Math.max(minVal, Math.min(maxVal, value));
    }

    function getDistance(t1: React.Touch, t2: React.Touch) {
        const dx = t2.clientX - t1.clientX;
        const dy = t2.clientY - t1.clientY;
        return Math.sqrt(dx * dx + dy * dy);
    }

    // ------------------------------------------------------------------
    // 9) Render
    // ------------------------------------------------------------------
    return (
        <div
            ref={containerRef}
            style={{
                position: "relative",
                overflow: "hidden",
                width: "100%",
                height: "100%", // or some min-height
                touchAction: "none",
                ...containerStyle,
            }}
            onWheel={handleWheel}
            onTouchStart={handleTouchStart}
            onTouchMove={handleTouchMove}
            onTouchEnd={handleTouchEnd}
        >
            <div
                ref={imageWrapperRef}
                style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    cursor: "grab",
                    userSelect: "none",
                    willChange: "transform",
                }}
            >
                {svgSize.width === 0 || svgSize.height === 0 ? (
                    <div>Loading SVG…</div>
                ) : (
                    <svg
                        viewBox={
                            svg.getAttribute("viewBox") ||
                            `0 0 ${svgSize.width} ${svgSize.height}`
                        }
                        width={svgSize.width}
                        height={svgSize.height}
                        dangerouslySetInnerHTML={{ __html: svg.innerHTML }}
                    />
                )}
            </div>
        </div>
    );
};

export default ZoomableDraggableSVG;
