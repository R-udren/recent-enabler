pub mod error;
pub mod recent;
pub mod service;
pub mod status;
pub mod sysmain;
pub mod system_restore;
pub mod utils;

// Public, stable-ish API surface for consumers (UI / other crates)

pub use crate::service::{
    check_recent, check_sysmain, check_system_restore, enable_recent, enable_sysmain,
    enable_system_restore,
};

pub use crate::status::{RecentStatus, SysMainStatus, SystemRestoreStatus};

pub use crate::error::{RecentEnablerError, Result};

pub use crate::utils::{is_admin, restart_as_admin};

pub mod prelude {
    pub use crate::error::{RecentEnablerError, Result};
    pub use crate::service::{
        check_recent, check_sysmain, check_system_restore, enable_recent, enable_sysmain,
        enable_system_restore,
    };
    pub use crate::status::{RecentStatus, SysMainStatus, SystemRestoreStatus};
    pub use crate::utils::{is_admin, restart_as_admin};
}
