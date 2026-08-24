/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use from_derive::{FromEnum, FromStruct};

enum ExternalState {
    Ready,
    Done,
}

#[derive(FromEnum)]
#[from_enum(ExternalState)]
enum State {
    Ready,
    Done,
}

struct ExternalRecord<T>
where
    T: Clone,
{
    value: T,
}

#[derive(FromStruct)]
#[from_struct(ExternalRecord)]
struct Record<T>
where
    T: Clone,
{
    value: T,
}

fn main() {
    let external: ExternalState = State::Ready.into();
    let _: State = external.into();

    let external: ExternalRecord<String> = Record {
        value: String::new(),
    }
    .into();
    let _: Record<String> = external.into();
}
