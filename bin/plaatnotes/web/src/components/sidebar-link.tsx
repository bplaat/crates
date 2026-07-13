/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { SidebarLink as PlaatuiSidebarLink, type SidebarLinkProps } from 'plaatui';
import { useLocation, useRoute } from 'wouter-preact';

type Props = Pick<SidebarLinkProps, 'href' | 'label' | 'icon'>;

export function SidebarLink(props: Props) {
    const [active] = useRoute(props.href);
    const [, navigate] = useLocation();

    return (
        <PlaatuiSidebarLink
            {...props}
            active={active}
            onClick={(event) => {
                event.preventDefault();
                navigate(props.href);
            }}
        />
    );
}
