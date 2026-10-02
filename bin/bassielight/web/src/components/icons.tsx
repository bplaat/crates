/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    mdiAccount,
    mdiChartBellCurveCumulative,
    mdiClose,
    mdiContentSaveOutline,
    mdiDelete,
    mdiFilePlusOutline,
    mdiFlash,
    mdiFolderOpenOutline,
    mdiFolderPlusOutline,
    mdiGestureTapButton,
    mdiGroup,
    mdiLightbulb,
    mdiLightbulbOff,
    mdiMetronome,
    mdiMicrophone,
    mdiMotionPlayOutline,
    mdiMusic,
    mdiPlay,
    mdiPlaylistPlay,
    mdiPlus,
    mdiPowerSocketDe,
    mdiQrcode,
    mdiScriptTextOutline,
    mdiSpotlightBeam,
    mdiSquareEditOutline,
    mdiStop,
} from '@mdi/js';
import type { ControlKind } from '../stage.ts';

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
export const ContentSaveOutlineIcon = () => <Icon path={mdiContentSaveOutline} />;
export const DeleteIcon = () => <Icon path={mdiDelete} />;
export const FilePlusOutlineIcon = () => <Icon path={mdiFilePlusOutline} />;
export const FolderOpenOutlineIcon = () => <Icon path={mdiFolderOpenOutline} />;
export const FolderPlusOutlineIcon = () => <Icon path={mdiFolderPlusOutline} />;
export const FlashIcon = () => <Icon path={mdiFlash} />;
export const GestureTapButtonIcon = () => <Icon path={mdiGestureTapButton} />;
export const GroupIcon = () => <Icon path={mdiGroup} />;
export const LightbulbIcon = () => <Icon path={mdiLightbulb} />;
export const LightbulbOffIcon = () => <Icon path={mdiLightbulbOff} />;
export const MetronomeIcon = () => <Icon path={mdiMetronome} />;
export const MicrophoneIcon = () => <Icon path={mdiMicrophone} />;
export const MusicIcon = () => <Icon path={mdiMusic} />;
export const MotionPlayOutlineIcon = () => <Icon path={mdiMotionPlayOutline} />;
export const PlaylistPlayIcon = () => <Icon path={mdiPlaylistPlay} />;
export const PlayIcon = () => <Icon path={mdiPlay} />;
export const PlusIcon = () => <Icon path={mdiPlus} />;
export const PowerSocketDeIcon = () => <Icon path={mdiPowerSocketDe} />;
export const QrcodeIcon = () => <Icon path={mdiQrcode} />;
export const ScriptTextOutlineIcon = () => <Icon path={mdiScriptTextOutline} />;
export const SpotlightBeamIcon = () => <Icon path={mdiSpotlightBeam} />;
export const StopIcon = () => <Icon path={mdiStop} />;
export const SquareEditOutlineIcon = () => <Icon path={mdiSquareEditOutline} />;

const KIND_ICONS: Record<ControlKind, () => preact.JSX.Element> = {
    rgb: LightbulbIcon,
    switch: PowerSocketDeIcon,
    strobe: FlashIcon,
    preset: PlaylistPlayIcon,
    movingHead: SpotlightBeamIcon,
};

export function KindIcon({ kind }: { kind: ControlKind }) {
    const KindIconComponent = KIND_ICONS[kind];
    return (
        <span class={`kind-icon is-${kind}`}>
            <KindIconComponent />
        </span>
    );
}
