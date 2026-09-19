/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    Avatar,
    CogIcon,
    DropdownDivider,
    DropdownItem,
    DropdownMenu,
    LogoutIcon,
    Navbar,
    NavbarMenu,
    NavbarSpacer,
    NavbarUserButton,
    NavbarUserName,
    PageLayoutSidebarLeftIcon,
    SidebarLayout,
    SidebarLink,
    useClickOutside,
} from 'plaatui';
import { type ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import { $authUser, logout } from '../services/auth.service.ts';

export function AppLayout({ children }: { children: ComponentChildren }) {
    const user = $authUser.value!;
    const [, navigate] = useLocation();
    const [open, setOpen] = useState(false);
    const menuRef = useClickOutside<HTMLDivElement>(open, () => setOpen(false));
    return (
        <SidebarLayout
            version={__APP_VERSION__}
            navbar={
                <Navbar>
                    <a
                        class="navbar-brand"
                        href="/"
                        onClick={(event) => {
                            event.preventDefault();
                            navigate('/');
                        }}
                    >
                        <span class="deploy-mark" aria-hidden="true">
                            PD
                        </span>
                        <span class="navbar-brand-name">Plaat Deploy</span>
                    </a>
                    <NavbarSpacer />
                    <NavbarMenu ref={menuRef}>
                        <NavbarUserButton onClick={() => setOpen(!open)}>
                            <Avatar>
                                {user.firstName[0]}
                                {user.lastName[0]}
                            </Avatar>
                            <NavbarUserName>
                                {user.firstName} {user.lastName}
                            </NavbarUserName>
                        </NavbarUserButton>
                        {open && (
                            <DropdownMenu>
                                <DropdownItem onClick={() => navigate('/settings')}>
                                    <CogIcon class="is-sm" />
                                    Settings
                                </DropdownItem>
                                <DropdownDivider />
                                <DropdownItem onClick={() => void logout()}>
                                    <LogoutIcon class="is-sm" />
                                    Log out
                                </DropdownItem>
                            </DropdownMenu>
                        )}
                    </NavbarMenu>
                </Navbar>
            }
            sidebar={
                <>
                    <SidebarLink
                        href="/"
                        label="Projects"
                        icon={PageLayoutSidebarLeftIcon}
                        active={location.pathname === '/'}
                    />
                    <SidebarLink
                        href="/settings"
                        label="Settings"
                        icon={CogIcon}
                        active={location.pathname.startsWith('/settings')}
                    />
                </>
            }
        >
            {children}
        </SidebarLayout>
    );
}
