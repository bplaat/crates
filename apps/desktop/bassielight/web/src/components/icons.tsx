/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    mdiAccount,
    mdiChartBellCurveCumulative,
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
export const ChartBellCurveCumulativeIcon = () => <Icon path={mdiChartBellCurveCumulative} />;
export const ChartLinearIcon = () => <Icon path="M4 19V20H22V22H2V2H4V16.75L22 3.75V6.25L4 19.25Z" />;
export const ChartStepIcon = () => <Icon path="M4 19V20H22V22H2V2H4V17H11V4H22V6H13V19H4Z" />;
export const CloseIcon = () => <Icon path={mdiClose} />;
export const CogIcon = () => <Icon path={mdiCog} />;
export const LightbulbIcon = () => <Icon path={mdiLightbulb} />;
export const LightbulbOffIcon = () => <Icon path={mdiLightbulbOff} />;
export const MusicIcon = () => <Icon path={mdiMusic} />;
export const MotionPlayOutlineIcon = () => <Icon path={mdiMotionPlayOutline} />;
export const QrcodeIcon = () => <Icon path={mdiQrcode} />;
export const RectangleOutlineIcon = () => <Icon path={mdiRectangleOutline} />;
export const SquareEditOutlineIcon = () => <Icon path={mdiSquareEditOutline} />;
