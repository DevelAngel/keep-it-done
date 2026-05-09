use strum::{Display, EnumIter, EnumString, IntoStaticStr};

/// URL-slug identifiers for the five task-list views.
///
/// Single source of truth consumed by `app::View` and end-to-end tests.
/// The serialised form (via `strum`) is the `?view=` query-parameter value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash,
         Display, EnumIter, EnumString, IntoStaticStr)]
pub enum ViewSlug {
    #[strum(serialize = "upcoming")]
    Upcoming,
    #[strum(serialize = "quickwins")]
    QuickWins,
    #[strum(serialize = "allopen")]
    AllOpen,
    #[strum(serialize = "finished")]
    WhatIFinished,
    #[strum(serialize = "recent")]
    RecentlyChanged,
}
