/*
 * Copyright (c) 2026 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useRef, useEffect } from 'preact/hooks';

export interface FixtureItem {
    id: string;
    type: string;
    x: number;
    y: number;
    color?: string;
    label?: string;
}

interface FixtureMapProps {
    items: FixtureItem[];
    selectedId?: string | null;
    onUpdate: (items: FixtureItem[]) => void;
    onSelect?: (id: string | null) => void;
    width?: number;
    height?: number;
}

export function FixtureMap({ items, selectedId, onUpdate, onSelect}: FixtureMapProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [draggingId, setDraggingId] = useState<string | null>(null);
    const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
    const [isDragging, setIsDragging] = useState(false);

    const handleMouseDown = (e: MouseEvent, item: FixtureItem) => {
        e.stopPropagation();
        setDraggingId(item.id);
        setIsDragging(false);

        if (containerRef.current) {
            setDragOffset({
                x: e.clientX - item.x,
                y: e.clientY - item.y
            });
        }
    };

    const handleContainerMouseDown = () => {
        if (onSelect) onSelect(null);
    };

    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (!draggingId || !containerRef.current) return;

            setIsDragging(true);

            // Calculate new X/Y
            let newX = e.clientX - dragOffset.x;
            let newY = e.clientY - dragOffset.y;

            // Optional: Snap to grid (e.g. 10px)
            const snap = 10;
            newX = Math.round(newX / snap) * snap;
            newY = Math.round(newY / snap) * snap;

            // Update item in list
            const newItems = items.map(it => {
                if (it.id === draggingId) {
                    return { ...it, x: newX, y: newY };
                }
                return it;
            });
            onUpdate(newItems);
        };

        const handleMouseUp = () => {
            if (draggingId && !isDragging && onSelect) {
                onSelect(draggingId);
            }
            setDraggingId(null);
            setIsDragging(false);
        };

        if (draggingId) {
            window.addEventListener('mousemove', handleMouseMove);
            window.addEventListener('mouseup', handleMouseUp);
        }

        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [draggingId, isDragging, items, onUpdate, onSelect, dragOffset]);

    return (
        <div
            ref={containerRef}
            onMouseDown={handleContainerMouseDown}
            style={{
                width: `100%`,
                height: `100%`,
                backgroundColor: '#222',
                position: 'relative',
                overflow: 'hidden',
                boxShadow: 'inset 0 0 20px rgba(0,0,0,0.5)',
                backgroundImage: 'linear-gradient(#333 1px, transparent 1px), linear-gradient(90deg, #333 1px, transparent 1px)',
                backgroundSize: '20px 20px'
            }}
        >
            {items.map(item => (
                <div
                    key={item.id}
                    onMouseDown={(e) => handleMouseDown(e, item)}
                    style={{
                        position: 'absolute',
                        left: `${item.x}px`,
                        top: `${item.y}px`,
                        width: '40px',
                        height: '40px',
                        backgroundColor: item.color || '#007bff',
                        border: selectedId === item.id ? '2px solid #fff' : '2px solid rgba(255,255,255,0.5)',
                        outline: selectedId === item.id ? '2px solid #007bff' : 'none',
                        borderRadius: '50%',
                        cursor: draggingId === item.id ? 'grabbing' : 'grab',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: '#fff',
                        fontSize: '0.8rem',
                        fontWeight: 'bold',
                        userSelect: 'none',
                        zIndex: draggingId === item.id ? 100 : (selectedId === item.id ? 10 : 1),
                        boxShadow: draggingId === item.id ? '0 5px 15px rgba(0,0,0,0.5)' : 'none',
                        transition: draggingId === item.id ? 'none' : 'box-shadow 0.2s, border-color 0.2s'
                    }}
                >
                    {/* Visual indicator based on type? For now just label or icon */}
                    {item.label || 'L'}
                </div>
            ))}
        </div>
    );
}
