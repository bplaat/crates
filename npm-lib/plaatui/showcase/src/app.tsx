/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

import {
    AccountIcon,
    Avatar,
    Badge,
    Button,
    Card,
    CogIcon,
    ConfirmDialog,
    ContentSaveIcon,
    DangerButton,
    DangerTextButton,
    DeleteOutlineIcon,
    Dialog,
    DropdownDivider,
    DropdownItem,
    DropdownMenu,
    EmptyState,
    Fab,
    Form,
    FormActions,
    FormField,
    FormInput,
    FormMessage,
    FormSelect,
    IconButton,
    LoadingText,
    LogoutIcon,
    MagnifyIcon,
    Navbar,
    NavbarBrand,
    NavbarMenu,
    NavbarSearch,
    NavbarSpacer,
    NavbarUserButton,
    NavbarUserName,
    Page,
    PageTitle,
    PencilIcon,
    PlusIcon,
    SearchInput,
    SecondaryButton,
    SidebarLayout,
    SidebarLink,
    SmallIconButton,
    Table,
    TextButton,
    useClickOutside,
} from 'plaatui';
import { type ComponentChildren } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { iconCatalog, iconEntries, type IconType } from './icon-catalog.ts';

function renderIcon(type: IconType, className: string) {
    const IconComponent = iconCatalog[type];
    return <IconComponent class={className} />;
}

const BRAND_IMAGE =
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='%23facc15'%3E%3Crect width='24' height='24' rx='6'/%3E%3C/svg%3E";

// Items for the standalone sidebar example.
const SIDEBAR_DEMO: { id: string; label: string; icon: IconType }[] = [
    { id: 'home', label: 'Home', icon: 'note-text' },
    { id: 'archive', label: 'Archive', icon: 'package-down' },
    { id: 'account', label: 'Account', icon: 'account' },
    { id: 'settings', label: 'Settings', icon: 'cog' },
];

// Theme color tokens shown as swatches (they adapt to light/dark).
const COLOR_TOKENS = [
    '--color-background',
    '--color-surface',
    '--color-surface-2',
    '--color-hover',
    '--color-border',
    '--color-text',
    '--color-text-muted',
    '--color-text-soft',
    '--color-text-subtle',
    '--color-accent-background',
    '--color-accent-text',
    '--color-badge-background',
    '--color-danger',
    '--color-success',
    '--color-link',
];

interface SectionDef {
    id: string;
    label: string;
    icon: IconType;
    description: string;
    render: () => ComponentChildren;
}

function Section({
    id,
    title,
    description,
    children,
}: {
    id: string;
    title: string;
    description: string;
    children: ComponentChildren;
}) {
    return (
        <section id={id} class="showcase-section">
            <h2 class="showcase-section-title">{title}</h2>
            <p class="showcase-section-desc">{description}</p>
            <Card>{children}</Card>
        </section>
    );
}

const THEME_KEY = 'plaatui-showcase-theme';

export function App() {
    const [dark, setDark] = useState(() => {
        const saved = localStorage.getItem(THEME_KEY);
        if (saved === 'dark') return true;
        if (saved === 'light') return false;
        return window.matchMedia('(prefers-color-scheme: dark)').matches;
    });
    const toggleTheme = () => {
        setDark((d) => {
            const next = !d;
            localStorage.setItem(THEME_KEY, next ? 'dark' : 'light');
            return next;
        });
    };
    const [query, setQuery] = useState('');
    const [menuOpen, setMenuOpen] = useState(false);
    const [active, setActive] = useState('colors');

    // Per-demo interactive state.
    const [text, setText] = useState('');
    const [select, setSelect] = useState('note-text');
    const [demoSearch, setDemoSearch] = useState('');
    const [dialogOpen, setDialogOpen] = useState(false);
    const [formDialogOpen, setFormDialogOpen] = useState(false);
    const [confirmOpen, setConfirmOpen] = useState(false);
    const [navSearch, setNavSearch] = useState('');
    const [navMenuOpen, setNavMenuOpen] = useState(false);
    const [sidebarActive, setSidebarActive] = useState('home');

    const menuRef = useClickOutside<HTMLDivElement>(menuOpen, () => setMenuOpen(false));
    const navMenuRef = useClickOutside<HTMLDivElement>(navMenuOpen, () => setNavMenuOpen(false));

    useEffect(() => {
        document.documentElement.classList.toggle('dark', dark);
    }, [dark]);

    const sections: SectionDef[] = [
        {
            id: 'colors',
            label: 'Colors',
            icon: 'palette-swatch',
            description: 'Theme color tokens - they adapt to light and dark mode.',
            render: () => (
                <div class="showcase-swatches">
                    {COLOR_TOKENS.map((token) => (
                        <div key={token} class="showcase-swatch">
                            <div class="showcase-swatch-box" style={`background-color: var(${token})`} />
                            <span>{token}</span>
                        </div>
                    ))}
                </div>
            ),
        },
        {
            id: 'typography',
            label: 'Typography',
            icon: 'format-letter-case',
            description: 'The sans-serif UI font and the monospace font.',
            render: () => (
                <div class="showcase-stack">
                    <div>
                        <p class="showcase-type-label">Sans-serif · --font-sans-serif</p>
                        <p class="showcase-type-sans">The quick brown fox jumps over the lazy dog</p>
                        <p class="showcase-type-sans is-bold">AaBbCcDdEe 0123456789</p>
                    </div>
                    <div>
                        <p class="showcase-type-label">Monospace · --font-monospace</p>
                        <p class="showcase-type-mono">const answer = 42; // the quick brown fox</p>
                    </div>
                </div>
            ),
        },
        {
            id: 'buttons',
            label: 'Buttons',
            icon: 'button-cursor',
            description: 'Primary, secondary, danger and icon button variants.',
            render: () => (
                <>
                    <div class="showcase-row">
                        <Button>Primary</Button>
                        <SecondaryButton>Secondary</SecondaryButton>
                        <DangerButton>Danger</DangerButton>
                        <Button disabled>Disabled</Button>
                    </div>
                    <div class="showcase-row" style="margin-top: 1rem;">
                        <Button>
                            <ContentSaveIcon class="is-sm" />
                            With icon
                        </Button>
                        <IconButton title="Edit">
                            <PencilIcon class="is-md" />
                        </IconButton>
                        <SmallIconButton title="Delete">
                            <DeleteOutlineIcon class="is-sm" />
                        </SmallIconButton>
                    </div>
                    <div class="showcase-row" style="margin-top: 1rem;">
                        <TextButton>Text button</TextButton>
                        <DangerTextButton>
                            <DeleteOutlineIcon class="is-sm" />
                            Danger text
                        </DangerTextButton>
                    </div>
                </>
            ),
        },
        {
            id: 'badges',
            label: 'Badges',
            icon: 'tag-outline',
            description: 'Small status labels, default and accent.',
            render: () => (
                <div class="showcase-row">
                    <Badge>Default</Badge>
                    <Badge accent>Accent</Badge>
                </div>
            ),
        },
        {
            id: 'inputs',
            label: 'Inputs',
            icon: 'form-textbox',
            description: 'Text input, select and the clearable search input.',
            render: () => (
                <div class="showcase-stack">
                    <FormInput
                        type="text"
                        value={text}
                        placeholder="Type something…"
                        onInput={(e) => setText((e.target as HTMLInputElement).value)}
                    />
                    <FormSelect value={select} onChange={(e) => setSelect((e.target as HTMLSelectElement).value)}>
                        <option value="note-text">Note</option>
                        <option value="account">Account</option>
                        <option value="cog">Settings</option>
                    </FormSelect>
                    <SearchInput
                        value={demoSearch}
                        onInput={setDemoSearch}
                        onClear={() => setDemoSearch('')}
                        placeholder="Search…"
                    />
                </div>
            ),
        },
        {
            id: 'forms',
            label: 'Forms',
            icon: 'clipboard-text-outline',
            description: 'Field labels with validation, messages and action rows.',
            render: () => (
                <div class="showcase-stack">
                    <FormField id="email" label="Email address">
                        <FormInput id="email" type="email" placeholder="you@example.com" />
                    </FormField>
                    <FormField id="password" label="Password" error="Password is too short">
                        <FormInput id="password" type="password" value="123" />
                    </FormField>
                    <FormMessage type="success" message="Your changes have been saved." />
                    <FormMessage type="error" message="Something went wrong." />
                    <FormActions>
                        <SecondaryButton>Cancel</SecondaryButton>
                        <Button>Save</Button>
                    </FormActions>
                </div>
            ),
        },
        {
            id: 'cards',
            label: 'Cards',
            icon: 'card-text-outline',
            description: 'A simple padded surface container.',
            render: () => (
                <div class="showcase-stack">
                    <Card>
                        <h3 style="font-weight: 600; margin-bottom: 0.5rem;">Card title</h3>
                        <p style="color: var(--color-text-soft);">Cards group related content on a raised surface.</p>
                    </Card>
                </div>
            ),
        },
        {
            id: 'table',
            label: 'Table',
            icon: 'table',
            description: 'A data table with responsive columns and row actions.',
            render: () => (
                <Table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th class="col-hide-sm">Email</th>
                            <th class="cell-actions">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {[
                            { name: 'Alice Appel', email: 'alice@example.com', admin: true },
                            { name: 'Bob Bakker', email: 'bob@example.com', admin: false },
                            { name: 'Carol Croket', email: 'carol@example.com', admin: false },
                        ].map((row) => (
                            <tr key={row.email}>
                                <td>
                                    <div class="cell-name">
                                        <Avatar>
                                            {row.name[0]}
                                            {row.name.split(' ')[1][0]}
                                        </Avatar>
                                        <span class="cell-name-text">{row.name}</span>
                                        {row.admin && <Badge accent>Admin</Badge>}
                                    </div>
                                </td>
                                <td class="col-hide-sm has-text-muted">{row.email}</td>
                                <td>
                                    <div class="table-actions">
                                        <SmallIconButton title="Edit">
                                            <PencilIcon class="is-sm" />
                                        </SmallIconButton>
                                        <SmallIconButton title="Delete" class="hover-danger">
                                            <DeleteOutlineIcon class="is-sm" />
                                        </SmallIconButton>
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </Table>
            ),
        },
        {
            id: 'dialogs',
            label: 'Dialogs',
            icon: 'message-text-outline',
            description: 'Plain, form and type-to-confirm destructive dialogs.',
            render: () => (
                <div class="showcase-row">
                    <Button onClick={() => setDialogOpen(true)}>Open dialog</Button>
                    <SecondaryButton onClick={() => setFormDialogOpen(true)}>
                        <PencilIcon class="is-sm" />
                        Open form dialog
                    </SecondaryButton>
                    <DangerButton onClick={() => setConfirmOpen(true)}>
                        <DeleteOutlineIcon class="is-sm" />
                        Delete item
                    </DangerButton>
                </div>
            ),
        },
        {
            id: 'empty-state',
            label: 'Empty state',
            icon: 'tray',
            description: 'A centered placeholder shown when there is no content.',
            render: () => (
                <div class="showcase-empty-demo">
                    <EmptyState icon={<MagnifyIcon class="is-huge" />} message="No results found" />
                </div>
            ),
        },
        {
            id: 'loading',
            label: 'Loading',
            icon: 'progress-clock',
            description: 'A centered loading-text placeholder.',
            render: () => <LoadingText>Loading notes...</LoadingText>,
        },
        {
            id: 'fab',
            label: 'FAB',
            icon: 'plus-circle',
            description: 'A floating action button, normally fixed to the corner of the screen.',
            render: () => (
                <div class="showcase-fab-demo">
                    <Fab title="Create">
                        <PlusIcon class="is-xl" />
                    </Fab>
                </div>
            ),
        },
        {
            id: 'icons',
            label: 'Icons',
            icon: 'material-design',
            description: 'All Material Design icons bundled with PlaatUI.',
            render: () => (
                <div class="showcase-icons">
                    {iconEntries.map(([name, CatalogIcon]) => (
                        <div key={name} class="showcase-icon">
                            <CatalogIcon class="is-lg" />
                            <span>{name}</span>
                        </div>
                    ))}
                </div>
            ),
        },
        {
            id: 'navbar',
            label: 'Navbar & menu',
            icon: 'page-layout-header',
            description: 'A real navbar with brand, centered search and a working user dropdown.',
            render: () => (
                <div class="showcase-navbar-demo">
                    <Navbar>
                        <NavbarBrand image={BRAND_IMAGE} name="PlaatUI" href="#navbar" />
                        <NavbarSpacer />
                        <NavbarSearch>
                            <SearchInput
                                value={navSearch}
                                onInput={setNavSearch}
                                onClear={() => setNavSearch('')}
                                placeholder="Search…"
                            />
                        </NavbarSearch>
                        <NavbarMenu ref={navMenuRef}>
                            <NavbarUserButton onClick={() => setNavMenuOpen((o) => !o)}>
                                <Avatar>BP</Avatar>
                                <NavbarUserName>Bastiaan</NavbarUserName>
                            </NavbarUserButton>
                            {navMenuOpen && (
                                <DropdownMenu>
                                    <DropdownItem onClick={() => setNavMenuOpen(false)}>
                                        <AccountIcon class="is-sm" />
                                        Profile
                                    </DropdownItem>
                                    <DropdownItem onClick={() => setNavMenuOpen(false)}>
                                        <CogIcon class="is-sm" />
                                        Settings
                                    </DropdownItem>
                                    <DropdownDivider />
                                    <DropdownItem onClick={() => setNavMenuOpen(false)}>
                                        <LogoutIcon class="is-sm" />
                                        Logout
                                    </DropdownItem>
                                </DropdownMenu>
                            )}
                        </NavbarMenu>
                    </Navbar>
                </div>
            ),
        },
        {
            id: 'sidebar',
            label: 'Sidebar',
            icon: 'page-layout-sidebar-left',
            description: 'The SidebarLink navigation - also the live shell of this page. Click an item below.',
            render: () => (
                <div class="showcase-sidebar-demo">
                    <aside class="sidebar">
                        <nav class="sidebar-nav">
                            {SIDEBAR_DEMO.map((item) => (
                                <SidebarLink
                                    key={item.id}
                                    href="#sidebar"
                                    label={item.label}
                                    icon={iconCatalog[item.icon]}
                                    active={sidebarActive === item.id}
                                    onClick={(e) => {
                                        e.preventDefault();
                                        setSidebarActive(item.id);
                                    }}
                                />
                            ))}
                        </nav>
                    </aside>
                    <div class="showcase-sidebar-demo-content">
                        <p class="section-label">Selected</p>
                        <p>{SIDEBAR_DEMO.find((item) => item.id === sidebarActive)?.label}</p>
                    </div>
                </div>
            ),
        },
    ];

    const q = query.trim().toLowerCase();
    const visible = sections.filter((s) => q === '' || `${s.label} ${s.description} ${s.id}`.toLowerCase().includes(q));

    const navbar = (
        <Navbar>
            <NavbarBrand image={BRAND_IMAGE} name="PlaatUI" href="#" />
            <NavbarSpacer />
            <NavbarSearch>
                <SearchInput
                    value={query}
                    onInput={setQuery}
                    onClear={() => setQuery('')}
                    placeholder="Search components…"
                />
            </NavbarSearch>
            <IconButton onClick={toggleTheme} title="Toggle theme">
                {renderIcon(dark ? 'weather-night' : 'weather-sunny', 'is-md')}
            </IconButton>
            <NavbarMenu ref={menuRef}>
                <NavbarUserButton onClick={() => setMenuOpen((o) => !o)}>
                    <Avatar>BP</Avatar>
                    <NavbarUserName>Bastiaan</NavbarUserName>
                </NavbarUserButton>
                {menuOpen && (
                    <DropdownMenu>
                        <DropdownItem onClick={() => setMenuOpen(false)}>
                            <AccountIcon class="is-sm" />
                            Profile
                        </DropdownItem>
                        <DropdownItem onClick={() => setMenuOpen(false)}>
                            <CogIcon class="is-sm" />
                            Settings
                        </DropdownItem>
                        <DropdownDivider />
                        <DropdownItem onClick={() => setMenuOpen(false)}>
                            <LogoutIcon class="is-sm" />
                            Logout
                        </DropdownItem>
                    </DropdownMenu>
                )}
            </NavbarMenu>
        </Navbar>
    );

    const sidebar = visible.map((s) => (
        <SidebarLink
            key={s.id}
            href={`#${s.id}`}
            label={s.label}
            icon={iconCatalog[s.icon]}
            active={active === s.id}
            onClick={() => setActive(s.id)}
        />
    ));

    return (
        <SidebarLayout navbar={navbar} sidebar={sidebar} version={__APP_VERSION__}>
            <Page>
                <PageTitle>PlaatUI component showcase</PageTitle>

                {visible.length === 0 ? (
                    <EmptyState icon={<MagnifyIcon class="is-huge" />} message={`No components match "${query}"`} />
                ) : (
                    visible.map((s) => (
                        <Section key={s.id} id={s.id} title={s.label} description={s.description}>
                            {s.render()}
                        </Section>
                    ))
                )}
            </Page>

            {dialogOpen && (
                <Dialog title="Example dialog" onClose={() => setDialogOpen(false)}>
                    <p class="modal-text">
                        This is a basic modal dialog. Press escape, click the backdrop or the close button to dismiss
                        it.
                    </p>
                    <FormActions flush>
                        <Button onClick={() => setDialogOpen(false)}>Got it</Button>
                    </FormActions>
                </Dialog>
            )}
            {formDialogOpen && (
                <Dialog title="Create note" onClose={() => setFormDialogOpen(false)}>
                    <Form
                        onSubmit={(event) => {
                            event.preventDefault();
                            setFormDialogOpen(false);
                        }}
                    >
                        <FormField id="dialog-title" label="Title">
                            <FormInput id="dialog-title" type="text" placeholder="My note" />
                        </FormField>
                        <FormField id="dialog-tag" label="Tag">
                            <FormSelect id="dialog-tag">
                                <option>Personal</option>
                                <option>Work</option>
                                <option>Ideas</option>
                            </FormSelect>
                        </FormField>
                        <FormActions flush>
                            <SecondaryButton type="button" onClick={() => setFormDialogOpen(false)}>
                                Cancel
                            </SecondaryButton>
                            <Button type="submit">
                                <ContentSaveIcon class="is-sm" />
                                Save
                            </Button>
                        </FormActions>
                    </Form>
                </Dialog>
            )}
            {confirmOpen && (
                <ConfirmDialog
                    title="Delete item"
                    message="This action cannot be undone. Type the name to confirm."
                    confirmLabel="Delete"
                    cancelLabel="Cancel"
                    confirmText="my-item"
                    typeToConfirmLabel={(name) => `Type "${name}" to confirm`}
                    onConfirm={() => setConfirmOpen(false)}
                    onClose={() => setConfirmOpen(false)}
                />
            )}
        </SidebarLayout>
    );
}
