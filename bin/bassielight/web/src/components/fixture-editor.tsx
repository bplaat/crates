/*
 * Copyright (c) 2026 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState } from 'preact/hooks';
import { LightbulbIcon } from './icons.tsx';
import { FixtureMap, FixtureItem } from './fixture-map.tsx';

// From rust config.rs
const FIXTURE_TYPES = [
    { value: 'american_dj_p56led', label: 'American DJ P56 LED' },
    { value: 'american_dj_mega_tripar', label: 'American DJ Mega Tripar' },
    { value: 'ayra_compar_10', label: 'Ayra ComPar 10' },
    { value: 'ayra_compar_20', label: 'Ayra ComPar 20' },
    { value: 'showtec_multidim_mkii', label: 'Showtec MultiDim MKII' }
];

export interface ExtendedFixtureItem extends FixtureItem {
    addr?: number;
    name?: string;
}

interface FixtureEditorProps {
    items: ExtendedFixtureItem[];
    onUpdate: (items: ExtendedFixtureItem[]) => void;
}

export function FixtureEditor({ items, onUpdate }: FixtureEditorProps) {
    const [selectedId, setSelectedId] = useState<string | null>(null);

    const handleMapUpdate = (newItems: FixtureItem[]) => {
        onUpdate(newItems as ExtendedFixtureItem[]);
    };

    const addFixture = () => {
        const id = (Math.max(0, ...items.map(i => parseInt(i.id) || 0)) + 1).toString();

        const lastItem = items[items.length - 1];
        const nextAddr = lastItem ? (lastItem.addr || 1) + 10 : 1;

        const newItem: ExtendedFixtureItem = {
            id,
            type: 'american_dj_p56led',
            x: 50,
            y: 50,
            label: `Fixture ${id}`,
            name: `Fixture ${id}`,
            addr: nextAddr
        };
        onUpdate([...items, newItem]);
        setSelectedId(id);
    };

    const selectedItem = items.find(i => i.id === selectedId);

    const updateSelectedItem = (updates: Partial<ExtendedFixtureItem>) => {
        if (!selectedId) return;
        const updatedItems = items.map(i => {
            if (i.id === selectedId) {
                const updated = { ...i, ...updates };
                // Keep label and name in sync for now
                if (updates.label) updated.name = updates.label;
                if (updates.name) updated.label = updates.name;
                return updated;
            }
            return i;
        });
        onUpdate(updatedItems);
    };

    const deleteSelectedItem = () => {
        if (!selectedId) return;
        onUpdate(items.filter(i => i.id !== selectedId));
        setSelectedId(null);
    };

    return (
        <>
            <div class="main">
                <FixtureMap
                    items={items}
                    onUpdate={handleMapUpdate}
                    selectedId={selectedId}
                    onSelect={setSelectedId}
                />

                <div class="buttons on-bottom is-centered">
                    <button class="button is-text is-large" onClick={addFixture}>
                        <LightbulbIcon />
                        Add fixture
                    </button>
                </div>
            </div>

            <div class="sidebar">
                <div class="subtitle">Properties</div>

                {selectedItem ? (
                    <div style={{ padding: '0.5rem' }}>
                        <div style={{ marginBottom: '1rem' }}>
                            <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: '#aaa' }}>Name</label>
                            <input
                                type="text"
                                value={selectedItem.label || ''}
                                onInput={(e) => updateSelectedItem({ label: (e.target as HTMLInputElement).value })}
                                style={{
                                    width: '100%',
                                    padding: '0.5rem',
                                    backgroundColor: '#333',
                                    border: '1px solid #444',
                                    color: '#fff',
                                    borderRadius: '4px',
                                    outline: 'none'
                                }}
                            />
                        </div>

                        <div style={{ marginBottom: '1rem' }}>
                            <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: '#aaa' }}>Type</label>
                            <select
                                value={selectedItem.type}
                                onChange={(e) => updateSelectedItem({ type: (e.target as HTMLSelectElement).value })}
                                style={{
                                    width: '100%',
                                    padding: '0.5rem',
                                    backgroundColor: '#333',
                                    border: '1px solid #444',
                                    color: '#fff',
                                    borderRadius: '4px',
                                    outline: 'none'
                                }}
                            >
                                {FIXTURE_TYPES.map(t => (
                                    <option key={t.value} value={t.value}>{t.label}</option>
                                ))}
                            </select>
                        </div>

                        <div style={{ marginBottom: '1rem' }}>
                            <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: '#aaa' }}>Address (DMX)</label>
                            <input
                                type="number"
                                min="1"
                                max="512"
                                value={selectedItem.addr || 1}
                                onInput={(e) => updateSelectedItem({ addr: parseInt((e.target as HTMLInputElement).value) || 1 })}
                                style={{
                                    width: '100%',
                                    padding: '0.5rem',
                                    backgroundColor: '#333',
                                    border: '1px solid #444',
                                    color: '#fff',
                                    borderRadius: '4px',
                                    outline: 'none'
                                }}
                            />
                        </div>

                        <div style={{ marginBottom: '1rem', fontSize: '0.8rem', color: '#666' }}>
                            ID: {selectedItem.id}<br/>
                            X: {selectedItem.x}, Y: {selectedItem.y}
                        </div>

                        <div style={{ marginTop: '2rem', borderTop: '1px solid #333', paddingTop: '1rem' }}>
                            <button
                                onClick={deleteSelectedItem}
                                style={{
                                    width: '100%',
                                    padding: '0.75rem',
                                    backgroundColor: '#dc3545',
                                    border: 'none',
                                    borderRadius: '4px',
                                    color: '#fff',
                                    fontWeight: 'bold',
                                    cursor: 'pointer',
                                    transition: 'background-color 0.2s'
                                }}
                                onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#bb2d3b')}
                                onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = '#dc3545')}
                            >
                                Delete Fixture
                            </button>
                        </div>
                    </div>
                ) : (
                    <div style={{ padding: '0.5rem', color: '#888' }}>
                        <p>Select a fixture to edit properties</p>
                        <p>Total: {items.length}</p>
                    </div>
                )}
            </div>
        </>
    );
}
