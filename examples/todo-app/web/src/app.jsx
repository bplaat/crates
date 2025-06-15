/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect } from 'preact/hooks';

export function App() {
    const [todos, setTodos] = useState([]);
    const [input, setInput] = useState('');

    useEffect(() => {
        window.ipc.postMessage(JSON.stringify({ type: 'get-todos' }));
        const messageListener = (event) => {
            const message = JSON.parse(event.data);
            if (message.type === 'get-todos-response') {
                setTodos(message.todos);
            }
        };
        window.ipc.addEventListener('message', messageListener);
        return () => {
            window.ipc.removeEventListener('message', messageListener);
        };
    }, []);

    useEffect(() => {
        window.ipc.postMessage(JSON.stringify({ type: 'update-todos', todos }));
    }, [todos]);

    function addTodo(e) {
        e.preventDefault();
        if (input.trim() === '') return;
        setTodos([...todos, { id: crypto.randomUUID(), text: input, completed: false }]);
        setInput('');
    }

    function toggleTodo(index) {
        setTodos(todos.map((todo, i) => (i === index ? { ...todo, completed: !todo.completed } : todo)));
    }

    return (
        <div>
            <h1>Todo App</h1>
            <form onSubmit={addTodo}>
                <input type="text" value={input} onInput={(e) => setInput(e.target.value)} placeholder="Add a todo" />
                <button type="submit">Add</button>
            </form>

            {todos.length === 0 ? (
                <p>
                    <i>No todos yet!</i>
                </p>
            ) : (
                <ul>
                    {todos.map((todo, i) => (
                        <li key={i} className={todo.completed ? 'is-completed' : ''} onClick={() => toggleTodo(i)}>
                            <input type="checkbox" checked={todo.completed} onChange={() => toggleTodo(i)} />
                            {todo.text}
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
