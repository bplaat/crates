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
import { cx } from '../utils.ts';
import './icons.css';

export type IconProps = JSX.IntrinsicElements['svg'] & {
    path: string;
};

export function Icon({ path, class: className, ...props }: IconProps) {
    return (
        <svg {...props} class={cx('icon', className)} viewBox="0 0 24 24" fill="currentColor">
            <path d={path} />
        </svg>
    );
}

function createIcon(name: string, path: string) {
    function PlaatIcon(props: Omit<IconProps, 'path'>) {
        return <Icon {...props} path={path} />;
    }
    PlaatIcon.displayName = name;
    return PlaatIcon;
}

export const AccountIcon = /*#__PURE__*/ createIcon('AccountIcon', mdiAccount);
export const AccountMultipleIcon = /*#__PURE__*/ createIcon('AccountMultipleIcon', mdiAccountMultiple);
export const ArrowLeftIcon = /*#__PURE__*/ createIcon('ArrowLeftIcon', mdiArrowLeft);
export const ButtonCursorIcon = /*#__PURE__*/ createIcon('ButtonCursorIcon', mdiButtonCursor);
export const CardTextOutlineIcon = /*#__PURE__*/ createIcon('CardTextOutlineIcon', mdiCardTextOutline);
export const ClipboardTextOutlineIcon = /*#__PURE__*/ createIcon('ClipboardTextOutlineIcon', mdiClipboardTextOutline);
export const CloseIcon = /*#__PURE__*/ createIcon('CloseIcon', mdiClose);
export const CloudUploadIcon = /*#__PURE__*/ createIcon('CloudUploadIcon', mdiCloudUpload);
export const CodeBracesIcon = /*#__PURE__*/ createIcon('CodeBracesIcon', mdiCodeBraces);
export const CodeTagsIcon = /*#__PURE__*/ createIcon('CodeTagsIcon', mdiCodeTags);
export const CogIcon = /*#__PURE__*/ createIcon('CogIcon', mdiCog);
export const ContentSaveIcon = /*#__PURE__*/ createIcon('ContentSaveIcon', mdiContentSave);
export const DeleteIcon = /*#__PURE__*/ createIcon('DeleteIcon', mdiDelete);
export const DeleteOutlineIcon = /*#__PURE__*/ createIcon('DeleteOutlineIcon', mdiDeleteOutline);
export const DownloadIcon = /*#__PURE__*/ createIcon('DownloadIcon', mdiDownload);
export const FormatBoldIcon = /*#__PURE__*/ createIcon('FormatBoldIcon', mdiFormatBold);
export const FormatItalicIcon = /*#__PURE__*/ createIcon('FormatItalicIcon', mdiFormatItalic);
export const FormatLetterCaseIcon = /*#__PURE__*/ createIcon('FormatLetterCaseIcon', mdiFormatLetterCase);
export const FormatListBulletedIcon = /*#__PURE__*/ createIcon('FormatListBulletedIcon', mdiFormatListBulleted);
export const FormatListNumberedIcon = /*#__PURE__*/ createIcon('FormatListNumberedIcon', mdiFormatListNumbered);
export const FormatQuoteOpenIcon = /*#__PURE__*/ createIcon('FormatQuoteOpenIcon', mdiFormatQuoteOpen);
export const FormatStrikethroughIcon = /*#__PURE__*/ createIcon('FormatStrikethroughIcon', mdiFormatStrikethrough);
export const FormatUnderlineIcon = /*#__PURE__*/ createIcon('FormatUnderlineIcon', mdiFormatUnderline);
export const FormTextboxIcon = /*#__PURE__*/ createIcon('FormTextboxIcon', mdiFormTextbox);
export const HistoryIcon = /*#__PURE__*/ createIcon('HistoryIcon', mdiHistory);
export const LaptopIcon = /*#__PURE__*/ createIcon('LaptopIcon', mdiLaptop);
export const LinkIcon = /*#__PURE__*/ createIcon('LinkIcon', mdiLink);
export const LockIcon = /*#__PURE__*/ createIcon('LockIcon', mdiLock);
export const LoginIcon = /*#__PURE__*/ createIcon('LoginIcon', mdiLogin);
export const LogoutIcon = /*#__PURE__*/ createIcon('LogoutIcon', mdiLogout);
export const MagnifyIcon = /*#__PURE__*/ createIcon('MagnifyIcon', mdiMagnify);
export const MaterialDesignIcon = /*#__PURE__*/ createIcon('MaterialDesignIcon', mdiMaterialDesign);
export const MessageTextOutlineIcon = /*#__PURE__*/ createIcon('MessageTextOutlineIcon', mdiMessageTextOutline);
export const MinusIcon = /*#__PURE__*/ createIcon('MinusIcon', mdiMinus);
export const NoteTextIcon = /*#__PURE__*/ createIcon('NoteTextIcon', mdiNoteText);
export const PackageDownIcon = /*#__PURE__*/ createIcon('PackageDownIcon', mdiPackageDown);
export const PackageUpIcon = /*#__PURE__*/ createIcon('PackageUpIcon', mdiPackageUp);
export const PageLayoutHeaderIcon = /*#__PURE__*/ createIcon('PageLayoutHeaderIcon', mdiPageLayoutHeader);
export const PageLayoutSidebarLeftIcon = /*#__PURE__*/ createIcon(
    'PageLayoutSidebarLeftIcon',
    mdiPageLayoutSidebarLeft,
);
export const PaletteSwatchIcon = /*#__PURE__*/ createIcon('PaletteSwatchIcon', mdiPaletteSwatch);
export const PencilIcon = /*#__PURE__*/ createIcon('PencilIcon', mdiPencil);
export const PinIcon = /*#__PURE__*/ createIcon('PinIcon', mdiPin);
export const PlusIcon = /*#__PURE__*/ createIcon('PlusIcon', mdiPlus);
export const PlusCircleIcon = /*#__PURE__*/ createIcon('PlusCircleIcon', mdiPlusCircle);
export const ProgressClockIcon = /*#__PURE__*/ createIcon('ProgressClockIcon', mdiProgressClock);
export const RestoreIcon = /*#__PURE__*/ createIcon('RestoreIcon', mdiRestore);
export const SecurityIcon = /*#__PURE__*/ createIcon('SecurityIcon', mdiSecurity);
export const TableIcon = /*#__PURE__*/ createIcon('TableIcon', mdiTable);
export const TagOutlineIcon = /*#__PURE__*/ createIcon('TagOutlineIcon', mdiTagOutline);
export const TextBoxIcon = /*#__PURE__*/ createIcon('TextBoxIcon', mdiTextBox);
export const TrayIcon = /*#__PURE__*/ createIcon('TrayIcon', mdiTray);
export const WeatherNightIcon = /*#__PURE__*/ createIcon('WeatherNightIcon', mdiWeatherNight);
export const WeatherSunnyIcon = /*#__PURE__*/ createIcon('WeatherSunnyIcon', mdiWeatherSunny);
