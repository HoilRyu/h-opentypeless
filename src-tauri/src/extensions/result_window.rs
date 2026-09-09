//! Physical coordinates keep the result beside the capsule across mixed-DPI screens.
use tauri::Manager;

#[cfg(target_os = "macos")]
mod panel {
    use tauri::Manager;
    use tauri_nspanel::{tauri_panel, ManagerExt, WebviewWindowExt};

    tauri_panel! {
        panel!(HResultPanel {
            config: {
                can_become_key_window: false,
                can_become_main_window: false,
                is_floating_panel: true
            }
        })
    }

    // Conversion and all AppKit mutations run on the runtime's main thread.
    // Keep the panel for the lifetime of the reusable ask window.
    pub fn show(window: &tauri::WebviewWindow) -> tauri::Result<()> {
        let w = window.clone();
        window.run_on_main_thread(move || {
            let panel = match w.app_handle().get_webview_panel(w.label()) {
                Ok(panel) => panel,
                Err(_) => match w.to_panel::<HResultPanel>() {
                    Ok(panel) => {
                        use tauri_nspanel::objc2_app_kit::NSWindowStyleMask;
                        panel.set_style_mask(NSWindowStyleMask::NonactivatingPanel);
                        panel.set_hides_on_deactivate(false);
                        panel.set_becomes_key_only_if_needed(true);
                        panel
                    }
                    Err(error) => {
                        tracing::error!("Could not create nonactivating result panel: {error}");
                        let _ = w.show();
                        return;
                    }
                },
            };
            panel.show();
        })
    }
}

pub fn show(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        panel::show(window)
    }
    #[cfg(not(target_os = "macos"))]
    {
        window.unminimize()?;
        window.show()?;
        window.set_focus()
    }
}

fn adjacent(
    anchor: (f64, f64, f64, f64),
    size: (f64, f64),
    area: (f64, f64, f64, f64),
    gap: f64,
) -> (i32, i32) {
    let (ax, ay, aw, ah) = anchor;
    let (x, y, width, height) = area;
    let left = x + gap;
    let top = y + gap;
    let right = (x + width - size.0 - gap).max(left);
    let bottom = (y + height - size.1 - gap).max(top);
    let above = ay - size.1 - gap;
    let preferred_y = if above >= top { above } else { ay + ah + gap };
    (
        ((ax + aw / 2.0 - size.0 / 2.0).clamp(left, right)).round() as i32,
        preferred_y.clamp(top, bottom).round() as i32,
    )
}

pub fn position(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let Some(capsule) = app.get_webview_window("capsule") else {
        return Ok(());
    };
    let Some(monitor) = capsule.current_monitor()? else {
        return Ok(());
    };
    let anchor = capsule.outer_position()?;
    let anchor_size = capsule.outer_size()?;
    let logical = window
        .outer_size()?
        .to_logical::<f64>(window.scale_factor()?);
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let (x, y) = adjacent(
        (
            anchor.x as f64,
            anchor.y as f64,
            anchor_size.width as f64,
            anchor_size.height as f64,
        ),
        (logical.width * scale, logical.height * scale),
        (
            area.position.x as f64,
            area.position.y as f64,
            area.size.width as f64,
            area.size.height as f64,
        ),
        10.0 * scale,
    );
    window.set_position(tauri::PhysicalPosition::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefers_above_and_switches_below_at_top() {
        assert_eq!(
            adjacent(
                (400., 600., 200., 80.),
                (400., 220.),
                (0., 0., 1000., 800.),
                10.
            ),
            (300, 370)
        );
        assert_eq!(
            adjacent(
                (400., 0., 200., 80.),
                (400., 220.),
                (0., 0., 1000., 800.),
                10.
            ),
            (300, 90)
        );
    }
    #[test]
    fn clamps_to_negative_coordinate_monitor_at_double_scale() {
        assert_eq!(
            adjacent(
                (-1950., 1400., 400., 160.),
                (800., 440.),
                (-2000., 0., 2000., 1600.),
                20.
            ),
            (-1980, 940)
        );
        let (x, y) = adjacent(
            (1990., 790., 200., 80.),
            (400., 220.),
            (1000., 0., 1000., 800.),
            10.,
        );
        assert_eq!((x, y), (1590, 560));
    }
}
