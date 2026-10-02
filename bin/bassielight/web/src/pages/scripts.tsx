/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useContext, useEffect, useState } from 'preact/hooks';
import { IpcContext } from '../app.tsx';
import { CodeEditor, disposeCodeModels } from '../components/code-editor.tsx';
import {
    ContentSaveOutlineIcon,
    DeleteIcon,
    FolderOpenOutlineIcon,
    FolderPlusOutlineIcon,
    PlayIcon,
    PlusIcon,
    StopIcon,
} from '../components/icons.tsx';
import { Visualization } from '../components/visualization.tsx';
import {
    $document,
    $scripts,
    buttonScripts,
    scriptFolder,
    scriptsRunning,
    scriptTree,
    toggleScript,
    toggleScripts,
    useDmxOutput,
} from '../stage.ts';
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

/// Same relative path rule as the app
const isValidName = (name: string) =>
    name.length > 0 &&
    name.length <= 255 &&
    name.split('/').every((part) => /^[A-Za-z0-9 _-]{1,64}$/.test(part) && part.trim() === part);

export function ScriptsPage() {
    const ipc = useContext(IpcContext)!;
    const { scripts, folders = [], running, errors } = $scripts.value;
    const names = Object.keys(scripts);
    const [selected, setSelected] = useState<string | null>(null);
    // Unsaved edits per script, scripts without edits follow their file, like when AI agents change it
    const [drafts, setDrafts] = useState<Record<string, string>>({});
    const [newName, setNewName] = useState('');
    const [newFolder, setNewFolder] = useState('');
    const [directory, setDirectory] = useState('');
    const currentDirectory = folders.includes(directory) ? directory : '';
    const pathInDirectory = (name: string) => (currentDirectory ? `${currentDirectory}/${name}` : name);
    const newScriptPath = pathInDirectory(newName);
    const newFolderPath = pathInDirectory(newFolder);
    // Scripts run while editing them and the stage folder keeps following changes
    const outputs = useDmxOutput(ipc);

    const name = selected !== null && selected in scripts ? selected : (names[0] ?? null);
    const isDirty = (script: string) => script in drafts && drafts[script] !== scripts[script];

    useEffect(() => disposeCodeModels(names), [names.join('\n')]);

    const save = (script: string) => {
        if (!isDirty(script)) return;
        ipc.send('saveScript', { name: script, source: drafts[script] });
        const { [script]: _, ...rest } = drafts;
        setDrafts(rest);
    };
    const create = () => {
        if (!newName || !isValidName(newScriptPath) || newScriptPath in scripts) return;
        ipc.send('saveScript', { name: newScriptPath, source: TEMPLATE });
        setSelected(newScriptPath);
        setNewName('');
    };
    const createFolder = () => {
        if (!newFolder || !isValidName(newFolderPath) || folders.includes(newFolderPath)) return;
        ipc.send('createScriptFolder', { name: newFolderPath });
        setDirectory(newFolderPath);
        setNewFolder('');
    };
    const remove = (script: string) => {
        if (!confirm(`Delete script ${script}?`)) return;
        ipc.send('deleteScript', { name: script });
        const { [script]: _, ...rest } = drafts;
        setDrafts(rest);
    };

    return (
        <>
            <div class="sidebar is-left script-sidebar">
                <h2 class="title">Scripts</h2>
                <nav class="list script-tree" aria-label="Script files">
                    <button
                        class={`script-folder ${currentDirectory === '' ? 'is-selected' : ''}`}
                        onClick={() => setDirectory('')}
                        aria-pressed={currentDirectory === ''}
                    >
                        <FolderOpenOutlineIcon /> scripts
                    </button>
                    {names.length === 0 && folders.length === 0 && <p>No scripts yet</p>}
                    {scriptTree(names, folders).map(({ path: script, label, depth, folder }) =>
                        folder ? (
                            <button
                                key={`folder:${script}`}
                                class={`script-folder ${currentDirectory === script ? 'is-selected' : ''}`}
                                style={{ marginLeft: `${depth * 0.75}rem` }}
                                title={script}
                                aria-pressed={currentDirectory === script}
                                onClick={() => setDirectory(script)}
                            >
                                <FolderOpenOutlineIcon /> {label}
                            </button>
                        ) : (
                            <div
                                key={script}
                                class={`script-item ${script === name ? 'is-selected' : ''}`}
                                style={{ marginLeft: `${depth * 0.75}rem` }}
                                title={script}
                            >
                                <button
                                    class="script-item-name"
                                    onClick={() => {
                                        setSelected(script);
                                        setDirectory(scriptFolder(script));
                                    }}
                                >
                                    <span
                                        class={`script-dot ${running.includes(script) ? 'is-running' : ''} ${errors[script] ? 'is-error' : ''}`}
                                    />
                                    {label}
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
                        ),
                    )}
                </nav>
                <p class="script-location">Create in: {currentDirectory || 'scripts'}</p>
                <div class="script-new">
                    <input
                        class="input"
                        placeholder="New script name"
                        value={newName}
                        onInput={(e) => setNewName(e.currentTarget.value)}
                        onKeyDown={(e) => e.key === 'Enter' && create()}
                    />
                    <button
                        class="icon-button"
                        title="Add script"
                        disabled={!newName || !isValidName(newScriptPath) || newScriptPath in scripts}
                        onClick={create}
                    >
                        <PlusIcon />
                    </button>
                </div>
                <div class="script-new">
                    <input
                        class="input"
                        placeholder="New folder name"
                        value={newFolder}
                        onInput={(e) => setNewFolder(e.currentTarget.value)}
                        onKeyDown={(e) => e.key === 'Enter' && createFolder()}
                    />
                    <button
                        class="icon-button"
                        title="Add folder"
                        disabled={!newFolder || !isValidName(newFolderPath) || folders.includes(newFolderPath)}
                        onClick={createFolder}
                    >
                        <FolderPlusOutlineIcon />
                    </button>
                </div>

                <p class="block script-hint">
                    Combine scripts that control different groups. Run full-room shows on their own. Timing follows the
                    BPM button; the stage folder has an AGENTS.md with the full API.
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
                            onButtonPress={(button) => toggleScripts(ipc, buttonScripts(button))}
                            isButtonActive={(button) => scriptsRunning(buttonScripts(button))}
                        />
                    )}
                </div>
            </div>
        </>
    );
}
