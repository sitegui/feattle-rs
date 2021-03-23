use chrono::{DateTime, Utc};

/// Store details of the last time the data was synchronized by calling
/// [`crate::Feattles::reload()`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LastReload {
    /// The data was never updated and all feattles carry their default values.
    Never,
    /// The reload finished with success, but no data was found. All feattle carry their default
    /// values.
    NoData { reload_date: DateTime<Utc> },
    /// The reload finished with success.
    Data {
        reload_date: DateTime<Utc>,
        version: i32,
        version_date: DateTime<Utc>,
    },
}

impl LastReload {
    /// Indicate when, if ever, a reload finished with success.
    pub fn reload_date(self) -> Option<DateTime<Utc>> {
        match self {
            LastReload::Never => None,
            LastReload::NoData { reload_date, .. } | LastReload::Data { reload_date, .. } => {
                Some(reload_date)
            }
        }
    }

    /// Indicate which is, if any, the current data version. Note that the value `0` is used for
    /// [`LastReload::NoData`].
    pub fn version(self) -> Option<i32> {
        match self {
            LastReload::Never => None,
            LastReload::NoData { .. } => Some(0),
            LastReload::Data { version, .. } => Some(version),
        }
    }

    /// Indicate when, if known, this data version was created.
    pub fn version_date(self) -> Option<DateTime<Utc>> {
        match self {
            LastReload::Never | LastReload::NoData { .. } => None,
            LastReload::Data { version_date, .. } => Some(version_date),
        }
    }
}
