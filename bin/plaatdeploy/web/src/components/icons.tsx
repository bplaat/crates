/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

function Icon({ path, class: className }: { path: string; class?: string }) {
    return (
        <svg
            class={className ? `icon ${className}` : 'icon'}
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            <path d={path} />
        </svg>
    );
}

export const HomeIcon = ({ class: c }: { class?: string }) => (
    <Icon path="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" class={c} />
);

export const RocketIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M13.13 22.19 11.5 18.36c1.57-.58 3.04-1.36 4.4-2.27l-2.77 6.1M5.64 12.5 1.81 10.87l6.1-2.77c-.91 1.36-1.69 2.83-2.27 4.4M21.61 2.39S16.66.27 11 5.93c-2.19 2.19-3.5 4.6-4.35 6.71-.28.75-.09 1.57.46 2.13l2.13 2.12c.55.56 1.37.74 2.12.46 2.11-.85 4.52-2.16 6.71-4.35 5.66-5.66 3.53-10.61 3.53-10.61M14.54 9.46c-.78-.78-.78-2.05 0-2.83s2.05-.78 2.83 0 .78 2.05 0 2.83-2.05.78-2.83 0z"
        class={c}
    />
);

export const CloudUploadIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"
        class={c}
    />
);

export const RefreshIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M17.65 6.35A7.96 7.96 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"
        class={c}
    />
);

export const GithubIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34-.46-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.87 1.52 2.34 1.07 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.92 0-1.11.38-2 1.03-2.71-.1-.25-.45-1.29.1-2.64 0 0 .84-.27 2.75 1.02.79-.22 1.65-.33 2.5-.33.85 0 1.71.11 2.5.33 1.91-1.29 2.75-1.02 2.75-1.02.55 1.35.2 2.39.1 2.64.65.71 1.03 1.6 1.03 2.71 0 3.82-2.34 4.66-4.57 4.91.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2z"
        class={c}
    />
);

export const OpenInNewIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3m-2 16H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7z"
        class={c}
    />
);

export const ConsoleIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M20 19V7H4v12h16M20 3a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5c0-1.11.9-2 2-2h16M13 17v-2h5v2h-5M9.58 13l-4.01-4H8.4l3.3 3.3c.39.39.39 1.03 0 1.42L8.42 17H5.59L9.58 13z"
        class={c}
    />
);

export const PlusIcon = ({ class: c }: { class?: string }) => (
    <Icon path="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" class={c} />
);

export const PencilIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"
        class={c}
    />
);

export const DeleteIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM8 9h8v10H8V9zm7.5-5l-1-1h-5l-1 1H5v2h14V4h-3.5z"
        class={c}
    />
);

export const CloseIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
        class={c}
    />
);

export const ArrowLeftIcon = ({ class: c }: { class?: string }) => (
    <Icon path="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" class={c} />
);

export const CogIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"
        class={c}
    />
);

export const LogoutIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M17 7l-1.41 1.41L18.17 11H8v2h10.17l-2.58 2.58L17 17l5-5zM4 5h8V3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h8v-2H4V5z"
        class={c}
    />
);

export const AccountIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
        class={c}
    />
);

export const AccountMultipleIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"
        class={c}
    />
);

export const LaptopIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M4 6h18V4H4c-1.1 0-2 .9-2 2v11H0v3h14v-3H4V6zm19 2h-6c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h6c.55 0 1-.45 1-1V9c0-.55-.45-1-1-1zm-1 9h-4v-7h4v7z"
        class={c}
    />
);

export const ShieldIcon = ({ class: c }: { class?: string }) => (
    <Icon
        path="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 4l4 1.78V11c0 3.35-2.32 6.48-4 7.44-1.68-.96-4-4.09-4-7.44V6.78L12 5z"
        class={c}
    />
);
