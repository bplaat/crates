-- Copyright (c) 2026 Bastiaan van der Plaat
-- SPDX-License-Identifier: MIT

UPDATE users SET email = lower(trim(email));
CREATE UNIQUE INDEX idx_users_email_nocase ON users (email COLLATE NOCASE);
