use egui::{Color32, Rect, Sense, Stroke, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Square {
    position: egui::Pos2,
    dimension: egui::Vec2,
}

impl Square {
    fn new() -> Self {
        Self {
            position: egui::pos2(10.0, 10.0),
            dimension: egui::vec2(200.0, 75.0),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    squares: Vec<Square>,
    label: String,
    canvas_size: Vec2,
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            squares: vec![Square::new()],
            canvas_size: Vec2::splat(500.0),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl egui::Widget for &mut TemplateApp {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Frame::canvas(ui.style())
            .show(ui, |ui| {
                egui::ScrollArea::new([true; 2]).show(ui, |ui| {
                    //draw shapes (instead of an enum this could be trait objects or whatever really)
                    for square in self.squares.iter_mut() {
                        square
                    }

                    // for shape in self.shapes.iter_mut() {
                    //     match shape {
                    //         Shapes::Arrow(arrow) => arrow.render(ui),
                    //         _ => (),
                    //     }
                    // }

                    // fill the scrollarea with canvas space, since we've drawn shapes on
                    // top and not actually filled the scrollarea with widgets
                    ui.allocate_at_least(self.canvas_size, Sense::hover());
                });
                //this ".response" here at the end just makes sure we return the right type,
                //you don't have to worry about it too much, but i can explain it if youd like.
            })
            .response
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            diagram,
        } = self;
        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("FlashTech", "https://github.com/afidegnum");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.squares) // The central panel the region left after adding TopPanel's and SidePanel's
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
