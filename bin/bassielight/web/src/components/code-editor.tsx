/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import EditorWorker from 'monaco-editor/editor/editor.worker?worker';
import { useEffect, useRef } from 'preact/hooks';
import * as monaco from '../monaco.ts';
import './code-editor.css';

(globalThis as { MonacoEnvironment?: monaco.Environment }).MonacoEnvironment = {
    getWorker: () => new EditorWorker(),
};

monaco.editor.defineTheme('bassielight', {
    base: 'vs-dark',
    inherit: true,
    rules: [],
    colors: {
        'editor.background': '#18181b',
        'editorGutter.background': '#18181b',
        'editor.lineHighlightBackground': '#27272a',
    },
});

// Completions for the script API, see scripting.md in the app
const FUNCTIONS: [string, string, string][] = [
    ['fixtures', 'fixtures(${1})', 'All fixtures, or only the fixtures of a kind'],
    ['group', 'group("${1:name}")', 'The fixtures in a group'],
    ['fixture', 'fixture("${1:name}")', 'One fixture by name or id'],
    ['wait', 'wait(${1:1})', 'Wait beats'],
    ['sync', 'sync(${1:4})', 'Wait until the next multiple of beats'],
    ['beat', 'beat()', 'Current position in beats'],
    ['bpm', 'bpm()', 'Current tempo'],
    ['mode', 'mode("${1|manual,black,auto|}")', 'Switch the mode'],
    ['rgb', 'rgb(${1:255}, ${2:0}, ${3:0})', 'Color from 0 to 255 parts'],
    ['hsv', 'hsv(${1:0}, ${2:1}, ${3:1})', 'Color from a hue of 0 to 360'],
];
const METHODS: [string, string, string][] = [
    ['set', 'set({ ${1} })', 'Set props'],
    ['tween', 'tween({ ${1} }, ${2:1})', 'Fade props over beats'],
    ['get', 'get("${1:color}")', 'Get a prop'],
    ['each', 'each(function(fixture, index)\n\t${1}\nend)', 'Call a function for each fixture'],
    ['filter', 'filter(function(fixture)\n\treturn ${1:true}\nend)', 'Selection of the accepted fixtures'],
    ['sorted', 'sorted("${1|x,y,name,id|}")', 'Selection sorted by a field'],
];
const PROPS = [
    'color',
    'toggle_color',
    'intensity',
    'toggle_tween',
    'toggle_speed',
    'strobe_speed',
    'preset',
    'preset_speed',
    'gobo',
    'focus',
    'movement',
    'movement_speed',
    'switches',
    'switch_on',
    'switch_all_press',
    'flash_on',
    'flash_intensity',
    'flash_speed',
];

monaco.languages.registerCompletionItemProvider('lua', {
    triggerCharacters: [':'],
    provideCompletionItems(model, position) {
        const word = model.getWordUntilPosition(position);
        const range = {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
        };
        const before = model.getValueInRange({ ...range, startColumn: 1 }).slice(0, word.startColumn - 1);
        const snippet = monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet;
        const items = (entries: [string, string, string][], kind: monaco.languages.CompletionItemKind) =>
            entries.map(([label, insertText, documentation]) => ({
                label,
                kind,
                insertText,
                insertTextRules: snippet,
                documentation,
                range,
            }));
        if (before.endsWith(':')) {
            return { suggestions: items(METHODS, monaco.languages.CompletionItemKind.Method) };
        }
        return {
            suggestions: [
                ...items(FUNCTIONS, monaco.languages.CompletionItemKind.Function),
                ...PROPS.map((prop) => ({
                    label: prop,
                    kind: monaco.languages.CompletionItemKind.Property,
                    insertText: `${prop} = `,
                    range,
                })),
            ],
        };
    },
});

const modelUri = (path: string) => monaco.Uri.from({ scheme: 'inmemory', authority: 'scripts', path: `/${path}.lua` });

/// Monaco code editor for Lua scripts, each path keeps its own model and undo history. The editor
/// owns its text, `source` is the saved file which replaces the text when it has no unsaved edits.
export function CodeEditor({
    path,
    source,
    onChange,
    onSave,
}: {
    path: string;
    source: string;
    onChange: (value: string) => void;
    onSave: () => void;
}) {
    const container = useRef<HTMLDivElement>(null);
    const editor = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    // Last saved source each model was in sync with
    const synced = useRef(new Map<string, string>());
    const callbacks = useRef({ onChange, onSave });
    callbacks.current = { onChange, onSave };

    useEffect(() => {
        const instance = monaco.editor.create(container.current!, {
            theme: 'bassielight',
            automaticLayout: true,
            minimap: { enabled: false },
            fontSize: 13,
            tabSize: 4,
            insertSpaces: true,
            scrollBeyondLastLine: false,
            padding: { top: 8 },
        });
        instance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => callbacks.current.onSave());
        instance.onDidChangeModelContent(() => callbacks.current.onChange(instance.getValue()));
        editor.current = instance;
        return () => instance.dispose();
    }, []);

    useEffect(() => {
        const uri = modelUri(path);
        let model = monaco.editor.getModel(uri);
        if (!model) {
            model = monaco.editor.createModel(source, 'lua', uri);
            synced.current.set(path, source);
        }
        editor.current!.setModel(model);
    }, [path]);

    // Apply changes to the file from outside, like an AI agent, as an edit so undo still works
    useEffect(() => {
        const model = editor.current!.getModel();
        if (!model) return;
        const text = model.getValue();
        if (text !== source && text === synced.current.get(path)) {
            model.pushEditOperations([], [{ range: model.getFullModelRange(), text: source }], () => null);
        }
        synced.current.set(path, source);
    }, [path, source]);

    return <div ref={container} class="code-editor" />;
}

/// Free the models of scripts that no longer exist
export function disposeCodeModels(paths: string[]) {
    const keep = new Set(paths.map((path) => modelUri(path).toString()));
    for (const model of monaco.editor.getModels()) {
        if (!keep.has(model.uri.toString())) model.dispose();
    }
}
