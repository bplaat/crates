/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import { useLocation } from 'wouter-preact';
import { useEffect, useState } from 'preact/hooks';
import { $authUser, logout } from '../services/auth.service.ts';
import { $searchQuery } from '../services/notes.service.ts';
import { t } from '../services/i18n.service.ts';
import {
    Avatar,
    DropdownItem,
    DropdownDivider,
    DropdownMenu,
    Icon,
    Navbar,
    NavbarBrand,
    NavbarMenu,
    NavbarSearch,
    NavbarSpacer,
    NavbarUserButton,
    NavbarUserName,
    SearchInput,
    useClickOutside,
} from 'plaatui';
import { lastNameInitial } from '../utils.ts';

export function PlaatNotesNavbar({ showSearch = false }: { showSearch?: boolean }) {
    const user = $authUser.value;
    const [dropdownOpen, setDropdownOpen] = useState(false);
    const dropdownRef = useClickOutside<HTMLDivElement>(dropdownOpen, () => setDropdownOpen(false));
    const [, navigate] = useLocation();

    useEffect(() => {
        if (!showSearch) $searchQuery.value = '';
    }, [showSearch]);

    async function handleLogout() {
        setDropdownOpen(false);
        await logout();
        navigate('/auth/login');
    }

    return (
        <Navbar>
            <NavbarBrand
                href="/"
                image="/assets/icon.svg"
                name="PlaatNotes"
                onClick={(event) => {
                    event.preventDefault();
                    navigate('/');
                }}
            />

            {showSearch && (
                <NavbarSearch>
                    <SearchInput
                        value={$searchQuery.value}
                        onInput={(v) => ($searchQuery.value = v)}
                        onClear={() => ($searchQuery.value = '')}
                        placeholder={t('nav.search')}
                    />
                </NavbarSearch>
            )}

            <NavbarSpacer />

            {user && (
                <NavbarMenu ref={dropdownRef}>
                    <NavbarUserButton onClick={() => setDropdownOpen(!dropdownOpen)}>
                        <Avatar>
                            {user.firstName[0].toUpperCase()}
                            {lastNameInitial(user.lastName)}
                        </Avatar>
                        <NavbarUserName>
                            {user.firstName} {user.lastName}
                        </NavbarUserName>
                    </NavbarUserButton>

                    {dropdownOpen && (
                        <DropdownMenu>
                            {user.role === 'admin' && (
                                <DropdownItem
                                    onClick={() => {
                                        setDropdownOpen(false);
                                        navigate('/admin/users');
                                    }}
                                >
                                    <Icon type="security" class="is-sm" />
                                    {t('nav.admin')}
                                </DropdownItem>
                            )}
                            <DropdownItem
                                onClick={() => {
                                    setDropdownOpen(false);
                                    navigate('/settings');
                                }}
                            >
                                <Icon type="cog" class="is-sm" />
                                {t('nav.settings')}
                            </DropdownItem>
                            <DropdownDivider />
                            <DropdownItem onClick={handleLogout}>
                                <Icon type="logout" class="is-sm" />
                                {t('nav.logout')}
                            </DropdownItem>
                        </DropdownMenu>
                    )}
                </NavbarMenu>
            )}
        </Navbar>
    );
}
