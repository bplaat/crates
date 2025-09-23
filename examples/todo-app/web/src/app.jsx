/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useState, useEffect } from 'preact/hooks';
import { Ipc } from './ipc.js';

const ipc = new Ipc();

export function App() {
    const [todos, setTodos] = useState([]);
    const [input, setInput] = useState('');

    useEffect(async () => {
        const { todos } = await ipc.request('getTodos');
        setTodos(todos);
    }, []);
    useEffect(async () => {
        await ipc.send('updateTodos', { todos });
    }, [todos]);

    const addTodo = (e) => {
        e.preventDefault();
        if (input.trim() === '') return;
        setTodos([...todos, { id: crypto.randomUUID(), text: input, completed: false }]);
        setInput('');
    };
    const removeCompleted = (e) => {
        e.preventDefault();
        setTodos(todos.filter((todo) => !todo.completed));
    };
    const completeTodo = (index, completed) => {
        setTodos(todos.map((todo, i) => (i === index ? { ...todo, completed } : todo)));
    };

    return (
        <>
            <h1>Todo App</h1>
            <form onSubmit={addTodo}>
                <input type="text" value={input} onInput={(e) => setInput(e.target.value)} placeholder="Add a todo" />
                <button type="submit">Add</button>
                <button type="button" onClick={removeCompleted} disabled={todos.length === 0}>
                    Clear done
                </button>
            </form>

            {todos.length === 0 ? (
                <p>
                    <i>No todos yet!</i>
                </p>
            ) : (
                <ul>
                    {todos.map((todo, i) => (
                        <li
                            key={i}
                            class={todo.completed ? 'is-completed' : ''}
                            onClick={() => completeTodo(i, !todo.completed)}
                        >
                            <input
                                type="checkbox"
                                checked={todo.completed}
                                onChange={(e) => completeTodo(i, e.target.checked)}
                            />
                            {todo.text}
                        </li>
                    ))}
                </ul>
            )}
        </>
    );
}
