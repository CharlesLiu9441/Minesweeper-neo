use eframe::egui;
use rand::seq::index::sample;

#[derive(Clone, Copy, PartialEq)]
enum CellState {Hidden, Revealed, Flagged}
#[derive(Clone, Copy)]
struct Cell {is_mine: bool, state: CellState, adjacent_mines: u8}
impl Default for Cell {
    fn default() -> Self {
        Self {is_mine: false, state: CellState::Hidden, adjacent_mines: 0}
    }
}
#[derive(Clone, Copy)]
struct GameConfig {width: usize, height: usize, mine_count: usize}
impl Default for GameConfig {
    fn default() -> Self {
        Self {width: 10, height: 10, mine_count: 15}
    }
}
struct Minesweeper {grid: Vec<Vec<Cell>>, config: GameConfig, game_over: bool, game_won: bool, first_click: bool}
impl Minesweeper {
    fn new(config: GameConfig) -> Self {
        Self {
            grid: vec![vec![Cell::default(); config.width]; config.height],
            config,
            game_over: false,
            game_won: false,
            first_click: true,
        }
    }
    fn reset(&mut self) {
        self.grid = vec![vec![Cell::default(); self.config.width]; self.config.height];
        self.game_over = false;
        self.game_won = false;
        self.first_click = true;
    }
    fn place_mines(&mut self, safe_x: usize, safe_y: usize) {
        let mut rng = rand::rng();
        let mut space:Vec<Vec<usize>>=vec![vec![0;2];0];
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                if y.abs_diff(safe_y)>1||x.abs_diff(safe_x)>1{
                    space.append(&mut vec![vec![x,y]; 1]);
                }
            }
        }
        let mine_places=sample(&mut rng,space.len(),self.config.mine_count);
        for mi in mine_places{
            self.grid[space[mi][1]][space[mi][0]].is_mine=true;
        }
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                if !self.grid[y][x].is_mine {
                    self.grid[y][x].adjacent_mines = self.count_adjacent_mines(x, y);
                }
            }
        }
    }
    fn count_adjacent_mines(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 && nx < self.config.width as isize && ny < self.config.height as isize {
                    if self.grid[ny as usize][nx as usize].is_mine {
                        count += 1;
                    }
                }
            }
        }
        count
    }
    fn reveal_cell(&mut self, x: usize, y: usize) {
        if self.game_over || self.game_won || self.grid[y][x].state == CellState::Revealed {
            return;
        }
        if self.first_click {
            self.place_mines(x, y);
            self.first_click = false;
        }
        if self.grid[y][x].is_mine {
            self.game_over = true;
            for row in &mut self.grid {
                for cell in row {
                    if !(cell.state == CellState::Flagged&&!cell.is_mine){
                        cell.state = CellState::Revealed;
                    }
                }
            }
            return;
        }
        self.grid[y][x].state = CellState::Revealed;
        if self.grid[y][x].adjacent_mines == 0 {
            self.expand_safe_zone(x, y);
        }
        self.check_win_condition();
    }
    fn expand_safe_zone(&mut self, x: usize, y: usize) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && ny >= 0 && nx < self.config.width as isize && ny < self.config.height as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.grid[ny][nx].state == CellState::Hidden && !self.grid[ny][nx].is_mine {
                        self.grid[ny][nx].state = CellState::Revealed;
                        if self.grid[ny][nx].adjacent_mines == 0 {
                            self.expand_safe_zone(nx, ny);
                        }
                    }
                }
            }
        }
    }
    fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.game_over || self.game_won || self.grid[y][x].state == CellState::Revealed {
            return;
        }
        if self.first_click {
            self.place_mines(x, y);
            self.first_click = false;
        }
        match self.grid[y][x].state {
            CellState::Hidden => self.grid[y][x].state = CellState::Flagged,
            CellState::Flagged => self.grid[y][x].state = CellState::Hidden,
            _ => {}
        }
        self.check_win_condition();
    }
    fn check_win_condition(&mut self) {
        let all_non_mines_revealed = !self.first_click&&self.grid.iter().flatten().all(|cell|
            cell.state != CellState::Hidden||cell.is_mine
        );

        let all_mines_flagged = self.grid.iter().flatten().all(|cell|
            cell.is_mine == (cell.state == CellState::Flagged)
        );

        if all_non_mines_revealed || all_mines_flagged {
            self.game_won = true;
            for row in &mut self.grid {
                for cell in row {
                    if cell.is_mine {
                        cell.state = CellState::Flagged;
                    }
                }
            }
        }
    }
}
struct MinesweeperApp {
    game: Minesweeper,
    show_settings: bool,
    temp_config: GameConfig,
    presets: Vec<(String, GameConfig)>,
}
impl Default for MinesweeperApp {
    fn default() -> Self {
        let presets = vec![
            ("Beginner (9×9, 10 mines)".to_string(), GameConfig { width: 9, height: 9, mine_count: 10 }),
            ("Intermediate (16×16, 40 mines)".to_string(), GameConfig { width: 16, height: 16, mine_count: 40 }),
            ("Expert (30×16, 99 mines)".to_string(), GameConfig { width: 30, height: 16, mine_count: 99 })
        ];
        Self {
            game: Minesweeper::new(GameConfig::default()),
            show_settings: false,
            temp_config: GameConfig::default(),
            presets,
        }
    }
}
impl eframe::App for MinesweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let visuals = ctx.style().visuals.clone();
        ctx.set_visuals(egui::Visuals {
            window_fill: if visuals.dark_mode {
                egui::Color32::from_rgba_unmultiplied(30,30,30,150)
            } else {
                egui::Color32::from_rgba_unmultiplied(190,190,190,150)
            },
            panel_fill: if visuals.dark_mode {
                egui::Color32::from_rgba_unmultiplied(30,30,30,150)
            } else {
                egui::Color32::from_rgba_unmultiplied(190,190,190,150)
            },
            ..ctx.style().visuals.clone()
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.heading(egui::RichText::new("MINESWEEPER").size(30.0).color(if visuals.dark_mode {
                        egui::Color32::from_gray(120)
                    } else {
                        egui::Color32::from_gray(180)
                    }));
                });
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                        self.show_settings = true;
                        self.temp_config = self.game.config;
                    }
                });
            });
        });
        if self.show_settings {
            let set_window_ctx=ctx.clone();
            set_window_ctx.set_visuals(egui::Visuals {
                window_fill: if visuals.dark_mode {
                    egui::Color32::from_gray(30)
                } else {
                    egui::Color32::from_gray(190)
                },
                ..set_window_ctx.style().visuals.clone()
            });
            egui::Window::new("Game Settings")
                .collapsible(false)
                .resizable(false)
                .show(&set_window_ctx, |ui| {
                    ui.label("Select difficulty preset:");
                    for (name, config) in &self.presets {
                        if ui.button(name).clicked() {
                            self.temp_config = *config;
                        }
                    }
                    ui.separator();
                    ui.label("Custom settings:");
                    ui.add(egui::Slider::new(&mut self.temp_config.width, 6..=50).text("Width"));
                    ui.add(egui::Slider::new(&mut self.temp_config.height, 6..=50).text("Height"));
                    ui.add(egui::Slider::new(&mut self.temp_config.mine_count, 1..=(self.temp_config.width * self.temp_config.height-9)).text("Mines"));
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_settings = false;
                        }
                        if ui.button("Apply").clicked() {
                            self.game = Minesweeper::new(self.temp_config);
                            self.show_settings = false;
                        }
                    });
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("🔄 Reset Game").size(15.0)).clicked() {
                    self.game.reset();
                }
            });
            let available_size = ui.available_size();
            let pixel_size_x=(available_size.x*0.95)/self.game.config.width as f32;
            let pixel_size_y=(available_size.y*0.95)/self.game.config.height as f32;
            let cell_pixel_size = pixel_size_x.min(pixel_size_y);
            let total_width = self.game.config.width as f32 * cell_pixel_size;
            let total_height = self.game.config.height as f32 * cell_pixel_size;
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(total_width, total_height),
                    egui::Sense::click_and_drag()
                );
                let pos = response.rect.min;
                let hover_pos = response.hover_pos();
                let mut hover_x = None;
                let mut hover_y = None;
                if let Some(pos) = hover_pos {
                    let x = ((pos.x - response.rect.min.x) / cell_pixel_size).floor() as usize;
                    let y = ((pos.y - response.rect.min.y) / cell_pixel_size).floor() as usize;
                    if x < self.game.config.width && y < self.game.config.height {
                        hover_x = Some(x);
                        hover_y = Some(y);
                    }
                }
                for y in 0..self.game.config.height {
                    for x in 0..self.game.config.width {
                        let cell = &self.game.grid[y][x];
                        let rect = egui::Rect::from_min_size(
                            pos + egui::Vec2::new(x as f32 * cell_pixel_size, y as f32 * cell_pixel_size),
                            egui::Vec2::splat(cell_pixel_size)
                        );
                        let mut bg_color = match cell.state {
                            CellState::Hidden => if visuals.dark_mode {
                                egui::Color32::from_gray(60)
                            } else {
                                egui::Color32::from_gray(220)
                            },
                            CellState::Flagged => if visuals.dark_mode {
                                egui::Color32::from_rgb(150, 70, 70)
                            } else {
                                egui::Color32::from_rgb(250, 180, 180)
                            },
                            CellState::Revealed => if cell.is_mine {
                                if visuals.dark_mode {
                                    egui::Color32::from_rgb(180, 70, 70)
                                } else {
                                    egui::Color32::from_rgb(255, 150, 150)
                                }
                            } else {
                                if visuals.dark_mode {
                                    egui::Color32::from_rgba_unmultiplied(70, 70, 90,150)
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(230, 240, 250,150)
                                }
                            }
                        };
                        if Some(x) == hover_x && Some(y) == hover_y && cell.state == CellState::Hidden {
                            bg_color = if visuals.dark_mode {
                                bg_color.gamma_multiply(1.2)
                            } else {
                                bg_color.gamma_multiply(0.8)
                            };
                        }
                        painter.rect_filled(rect, 0.0, bg_color);
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(0.5, egui::Color32::from_gray(100)),
                            egui::StrokeKind::Outside
                        );
                        let text = match cell.state {
                            CellState::Hidden => "",
                            CellState::Flagged => "🚩",
                            CellState::Revealed => {
                                if cell.is_mine {
                                    "💣"
                                } else if cell.adjacent_mines>0{
                                    &cell.adjacent_mines.to_string() as &str
                                }else{
                                    ""
                                }
                            }
                        };
                        let text_color =match cell.state{
                            CellState::Revealed => match cell.adjacent_mines {
                                1 => egui::Color32::from_rgb(100, 150, 255),
                                2 => egui::Color32::from_rgb(50, 200, 50),
                                3 => egui::Color32::from_rgb(255, 80, 80),
                                4 => egui::Color32::from_rgb(0, 0, 150),
                                5 => egui::Color32::from_rgb(150, 70, 0),
                                6 => egui::Color32::from_rgb(0, 180, 180),
                                7 => egui::Color32::BLACK,
                                8 => egui::Color32::from_gray(120),
                                _ => egui::Color32::WHITE,
                            },
                            _ => egui::Color32::WHITE
                        };
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::monospace(cell_pixel_size*0.7),
                            text_color
                        );
                    }
                }
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let x = ((pos.x - response.rect.min.x) / cell_pixel_size).floor() as usize;
                        let y = ((pos.y - response.rect.min.y) / cell_pixel_size).floor() as usize;
                        if x < self.game.config.width && y < self.game.config.height {
                            if self.game.grid[y][x].state!=CellState::Flagged{
                                self.game.reveal_cell(x, y);
                            }
                            if self.game.grid[y][x].adjacent_mines!=0&&self.game.grid[y][x].state==CellState::Revealed{
                                let mut near_flagged = 0;
                                for dx in -1..=1{
                                    for dy in -1..=1{
                                        let nx=x as i32 + dx;
                                        let ny=y as i32 + dy;
                                        if nx<0||nx>=self.game.config.width as i32||ny<0||ny>=self.game.config.height as i32{
                                            continue;
                                        }
                                        if self.game.grid[ny as usize][nx as usize].state==CellState::Flagged{
                                            near_flagged+=1;
                                        }
                                    }
                                }
                                if near_flagged==self.game.grid[y][x].adjacent_mines{
                                    for dx in -1..=1{
                                        for dy in -1..=1{
                                            let nx=x as i32 + dx;
                                            let ny=y as i32 + dy;
                                            if nx<0||nx>=self.game.config.width as i32||ny<0||ny>=self.game.config.height as i32{
                                                continue;
                                            }
                                            if self.game.grid[ny as usize][nx as usize].state!=CellState::Flagged{
                                                self.game.reveal_cell(nx as usize, ny as usize);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let x = ((pos.x - response.rect.min.x) / cell_pixel_size).floor() as usize;
                        let y = ((pos.y - response.rect.min.y) / cell_pixel_size).floor() as usize;
                        if x < self.game.config.width && y < self.game.config.height {
                            self.game.toggle_flag(x, y);
                        }
                        if self.game.grid[y][x].adjacent_mines!=0&&self.game.grid[y][x].state==CellState::Revealed{
                            let mut near_unrevealed = 0;
                            for dx in -1..=1{
                                for dy in -1..=1{
                                    let nx=x as i32 + dx;
                                    let ny=y as i32 + dy;
                                    if nx<0||nx>=self.game.config.width as i32||ny<0||ny>=self.game.config.height as i32{
                                        continue;
                                    }
                                    if self.game.grid[ny as usize][nx as usize].state!=CellState::Revealed{
                                        near_unrevealed+=1;
                                    }
                                }
                            }
                            if near_unrevealed==self.game.grid[y][x].adjacent_mines{
                                for dx in -1..=1{
                                    for dy in -1..=1{
                                        let nx=x as i32 + dx;
                                        let ny=y as i32 + dy;
                                        if nx<0||nx>=self.game.config.width as i32||ny<0||ny>=self.game.config.height as i32{
                                            continue;
                                        }
                                        if self.game.grid[ny as usize][nx as usize].state!=CellState::Revealed&&self.game.grid[ny as usize][nx as usize].state!=CellState::Flagged{
                                            self.game.toggle_flag(nx as usize, ny as usize);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if self.game.game_over || self.game.game_won {
                    let rect = response.rect;
                    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(150));
                    let (text, color) = if self.game.game_over {
                        ("💥 Game Over! You hit a mine! 💥", egui::Color32::from_rgb(255, 100, 100))
                    } else {
                        ("🎉 Congratulations! You won! 🎉", egui::Color32::from_rgb(100, 255, 100))
                    };
                    painter.text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(24.0), color);
                    let button_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.center().x, rect.center().y + 50.0),
                        egui::Vec2::new(150.0, 40.0)
                    );
                    if ui.put(button_rect, egui::Button::new("🔄 Restart")).clicked() {
                        self.game.reset();
                    }
                }
            });
        });
    }
}
fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_transparent(true).with_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals {
                window_fill: egui::Color32::TRANSPARENT,
                panel_fill: egui::Color32::TRANSPARENT,
                ..Default::default()
            });
            Ok(Box::new(MinesweeperApp::default()))
        })
    ).expect("Error occurred while initializing the main window");
}