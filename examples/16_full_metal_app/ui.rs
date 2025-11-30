//! UI drawing functions

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]

use screencapturekit::prelude::*;

use crate::font::BitmapFont;
use crate::overlay::ConfigMenu;
use crate::vertex::VertexBufferBuilder;

// Synthwave color constants
const NEON_PINK: [f32; 4] = [1.0, 0.2, 0.6, 1.0];
const NEON_CYAN: [f32; 4] = [0.0, 1.0, 0.9, 1.0];
#[allow(dead_code)]
const NEON_PURPLE: [f32; 4] = [0.7, 0.3, 1.0, 1.0];
const NEON_YELLOW: [f32; 4] = [1.0, 0.95, 0.3, 1.0];
const DARK_BG: [f32; 4] = [0.04, 0.02, 0.08, 0.95];

impl VertexBufferBuilder {
    pub fn help_overlay(
        &mut self,
        font: &BitmapFont,
        vw: f32,
        vh: f32,
        is_capturing: bool,
        is_recording: bool,
        source_name: &str,
        menu_selection: usize,
        menu_items: &[&str],
    ) {
        let base_scale = (vw.min(vh) / 800.0).clamp(0.8, 2.0);
        let scale = 1.5 * base_scale;
        let line_h = 18.0 * base_scale;
        let padding = 16.0 * base_scale;
        let has_source = !source_name.is_empty() && source_name != "None";

        // Determine menu mode from items
        let is_initial = menu_items.len() == 2 && menu_items[0] == "Pick Source";

        // Generate values for each menu item based on current state
        let menu_values: Vec<&str> = menu_items
            .iter()
            .map(|&item| match item {
                "Capture" => {
                    if is_capturing {
                        "Stop"
                    } else {
                        "Start"
                    }
                }
                "Screenshot" => "Take",
                "Record" => {
                    if is_recording {
                        "Stop"
                    } else {
                        "Start"
                    }
                }
                "Config" | "Rec Config" => "Open",
                _ => "", // Pick Source, Change Source, Quit
            })
            .collect();

        let item_count = menu_items.len() as f32;
        let box_w = (320.0 * base_scale).min(vw * 0.8);
        let box_h = (line_h * (item_count + 2.5) + padding * 2.0).min(vh * 0.75);
        let x = (vw - box_w) / 2.0;
        let y = (vh - box_h) / 2.0;

        // Title above menu
        let (title_text, title_color): (String, [f32; 4]) = if is_initial {
            ("Select a Source to Begin".to_string(), [0.6, 0.5, 0.7, 1.0])
        } else if has_source {
            let display = if source_name.len() > 30 {
                format!("{}...", &source_name.chars().take(27).collect::<String>())
            } else {
                source_name.to_string()
            };
            (display, NEON_CYAN)
        } else {
            ("No Source Selected".to_string(), [0.5, 0.4, 0.6, 1.0])
        };

        let title_scale = scale * 1.4;
        let title_actual = (title_scale as i32) as f32;
        let title_w = title_text.len() as f32 * 8.0 * title_actual;
        let title_x = (vw - title_w) / 2.0;
        let title_y = y - line_h * 2.2;
        self.text(
            font,
            &title_text,
            title_x,
            title_y,
            title_scale,
            title_color,
        );

        // Dark purple background with neon border
        self.rect(x, y, box_w, box_h, DARK_BG);
        self.rect_outline(x, y, box_w, box_h, 2.0, NEON_PINK);
        self.rect_outline(
            x + 1.0,
            y + 1.0,
            box_w - 2.0,
            box_h - 2.0,
            1.0,
            [0.3, 0.1, 0.4, 0.5],
        );

        let mut ly = y + padding;
        let text_x = 12.0f32.mul_add(base_scale, x + padding);

        let actual_scale = (scale as i32) as f32;
        let text_h = 8.0 * actual_scale;

        for (i, (item, value)) in menu_items.iter().zip(menu_values.iter()).enumerate() {
            let is_selected = i == menu_selection;
            let text_y = ly + (line_h - text_h) / 2.0;

            if is_selected {
                // Selection highlight - purple glow
                self.rect(x + 3.0, ly, box_w - 6.0, line_h, [0.15, 0.05, 0.25, 0.9]);
                self.rect(x + 3.0, ly, 2.0, line_h, NEON_PINK);
                self.text(font, ">", x + padding * 0.5, text_y, scale, NEON_YELLOW);
            }

            let item_color = if is_selected {
                NEON_CYAN
            } else {
                [0.8, 0.8, 0.9, 1.0]
            };

            self.text(font, item, text_x, text_y, scale, item_color);

            if !value.is_empty() {
                let vx = (value.len() as f32 * 8.0).mul_add(-actual_scale, x + box_w - padding);
                let val_color = if is_selected {
                    NEON_YELLOW
                } else {
                    [0.5, 0.5, 0.6, 1.0]
                };
                self.text(font, value, vx, text_y, scale, val_color);
            }
            ly += line_h;
        }

        // Footer
        ly += line_h * 0.2;
        self.rect(
            x + padding,
            ly,
            box_w - padding * 2.0,
            1.0,
            [0.3, 0.15, 0.4, 0.4],
        );
        ly += line_h * 0.4;
        self.text(
            font,
            "ARROWS  ENTER  ESC",
            text_x,
            ly,
            scale * 0.6,
            [0.5, 0.4, 0.6, 1.0],
        );
    }

    pub fn config_menu(
        &mut self,
        font: &BitmapFont,
        vw: f32,
        vh: f32,
        config: &SCStreamConfiguration,
        mic_device_idx: Option<usize>,
        selection: usize,
        is_capturing: bool,
        source_name: &str,
    ) {
        let base_scale = (vw.min(vh) / 800.0).clamp(0.8, 2.0);
        let scale = 1.5 * base_scale;
        let line_h = 18.0 * base_scale;
        let padding = 16.0 * base_scale;
        let option_count = ConfigMenu::option_count();
        let box_w = (340.0 * base_scale).min(vw * 0.85);
        let box_h = (line_h * (option_count as f32 + 5.0) + padding * 2.0).min(vh * 0.8);
        let x = (vw - box_w) / 2.0;
        let y = (vh - box_h) / 2.0;

        // Dark purple background with neon border
        self.rect(x, y, box_w, box_h, DARK_BG);
        self.rect_outline(x, y, box_w, box_h, 2.0, NEON_CYAN);
        self.rect_outline(
            x + 1.0,
            y + 1.0,
            box_w - 2.0,
            box_h - 2.0,
            1.0,
            [0.1, 0.3, 0.4, 0.5],
        );

        let mut ly = y + padding;
        let text_x = 12.0f32.mul_add(base_scale, x + padding);

        // Source heading (larger, centered)
        let source_display = if source_name.is_empty() || source_name == "None" {
            "No Source"
        } else {
            source_name
        };
        let source_w = source_display.len() as f32 * 8.0 * scale;
        let source_x = x + (box_w - source_w) / 2.0;
        self.text(font, source_display, source_x, ly, scale * 1.1, NEON_YELLOW);
        ly += line_h * 1.5;

        // Separator line
        self.rect(
            x + padding,
            ly - 4.0,
            box_w - padding * 2.0,
            1.0,
            NEON_PURPLE,
        );
        ly += line_h * 0.3;

        // Title row with live indicator
        self.text(font, "CONFIG", text_x - 4.0, ly, scale * 0.8, NEON_PINK);

        // Live indicator
        if is_capturing {
            let live_x = 32.0f32.mul_add(-base_scale, x + box_w - padding);
            self.rect(
                live_x - 3.0,
                ly - 1.0,
                38.0 * base_scale,
                line_h * 0.9,
                [0.5, 0.1, 0.15, 0.9],
            );
            self.text(font, "LIVE", live_x, ly, scale * 0.7, [1.0, 0.3, 0.3, 1.0]);
        }

        ly += line_h * 1.0;

        let actual_scale = (scale as i32) as f32;
        let text_h = 8.0 * actual_scale;

        for i in 0..option_count {
            let is_selected = i == selection;
            let text_y = ly + (line_h - text_h) / 2.0;

            if is_selected {
                self.rect(x + 3.0, ly, box_w - 6.0, line_h, [0.1, 0.05, 0.2, 0.9]);
                self.rect(x + 3.0, ly, 2.0, line_h, NEON_CYAN);
                self.text(font, ">", x + padding * 0.5, text_y, scale, NEON_YELLOW);
            }

            let name = ConfigMenu::option_name(i);
            let value = ConfigMenu::option_value(config, mic_device_idx, i);

            let name_color = if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.7, 0.7, 0.8, 1.0]
            };
            self.text(font, name, text_x, text_y, scale, name_color);

            let t: String = if value.len() > 12 {
                format!("{}...", &value.chars().take(9).collect::<String>())
            } else {
                value
            };
            let vx = (t.len() as f32 * 8.0).mul_add(-actual_scale, x + box_w - padding);

            let value_color = if is_selected {
                if t == "On" {
                    [0.3, 1.0, 0.5, 1.0]
                } else if t == "Off" {
                    [1.0, 0.4, 0.4, 1.0]
                } else {
                    NEON_YELLOW
                }
            } else if t == "On" {
                [0.2, 0.7, 0.4, 1.0]
            } else if t == "Off" {
                [0.5, 0.3, 0.3, 1.0]
            } else {
                [0.5, 0.5, 0.6, 1.0]
            };
            self.text(font, &t, vx, text_y, scale, value_color);
            ly += line_h;
        }

        // Footer
        ly += line_h * 0.2;
        self.rect(
            x + padding,
            ly,
            box_w - padding * 2.0,
            1.0,
            [0.3, 0.15, 0.4, 0.4],
        );
        ly += line_h * 0.4;
        let hint = if is_capturing {
            "L/R  ENTER=Apply  ESC"
        } else {
            "LEFT/RIGHT  ESC"
        };
        self.text(font, hint, text_x, ly, scale * 0.6, [0.5, 0.4, 0.6, 1.0]);
    }

    #[cfg(feature = "macos_15_0")]
    pub fn recording_config_menu(
        &mut self,
        font: &BitmapFont,
        vw: f32,
        vh: f32,
        config: &crate::recording::RecordingConfig,
        selection: usize,
    ) {
        use crate::recording::RecordingConfigMenu;

        let base_scale = (vw.min(vh) / 800.0).clamp(0.8, 2.0);
        let scale = 1.5 * base_scale;
        let line_h = 18.0 * base_scale;
        let padding = 16.0 * base_scale;
        let option_count = RecordingConfigMenu::option_count();
        let box_w = (280.0 * base_scale).min(vw * 0.7);
        let box_h = (line_h * (option_count as f32 + 4.0) + padding * 2.0).min(vh * 0.6);
        let x = (vw - box_w) / 2.0;
        let y = (vh - box_h) / 2.0;

        // Dark purple background with neon border
        self.rect(x, y, box_w, box_h, DARK_BG);
        self.rect_outline(x, y, box_w, box_h, 2.0, NEON_PINK);
        self.rect_outline(
            x + 1.0,
            y + 1.0,
            box_w - 2.0,
            box_h - 2.0,
            1.0,
            [0.4, 0.1, 0.3, 0.5],
        );

        let mut ly = y + padding;
        let text_x = 12.0f32.mul_add(base_scale, x + padding);

        // Title
        self.text(font, "RECORDING", text_x - 4.0, ly, scale * 0.9, NEON_PINK);
        ly += line_h * 1.2;

        // Separator line
        self.rect(
            x + padding,
            ly - 4.0,
            box_w - padding * 2.0,
            1.0,
            NEON_PURPLE,
        );
        ly += line_h * 0.3;

        let actual_scale = (scale as i32) as f32;
        let text_h = 8.0 * actual_scale;

        for i in 0..option_count {
            let is_selected = i == selection;
            let text_y = ly + (line_h - text_h) / 2.0;
            let item = RecordingConfigMenu::option_name(i);
            let value = RecordingConfigMenu::option_value(config, i);

            if is_selected {
                // Selection highlight
                self.rect(x + 3.0, ly, box_w - 6.0, line_h, [0.25, 0.05, 0.15, 0.9]);
                self.rect(x + 3.0, ly, 2.0, line_h, NEON_PINK);
                self.text(font, ">", x + padding * 0.5, text_y, scale, NEON_YELLOW);
            }

            let item_color = if is_selected {
                NEON_CYAN
            } else {
                [0.8, 0.8, 0.9, 1.0]
            };

            self.text(font, item, text_x, text_y, scale, item_color);

            let vx = (value.len() as f32 * 8.0).mul_add(-actual_scale, x + box_w - padding);
            let value_color = if is_selected {
                NEON_YELLOW
            } else {
                [0.5, 0.5, 0.6, 1.0]
            };
            self.text(font, &value, vx, text_y, scale, value_color);
            ly += line_h;
        }

        // Footer
        ly += line_h * 0.2;
        self.rect(
            x + padding,
            ly,
            box_w - padding * 2.0,
            1.0,
            [0.3, 0.15, 0.4, 0.4],
        );
        ly += line_h * 0.4;
        self.text(
            font,
            "LEFT/RIGHT  ESC",
            text_x,
            ly,
            scale * 0.6,
            [0.5, 0.4, 0.6, 1.0],
        );
    }
}
