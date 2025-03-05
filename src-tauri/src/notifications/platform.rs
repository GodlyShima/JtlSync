use log::{info, error};
use std::process::Command;

use crate::error::{Result, Error};

#[cfg(target_os = "windows")]
pub fn show_notification(title: &str, message: &str) -> Result<()> {
    info!("Showing Windows notification: {} - {}", title, message);
    
    // PowerShell command to show a Windows notification
    let ps_script = format!(
        r#"
        [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
        [Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
        [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

        $APP_ID = "JTLSync"

        $template = @"
        <toast>
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>
        </toast>
        "@

        $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
        $xml.LoadXml($template)
        $toast = New-Object Windows.UI.Notifications.ToastNotification $xml
        [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($APP_ID).Show($toast)
        "#,
        title, message
    );

    match Command::new("powershell")
        .args(&["-Command", &ps_script])
        .output() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error showing Windows notification: {}", e);
            Err(Error::System(format!("Failed to show notification: {}", e)))
        }
    }
}

#[cfg(target_os = "linux")]
pub fn show_notification(title: &str, message: &str) -> Result<()> {
    info!("Showing Linux notification: {} - {}", title, message);
    
    match Command::new("notify-send")
        .args(&[title, message])
        .output() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error showing Linux notification: {}", e);
            Err(Error::System(format!("Failed to show notification: {}", e)))
        }
    }
}

#[cfg(target_os = "macos")]
pub fn show_notification(title: &str, message: &str) -> Result<()> {
    info!("Showing macOS notification: {} - {}", title, message);
    
    let apple_script = format!(
        r#"display notification "{}" with title "{}""#,
        message.replace("\"", "\\\""),
        title.replace("\"", "\\\"")
    );
    
    match Command::new("osascript")
        .args(&["-e", &apple_script])
        .output() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error showing macOS notification: {}", e);
            Err(Error::System(format!("Failed to show notification: {}", e)))
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn show_notification(title: &str, message: &str) -> Result<()> {
    error!("Notifications not supported on this platform");
    Err(Error::System("Notifications not supported on this platform".to_string()))
}