/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { IpcContext } from '../app.tsx';
import { CodeEditor, disposeCodeModels } from '../components/code-editor.tsx';
import { ContentSaveOutlineIcon, DeleteIcon, PlayIcon, PlusIcon, StopIcon } from '../components/icons.tsx';
import { Visualization } from '../components/visualization.tsx';
import { $document, $scripts, toggleScript, useDmxOutput } from '../stage.ts';
import './scripts.css';

const TEMPLATE = `-- Runs on the beat of the BPM button, see AGENTS.md in the stage folder for the full API
local lights = fixtures("rgb")
while true do
    lights:set({ color = "red" })
    wait(1)
    lights:set({ color = "blue" })
    wait(1)
end
`;

/// Same rule as the app, script names are file names
const isValidName = (name: string) => /^[A-Za-z0-9 _-]{1,64}$/.test(name);

export function ScriptsPage() {
    const ipc = useContext(IpcContext)!;
    const { scripts, running, errors } = $scripts.value;
    const names = Object.keys(scripts);
    const [selected, setSelected] = useState<string | null>(null);
    // Unsaved edits per script, scripts without edits follow their file, like when AI agents change it
    const [drafts, setDrafts] = useState<Record<string, string>>({});
    const [newName, setNewName] = useState('');
    // Scripts run while editing them and the stage folder keeps following changes
    const outputs = useDmxOutput(ipc, false);

    const name = selected !== null && selected in scripts ? selected : (names[0] ?? null);
    const isDirty = (script: string) => script in drafts && drafts[script] !== scripts[script];

    useEffect(() => {
        document.title = 'BassieLight - Scripts';
    }, []);
    useEffect(() => disposeCodeModels(names), [names.join('\n')]);

    const save = (script: string) => {
        if (!isDirty(script)) return;
        ipc.send('saveScript', { name: script, source: drafts[script] });
        const { [script]: _, ...rest } = drafts;
        setDrafts(rest);
    };
    const create = () => {
        if (!isValidName(newName) || newName in scripts) return;
        ipc.send('saveScript', { name: newName, source: TEMPLATE });
        setSelected(newName);
        setNewName('');
    };
    const remove = (script: string) => {
        if (!confirm(`Delete script ${script}?`)) return;
        ipc.send('deleteScript', { name: script });
        const { [script]: _, ...rest } = drafts;
        setDrafts(rest);
    };

    return (
        <>
            <div class="sidebar is-left">
                <h2 class="title">Scripts</h2>
                <div class="list">
                    {names.length === 0 && <p>No scripts yet</p>}
                    {names.map((script) => (
                        <div key={script} class={`script-item ${script === name ? 'is-selected' : ''}`}>
                            <button class="script-item-name" onClick={() => setSelected(script)}>
                                <span
                                    class={`script-dot ${running.includes(script) ? 'is-running' : ''} ${errors[script] ? 'is-error' : ''}`}
                                />
                                {script}
                                {isDirty(script) && <span class="script-dirty" title="Unsaved changes" />}
                            </button>
                            <button
                                class="icon-button"
                                title={running.includes(script) ? 'Stop' : 'Start'}
                                onClick={() => toggleScript(ipc, script)}
                            >
                                {running.includes(script) ? <StopIcon /> : <PlayIcon />}
                            </button>
                        </div>
                    ))}
                </div>
                <div class="script-new">
                    <input
                        class="input"
                        placeholder="New script name"
                        value={newName}
                        onInput={(e) => setNewName(e.currentTarget.value)}
                        onKeyDown={(e) => e.key === 'Enter' && create()}
                    />
                    <button class="icon-button" title="Add script" disabled={!isValidName(newName)} onClick={create}>
                        <PlusIcon />
                    </button>
                </div>

                <p class="block script-hint">
                    Timing is in beats of the BPM button. The stage folder has an AGENTS.md with the full API, so AI
                    agents like Claude Code and Codex can write scripts, which show up here right away.
                </p>
            </div>

            <div class="main">
                {name !== null ? (
                    <div class="script-editor">
                        <div class="script-toolbar">
                            <strong class="script-name">
                                {name}.lua
                                {isDirty(name) && <span class="script-dirty" title="Unsaved changes" />}
                            </strong>
                            <button
                                class="icon-button"
                                title="Save (Cmd+S)"
                                disabled={!isDirty(name)}
                                onClick={() => save(name)}
                            >
                                <ContentSaveOutlineIcon />
                            </button>
                            <button
                                class={`icon-button ${running.includes(name) ? 'is-running' : ''}`}
                                title={running.includes(name) ? 'Stop' : 'Start'}
                                onClick={() => toggleScript(ipc, name)}
                            >
                                {running.includes(name) ? <StopIcon /> : <PlayIcon />}
                            </button>
                            <button class="icon-button" title="Delete" onClick={() => remove(name)}>
                                <DeleteIcon />
                            </button>
                        </div>
                        {errors[name] && <pre class="script-error">{errors[name]}</pre>}
                        <CodeEditor
                            path={name}
                            source={scripts[name]}
                            onChange={(source) => setDrafts((drafts) => ({ ...drafts, [name]: source }))}
                            onSave={() => save(name)}
                        />
                    </div>
                ) : (
                    <p class="block script-empty">No scripts yet, add one in the sidebar</p>
                )}

                <div class="script-preview">
                    {$document.value && (
                        <Visualization
                            stage={$document.value.stage}
                            fixtureTypes={$document.value.fixtureTypes}
                            outputs={outputs}
                            selection={null}
                            onSelect={() => {}}
                            onScriptToggle={(script) => toggleScript(ipc, script)}
                        />
                    )}
                </div>
            </div>
        </>
    );
}
