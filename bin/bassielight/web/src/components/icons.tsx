/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    mdiAccount,
    mdiClose,
    mdiCog,
    mdiLightbulb,
    mdiLightbulbOff,
    mdiMotionPlayOutline,
    mdiMusic,
    mdiQrcode,
    mdiRectangleOutline,
    mdiSquareEditOutline,
} from '@mdi/js';

function Icon({ path }: { path: string }) {
    return (
        <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
            <path d={path} />
        </svg>
    );
}

export const AccountIcon = () => <Icon path={mdiAccount} />;
export const CloseIcon = () => <Icon path={mdiClose} />;
export const CogIcon = () => <Icon path={mdiCog} />;
export const LightbulbIcon = () => <Icon path={mdiLightbulb} />;
export const LightbulbOffIcon = () => <Icon path={mdiLightbulbOff} />;
export const MusicIcon = () => <Icon path={mdiMusic} />;
export const MotionPlayOutlineIcon = () => <Icon path={mdiMotionPlayOutline} />;
export const QrcodeIcon = () => <Icon path={mdiQrcode} />;
export const RectangleOutlineIcon = () => <Icon path={mdiRectangleOutline} />;
export const SquareEditOutlineIcon = () => <Icon path={mdiSquareEditOutline} />;
export const TweenDirect = () => <Icon path="M4,19V20H22V22H2V2H4V17H12V4H22V6H14V19L4,19Z" />;
export const TweenLinear = () => <Icon path="M4,19V20H22V22H2V2H4V17L21.5,4V6L4,19Z" />;
export const TweenEase = () => (
    <Icon path="M4 19V20H22V22H2V2H4V17C7 17 10 15 12.1 11.4C15.1 6.4 18.4 4 22 4V6C19.2 6 16.5 8.1 13.9 12.5C11.3 16.6 7.7 19 4 19Z" />
);
