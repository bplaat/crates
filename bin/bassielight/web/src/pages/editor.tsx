/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { IpcContext } from '../app.tsx';
import { FixtureEditor, ExtendedFixtureItem } from '../components/fixture-editor.tsx';

export function EditorPage() {
    const ipc = useContext(IpcContext);
    const [items, setItems] = useState<ExtendedFixtureItem[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        document.title = 'BassieLight - Editor';

        let isMounted = true;
        if (ipc) {
            const loadConfig = async () => {
                try {
                    const data: any = await ipc.request('getConfig');
                    
                    if (isMounted && data && data.config && Array.isArray(data.config.fixtures)) {
                        const loadedItems = data.config.fixtures.map((f: any, index: number) => ({
                            id: index.toString(),
                            type: f.type,
                            x: f.x || 0,
                            y: f.y || 0,
                            label: f.name,
                            name: f.name,
                            addr: f.addr,
                        }));
                        setItems(loadedItems);
                    }
                } catch (e) {
                    console.warn('Could not load config:', e);
                } finally {
                    if (isMounted) setLoading(false);
                }
            };
            loadConfig();
        } else {
            setLoading(false);
        }

        return () => { isMounted = false; };
    }, [ipc]);

    const handleUpdate = (newItems: ExtendedFixtureItem[]) => {
        setItems(newItems);
    };

    if (loading) {
        return (
            <div class="main" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <p>Loading configuration...</p>
            </div>
        );
    }

    return <FixtureEditor items={items} onUpdate={handleUpdate} />;
}
