//! Outbound adapters to the PVsyst executables: the documented CLI and the
//! Win32 desktop GUI.

pub(crate) mod cli;
pub(crate) mod gui;
pub(crate) mod http;
pub(crate) mod ledger;
#[cfg(windows)]
pub(crate) mod uia;
