/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { cpSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig, type Plugin } from 'vite';
import preact from '@preact/preset-vite';

const require = createRequire(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

function swaggerUi(): Plugin {
    return {
        name: 'swagger-ui',
        apply: 'build',
        closeBundle() {
            const outDir = path.resolve(__dirname, 'dist/swagger-ui');
            mkdirSync(outDir, { recursive: true });

            const swaggerUiDistDir = path.dirname(require.resolve('swagger-ui-dist/package.json'));
            for (const file of ['swagger-ui.css', 'swagger-ui-bundle.js']) {
                cpSync(path.join(swaggerUiDistDir, file), path.join(outDir, file));
            }

            cpSync(path.resolve(__dirname, '../openapi.yaml'), path.join(outDir, 'openapi.yaml'));

            writeFileSync(
                path.join(outDir, 'swagger-initializer.js'),
                `window.onload = () => {
    window.ui = SwaggerUIBundle({
        url: './openapi.yaml',
        dom_id: '#swagger-ui',
    });
};
`,
            );

            writeFileSync(
                path.join(outDir, 'index.html'),
                `<!doctype html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>PlaatNotes Swagger UI</title>
        <link rel="icon" href="data:," />
        <link rel="stylesheet" href="./swagger-ui.css" />
    </head>
    <body>
        <div id="swagger-ui"></div>
        <script src="./swagger-ui-bundle.js"></script>
        <script src="./swagger-initializer.js"></script>
    </body>
</html>
`,
            );
        },
    };
}

const { version } = JSON.parse(readFileSync('./package.json', 'utf-8'));

export default defineConfig({
    plugins: [preact(), tailwindcss(), swaggerUi()],
    define: {
        __APP_VERSION__: JSON.stringify(version),
    },
    server: {
        proxy: {
            '/api': 'http://localhost:8080',
        },
    },
});
