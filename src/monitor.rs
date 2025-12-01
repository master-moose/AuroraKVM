use crate::config::LocalScreen;
use display_info::DisplayInfo;

/// Detect all connected monitors and return their screen information
pub fn detect_monitors() -> Vec<LocalScreen> {
    match DisplayInfo::all() {
        Ok(displays) => {
            if displays.is_empty() {
                // Fallback to default if no displays detected
                return vec![LocalScreen {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                }];
            }

            displays
                .iter()
                .map(|display| LocalScreen {
                    x: display.x,
                    y: display.y,
                    width: display.width,
                    height: display.height,
                })
                .collect()
        }
        Err(e) => {
            eprintln!("Failed to detect monitors: {}, using default", e);
            vec![LocalScreen {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            }]
        }
    }
}
