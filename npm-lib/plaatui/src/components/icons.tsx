/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    mdiAccount,
    mdiAccountMultiple,
    mdiArrowLeft,
    mdiButtonCursor,
    mdiCardTextOutline,
    mdiClipboardTextOutline,
    mdiClose,
    mdiCloudUpload,
    mdiCodeBraces,
    mdiCodeTags,
    mdiCog,
    mdiContentSave,
    mdiDelete,
    mdiDeleteOutline,
    mdiDownload,
    mdiFormatBold,
    mdiFormatItalic,
    mdiFormatLetterCase,
    mdiFormatListBulleted,
    mdiFormatListNumbered,
    mdiFormatQuoteOpen,
    mdiFormatStrikethrough,
    mdiFormatUnderline,
    mdiFormTextbox,
    mdiHistory,
    mdiLaptop,
    mdiLink,
    mdiLock,
    mdiLogin,
    mdiLogout,
    mdiMagnify,
    mdiMaterialDesign,
    mdiMessageTextOutline,
    mdiMinus,
    mdiNoteText,
    mdiPackageDown,
    mdiPackageUp,
    mdiPageLayoutHeader,
    mdiPageLayoutSidebarLeft,
    mdiPaletteSwatch,
    mdiPencil,
    mdiPin,
    mdiPlus,
    mdiPlusCircle,
    mdiProgressClock,
    mdiRestore,
    mdiSecurity,
    mdiTable,
    mdiTagOutline,
    mdiTextBox,
    mdiTray,
    mdiWeatherNight,
    mdiWeatherSunny,
} from '@mdi/js';
import { type JSX } from 'preact/jsx-runtime';
import './icons.css';
import { cx } from '../utils.ts';

const iconPaths = {
    account: mdiAccount,
    'account-multiple': mdiAccountMultiple,
    'arrow-left': mdiArrowLeft,
    'button-cursor': mdiButtonCursor,
    'card-text-outline': mdiCardTextOutline,
    'clipboard-text-outline': mdiClipboardTextOutline,
    close: mdiClose,
    'cloud-upload': mdiCloudUpload,
    'code-braces': mdiCodeBraces,
    'code-tags': mdiCodeTags,
    cog: mdiCog,
    'content-save': mdiContentSave,
    delete: mdiDelete,
    'delete-outline': mdiDeleteOutline,
    download: mdiDownload,
    'form-textbox': mdiFormTextbox,
    'format-bold': mdiFormatBold,
    'format-italic': mdiFormatItalic,
    'format-letter-case': mdiFormatLetterCase,
    'format-list-bulleted': mdiFormatListBulleted,
    'format-list-numbered': mdiFormatListNumbered,
    'format-quote-open': mdiFormatQuoteOpen,
    'format-strikethrough': mdiFormatStrikethrough,
    'format-underline': mdiFormatUnderline,
    history: mdiHistory,
    laptop: mdiLaptop,
    link: mdiLink,
    lock: mdiLock,
    login: mdiLogin,
    logout: mdiLogout,
    magnify: mdiMagnify,
    'material-design': mdiMaterialDesign,
    'message-text-outline': mdiMessageTextOutline,
    minus: mdiMinus,
    'note-text': mdiNoteText,
    'package-down': mdiPackageDown,
    'package-up': mdiPackageUp,
    'page-layout-header': mdiPageLayoutHeader,
    'page-layout-sidebar-left': mdiPageLayoutSidebarLeft,
    'palette-swatch': mdiPaletteSwatch,
    pencil: mdiPencil,
    pin: mdiPin,
    plus: mdiPlus,
    'plus-circle': mdiPlusCircle,
    'progress-clock': mdiProgressClock,
    restore: mdiRestore,
    security: mdiSecurity,
    table: mdiTable,
    'tag-outline': mdiTagOutline,
    'text-box': mdiTextBox,
    tray: mdiTray,
    'weather-night': mdiWeatherNight,
    'weather-sunny': mdiWeatherSunny,
} as const;

export type IconType = keyof typeof iconPaths;

// All available icon names, in declaration order.
export const iconTypes = Object.keys(iconPaths) as IconType[];

export type IconProps = Omit<JSX.IntrinsicElements['svg'], 'type'> & {
    type: IconType;
};

export function Icon({ type, class: className, ...props }: IconProps) {
    return (
        <svg {...props} class={cx('icon', className)} viewBox="0 0 24 24" fill="currentColor">
            <path d={iconPaths[type]} />
        </svg>
    );
}
