use minesweeper::board::{Board, Square};

pub struct TemplateApp {
    rows: usize,
    cols: usize,
    mines: usize,
    seed: u64,
    use_seed: bool,
    board: Board,
    previous_frame_time: Option<f64>,
    primary_button_down_event_fired: bool,
    last_primary_press_processed: bool,
    secondary_button_down_event_fired: bool,
    last_secondary_press_processed: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            rows: 9,
            cols: 9,
            mines: 10,
            seed: 1,
            use_seed: false,
            board: Board::new(9, 9, 10),
            previous_frame_time: None,
            primary_button_down_event_fired: false,
            last_primary_press_processed: false,
            secondary_button_down_event_fired: false,
            last_secondary_press_processed: false,
        }
    }
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                // egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Minesweeper configuration");
                ui.add_space(10.0);

                if ui.button("Reset board").clicked() {
                    self.board = Board::new(self.rows, self.cols, self.mines);
                }

                ui.add_space(10.0);

                if ui.button("Beginner").clicked() {
                    self.rows = 9;
                    self.cols = 9;
                    self.mines = 10;
                    self.board = Board::new(9, 9, 10);
                }

                if ui.button("Intermediate").clicked() {
                    self.rows = 16;
                    self.cols = 16;
                    self.mines = 40;
                    self.board = Board::new(16, 16, 40);
                }

                if ui.button("Expert").clicked() {
                    self.rows = 16;
                    self.cols = 30;
                    self.mines = 99;
                    self.board = Board::new(16, 30, 99);
                }

                ui.add_space(10.0);
                ui.label("Customize behaviour");

                let sliders = vec![
                    ui.add(egui::Slider::new(&mut self.rows, 5..=40).text("Nr of rows")),
                    ui.add(egui::Slider::new(&mut self.cols, 5..=50).text("Nr of columns")),
                    ui.add(
                        egui::Slider::new(&mut self.mines, 1..=(self.rows * self.cols - 1))
                            .text("Nr of mines"),
                    ),
                ];

                for r in sliders {
                    if r.changed() {
                        self.board = Board::new(self.rows, self.cols, self.mines);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                let seed_toggle = ui.add(egui::Checkbox::new(&mut self.use_seed, "Use seed?"));
                let seed_response =
                    ui.add(egui::Slider::new(&mut self.seed, 0..=1000).text("Seed"));
                // reset board
                if seed_toggle.clicked() {
                    self.board = Board::new(self.rows, self.cols, self.mines);
                }
                if seed_response.changed() && self.use_seed {
                    self.board = Board::new(self.rows, self.cols, self.mines);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.image(egui::include_image!("../assets/Ferris.svg"));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // FPS calculation
            let now = ui.ctx().input(|i| i.time);
            let mut fps = 0.0;
            if let Some(prev) = self.previous_frame_time {
                let delta_time = now - prev;
                if delta_time > 0.0 {
                    fps = 1.0 / delta_time;
                }
            }
            self.previous_frame_time = Some(now);

            ui.vertical_centered(|ui| {
                ui.heading("Minesweeper");
                ui.label(format!("FPS: {:.2}", fps));
            });
            ui.separator();

            let central_panel_rect = ui.min_rect();
            let center_x = central_panel_rect.center().x;
            let center_y = central_panel_rect.center().y;
            let mut responses = Vec::new();

            let max_square_size = 50.;

            let max_col_size = ui.available_width() / self.cols as f32;
            let max_row_size = ui.available_height() / self.rows as f32;

            let square_size = if max_row_size > max_square_size && max_col_size > max_square_size {
                max_square_size
            } else if max_row_size < max_col_size {
                max_row_size
            } else {
                max_col_size
            };

            let board_top_left = egui::Pos2 {
                x: center_x - (self.cols as f32 / 2. * square_size),
                y: center_y - (self.rows as f32 / 2. * square_size),
            };

            let grid = self.board.get_board_state();

            for row in 0..grid.len() {
                for col in 0..grid[0].len() {
                    let square = grid[row][col];
                    let color = match square {
                        Square::NotYetOpened => egui::Color32::from_rgb(255, 255, 255),
                        Square::Mine => egui::Color32::from_rgb(255, 255, 255),
                        Square::Flag => egui::Color32::from_rgb(255, 255, 255),
                        Square::Opened(_) => egui::Color32::from_rgb(255, 255, 255),
                    };
                    let top_left = egui::Pos2 {
                        x: board_top_left.x + (col as f32 * square_size),
                        y: board_top_left.y + (row as f32 * square_size),
                    };
                    let bottom_right = egui::Pos2 {
                        x: top_left.x + square_size,
                        y: top_left.y + square_size,
                    };
                    let rect = egui::Rect::from_two_pos(top_left, bottom_right);
                    let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                    responses.push((response, rect, color, col, row, square));
                }
            }
            let painter = ui.painter();
            for (response, rect, color, col, row, square) in responses {
                painter.rect_filled(rect, 0.0, color);
                let stroke = egui::Stroke::new(rect.width() * 0.02, egui::Color32::BLACK);
                painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Middle);
                let text_pos = rect.center();
                let pos_str = match square {
                    Square::NotYetOpened => "",
                    Square::Mine => "💣",
                    Square::Flag => "🚩",
                    Square::Opened(count) => &format!("{}", count),
                };
                // check for primary button press
                if response.is_pointer_button_down_on()
                    && !self.last_primary_press_processed
                    && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary))
                {
                    self.primary_button_down_event_fired = true;
                    self.last_primary_press_processed = true;
                    if !self.board.initialized() {
                        self.board.init_mines(
                            (col, row),
                            if self.use_seed { Some(self.seed) } else { None },
                        );
                    } else {
                        // TODO handle result
                        let _open_res = self.board.open((col, row));
                    }
                }
                // Reset the processed flag when button is use released
                if ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                    self.last_primary_press_processed = false;
                }
                // check for secondary button press
                if response.is_pointer_button_down_on()
                    && !self.last_secondary_press_processed
                    && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary))
                {
                    self.secondary_button_down_event_fired = true;
                    self.last_secondary_press_processed = true;
                    // TODO handle result
                    let _flag_res = self.board.flag((col, row));
                }
                // Reset the processed flag when button is use released
                if ctx.input(|i| i.pointer.button_released(egui::PointerButton::Secondary)) {
                    self.last_secondary_press_processed = false;
                }
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    pos_str,
                    egui::FontId::proportional(square_size * 0.25),
                    egui::Color32::BLACK,
                );
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
