pub mod database;
pub mod notification;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod tray;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod webserver;
