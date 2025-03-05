// Updated notifications.rs with correct function signature
use log::{info, error};
use std::process::Command;
use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
}

#[cfg(target_os = "windows")]
pub fn show_windows_notification(title: &str, message: &str) -> Result<(), String> {
    info!("Windows-Benachrichtigung anzeigen: {} - {}", title, message);
    
    // PowerShell-Befehl zum Anzeigen einer Windows-Benachrichtigung
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
            error!("Fehler beim Anzeigen der Windows-Benachrichtigung: {}", e);
            Err(format!("Fehler beim Anzeigen der Benachrichtigung: {}", e))
        }
    }
}

#[cfg(target_os = "linux")]
pub fn show_linux_notification(title: &str, message: &str) -> Result<(), String> {
    info!("Linux-Benachrichtigung anzeigen: {} - {}", title, message);
    
    match Command::new("notify-send")
        .args(&[title, message])
        .output() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Fehler beim Anzeigen der Linux-Benachrichtigung: {}", e);
            Err(format!("Fehler beim Anzeigen der Benachrichtigung: {}", e))
        }
    }
}

#[cfg(target_os = "macos")]
pub fn show_macos_notification(title: &str, message: &str) -> Result<(), String> {
    info!("MacOS-Benachrichtigung anzeigen: {} - {}", title, message);
    
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
            error!("Fehler beim Anzeigen der MacOS-Benachrichtigung: {}", e);
            Err(format!("Fehler beim Anzeigen der Benachrichtigung: {}", e))
        }
    }
}

// Plattformübergreifende Benachrichtigungsfunktion
pub fn show_notification(title: &str, message: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return show_windows_notification(title, message);
    
    #[cfg(target_os = "linux")]
    return show_linux_notification(title, message);
    
    #[cfg(target_os = "macos")]
    return show_macos_notification(title, message);
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        error!("Benachrichtigungen werden auf dieser Plattform nicht unterstützt");
        Err("Benachrichtigungen werden auf dieser Plattform nicht unterstützt".to_string())
    }
}

// Direct tauri command to show notification
#[tauri::command]
pub fn show_notification_command(notification: NotificationPayload) -> Result<(), String> {
    info!("Notification command received: {} - {}", notification.title, notification.body);
    show_notification(&notification.title, &notification.body)
}

// Keep the same signature but change the implementation to work with immutable reference
pub fn setup_notification_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Get the app handle and use that instead of the mutable app reference
    let app_handle = app.handle();
    
    // Now use app_handle for anything needed
    if let Some(window) = app_handle.get_webview_window("main") {
        info!("Notification system initialized for window: {}", window.label());
    } else {
        info!("Notification system initialized (no main window found)");
    }
    
    Ok(())
}