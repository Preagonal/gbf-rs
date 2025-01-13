"use client";

import React, { useRef, useEffect, useState } from "react";
import gsap from "gsap";
import { Draggable } from "gsap/Draggable";
import Image from "next/image";
gsap.registerPlugin(Draggable);

const SCROLL_RESISTANCE = 15;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 1;

interface ZoomableDraggableSVGProps {
    /** The URL of your .svg file. */
    svgUrl: string;
    /** Optional: if the parent wants to pass inline styles for the container (width/height, etc.) */
    containerStyle?: React.CSSProperties;
}

const ZoomableDraggableSVG: React.FC<ZoomableDraggableSVGProps> = ({
    svgUrl,
    containerStyle = {}, // parent can override
}) => {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const imageWrapperRef = useRef<HTMLDivElement | null>(null);
    const draggable = useRef<Draggable | null>(null);

    const [naturalSize, setNaturalSize] = useState({ width: 1, height: 1 });

    // We'll keep the currentZoom in state so the component re-renders
    // whenever zoom changes (and physically resizes the image).
    const [zoom, setZoom] = useState(1);

    useEffect(() => {
        if (!containerRef.current || !imageWrapperRef.current) return;

        // Create the Draggable instance on our image wrapper <div>
        draggable.current = Draggable.create(imageWrapperRef.current, {
            type: "x,y",
            bounds: containerRef.current, // keep child within container
        })[0];

        // Cleanup
        return () => {
            draggable.current?.kill();
        };
    }, []);

    // Store the natural SVG dimensions once loaded.
    const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement>) => {
        const img = e.currentTarget;
        setNaturalSize({
            width: img.naturalWidth,
            height: img.naturalHeight,
        });
    };

    /** Handle wheel to zoom in/out, keeping the mouse position "pinned." */
    const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
        // ensure shift key is pressed
        if (!e.shiftKey) return;
        e.preventDefault();

        // 1) Figure out the new zoom factor
        const oldZoom = zoom;
        const direction = Math.sign(e.deltaY); // +1 or -1
        let newZoom = oldZoom - direction / SCROLL_RESISTANCE;
        newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newZoom));

        // 2) If the zoom didn't actually change, no need to proceed
        if (newZoom === oldZoom) return;

        // 3) Grab the container's bounding box
        const containerRect = containerRef.current?.getBoundingClientRect();
        if (!containerRect) return;

        // Mouse position within the container
        const pointerInContainerX = e.clientX - containerRect.left;
        const pointerInContainerY = e.clientY - containerRect.top;

        // 4) Get Draggable's current offset (top-left corner of image)
        const currentDraggable = draggable.current;
        if (!currentDraggable) return;
        const oldOffsetX = currentDraggable.x || 0;
        const oldOffsetY = currentDraggable.y || 0;

        // 5) Figure out the mouse position within the image’s coordinates
        const pointerInImageX = pointerInContainerX - oldOffsetX;
        const pointerInImageY = pointerInContainerY - oldOffsetY;

        // 6) Compute ratio of newZoom to oldZoom
        const ratio = newZoom / oldZoom;

        // 7) The new pointer coordinates in the image’s new scale
        const newPointerInImageX = pointerInImageX * ratio;
        const newPointerInImageY = pointerInImageY * ratio;

        // 8) We want the pointer to stay in the same container location,
        //    so we solve for the new top-left offset:
        //    pointerInContainerX - newOffsetX = newPointerInImageX
        //    => newOffsetX = pointerInContainerX - newPointerInImageX
        const newOffsetX = pointerInContainerX - newPointerInImageX;
        const newOffsetY = pointerInContainerY - newPointerInImageY;

        // 9) Update Draggable’s x,y so the image shifts around that point
        gsap.set(currentDraggable.target, {
            x: newOffsetX,
            y: newOffsetY,
        });

        // 10) Force Draggable to update internal data
        currentDraggable.update();

        // 11) Finally, update our React state for the new zoom (which triggers a re-render)
        setZoom(newZoom);
    };

    // Whenever zoom changes, we need to re-apply bounds in case the
    // image is now bigger or smaller than before.
    useEffect(() => {
        if (!draggable.current) return;
        draggable.current.applyBounds(containerRef.current);
        draggable.current.update();
    }, [zoom]);

    // Physically compute the new size of the image.
    const scaledWidth = naturalSize.width * zoom;
    const scaledHeight = naturalSize.height * zoom;

    return (
        <div
            ref={containerRef}
            onWheel={handleWheel}
            style={{
                position: "relative",
                overflow: "hidden",
                width: "100%",
                // The parent can control the final height or pass via containerStyle
                ...containerStyle,
            }}
        >
            {/* Absolutely-positioned child, so we can drag it around. */}
            <div
                ref={imageWrapperRef}
                style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: scaledWidth,
                    height: scaledHeight,
                    cursor: "grab",
                }}
            >
                {/* The <img> is physically resized to fill the wrapper's new scaledWidth/Height. */}
                <Image
                    src={svgUrl}
                    alt="Zoomable draggable"
                    onLoad={handleImageLoad}
                    width={scaledWidth}
                    height={scaledHeight}
                />
            </div>
        </div>
    );
};

export default ZoomableDraggableSVG;
