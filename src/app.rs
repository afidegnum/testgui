use std::sync::mpsc::{Receiver, Sender};

use crate::meta::get_metadata;
use egui::{Color32, FontId, Sense, Vec2};
use emath::{Align2, Pos2};
use tokio::runtime;
use tokio_postgres::NoTls;

pub const INIT_POS: Pos2 = egui::pos2(10.0, 15.0);
pub const ATTR_SIZE: Vec2 = egui::vec2(150.0, 25.0);
// pub const ATTR_PADDING: Vec2 = egui::vec2(15.0, 10.0);

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
///
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default = "Diagram::new")]
struct Diagram {
    shapes: Vec<Square>,
    canvas_size: Vec2,
    #[serde(skip)]
    task_reciever: Receiver<TaskMessage>,
    //the ui thread doesn't use this, but gives it to "worker threads" when spawing them
    //this can also be given to a thread that works alongside the ui thread from the start (if you have one)
    #[serde(skip)]
    _task_sender: Sender<TaskMessage>,
}

impl Diagram {
    fn new() -> Self {
        Self {
            shapes: vec![Square::new()],
            canvas_size: Vec2::splat(400.0),
            task_reciever: todo!(),
            _task_sender: todo!(),
        }
    }

    pub fn handle_responses(&mut self) {
        let responses: Vec<TaskMessage> = self.task_reciever.try_iter().collect();
        for response in responses {
            match response {
                TaskMessage::Generic(gen_function) => {
                    //Since "gen_function" is of type "FnOnce(&mut MyApp)",
                    //We can call it like a function with ourself as the parameter,
                    gen_function(self);
                }
            }
        }
    }
}

impl egui::Widget for &mut Diagram {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Frame::canvas(ui.style())
            .show(ui, |ui| {
                egui::ScrollArea::new([true; 2]).show(ui, |ui| {
                    // let rt = runtime::Runtime::new().unwrap();

                    tokio::task::spawn_blocking(|| {
                        let (client, connection) = tokio_postgres::connect(
                            "postgresql://postgres:chou1979@localhost/authenticate",
                            NoTls,
                        )
                        .await
                        .unwrap();

                        let schema = "public".to_string();
                        // let meta_data = get_metadata(&client, schema).await;
                        //
                        //
                        let sender = self._task_sender.clone();
                        let other_ctx = ui.ctx().clone();
                        //requires you add the "rt" feature to tokio!!
                        tokio::task::spawn(async {
                            get_metadata(&client, schema, other_ctx, sender)
                        });
                        //
                        //
                        //you'll probably want to use somehting like this since it's what you returned earlier
                    })
                    .await
                    .unwrap();
                    // rt.block_on(async {});

                    for shape in self.shapes.iter_mut() {
                        let container = "container".to_string();

                        shape.render(ui, container);
                    }
                    ui.allocate_at_least(self.canvas_size, Sense::hover());
                });
            })
            .response
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Square {
    position: egui::Pos2,
    dimension: egui::Vec2,
    attributes: Vec<InnerSquare>,
}

impl Square {
    fn new() -> Self {
        Self {
            position: egui::pos2(INIT_POS.x, INIT_POS.y),
            dimension: egui::vec2(ATTR_SIZE.x, ATTR_SIZE.y),
            attributes: vec![InnerSquare::new()],
        }
    }
    fn render(&mut self, ui: &mut egui::Ui, container: String) {
        let square_body = egui::Rect::from_min_size(self.position, self.dimension);
        //"finalized rect" which is offset properly
        let transformed_rect = {
            let resp = ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::click());
            let relative_to_screen = egui::emath::RectTransform::from_to(
                egui::Rect::from_min_size(Pos2::ZERO, resp.rect.size()),
                resp.rect,
            );
            relative_to_screen.transform_rect(square_body)
        };

        // It would probably be better to use a frame than draw everything manually
        // the frame will resize properly to everything you put inside it
        // and we don't have to deal with draw order shenanigans
        let frame = {
            let rounding_radius = 2.0;
            let fill = egui::Color32::LIGHT_GREEN;
            let stroke = egui::epaint::Stroke::new(2.0, Color32::DARK_BLUE);
            egui::Frame::none()
                .rounding(rounding_radius)
                .fill(fill)
                .stroke(stroke)
                .inner_margin(10.0)
        };
        //Creates a new ui where our square is supposed to appear
        ui.allocate_ui_at_rect(transformed_rect, |ui| {
            frame.show(ui, |ui| {
                //draw each attribute
                ui.label(
                    egui::RichText::new(container)
                        .heading()
                        .color(egui::Color32::BLACK),
                );
                for inner_square in self.attributes.iter_mut() {
                    inner_square.render(ui);
                }
            });
        });
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct InnerSquare {
    position: egui::Pos2,
    dimension: egui::Vec2,
}

impl InnerSquare {
    fn new() -> Self {
        Self {
            position: egui::pos2(INIT_POS.x, INIT_POS.y),
            dimension: egui::vec2(ATTR_SIZE.x, ATTR_SIZE.y),
        }
    }
    fn render(&mut self, ui: &mut egui::Ui) {
        //this replaces all the transform and ui.allocate code.
        //this also removes the need for ``position`` in InnerSquare, since the
        // ``Square``s ui will lay everything out for us
        let (rect, resp) = ui.allocate_at_least(self.dimension, Sense::click());

        let mut fill = egui::Color32::LIGHT_BLUE;
        if resp.hovered() {
            fill = egui::Color32::DARK_BLUE;
        }

        let rounding_radius = 2.0;
        let stroke = egui::epaint::Stroke::new(1.0, Color32::BLACK);

        //draw the rect
        ui.painter().rect(rect, rounding_radius, fill, stroke);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "INNER RECT",
            FontId::proportional(14.0),
            Color32::BLACK,
        );
    }
}

// #[derive(serde::Deserialize, serde::Serialize)]
pub enum TaskMessage {
    //Applicaple to any scenario, behaves almost like a callback
    Generic(Box<dyn FnOnce(&mut TemplateApp) + Send>),
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    diagram: Diagram,
    label: String,
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
    //this is what the ui thread will just to catch returns of tasks, in this case it's the std
    //mpsc channels, but any channel which has smiliar behaviour works
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            diagram: Diagram::new(),
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
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        Default::default()
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
            diagram,
            value,
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
            ui.add(&mut self.diagram) // The central panel the region left after adding TopPanel's and SidePanel's
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
