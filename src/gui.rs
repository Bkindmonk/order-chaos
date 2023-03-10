use eframe::epaint::{Vec2, Rounding};
use eframe::{App, egui, Frame, NativeOptions, run_native};
use eframe::egui::{Color32, Context, FontFamily, FontId, RichText, TextFormat, Ui, Button};
use eframe::egui::text::LayoutJob;
use egui::CentralPanel;
use Screens::{End, Game};
use Tile::Empty;
use tile::Tile::{Blue, Red};
use crate::gui::Screens::Welcome;
use crate::players::Player;
use crate::players::Player::{Chaos, Order};
use crate::state::GameState;
use crate::{config, make_a_random_move, tile};


use crate::tile::Tile;


pub struct MainWindow {
    game_state: GameState,
    chosen_tile: Tile,
    screen: Screens,
    winner: Option<Player>,
    tile_size: f32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Screens {
    Welcome,
    Game,
    End,
}

impl MainWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn default() -> Self {
        Self {
            game_state: GameState::default_new(),
            chosen_tile: Blue,
            screen: Welcome,
            winner: None,
            tile_size: 110.0,
        }
    }

    fn show_grid(&mut self, ui: &mut Ui, interactive: bool) {
        egui::Grid::new("Demo Grid").spacing(Vec2::new(0.0, 0.0)).show(ui, |ui| {
            let original_board;
            {
                original_board = self.game_state.board
            }
            for (row_index, row) in original_board.iter().enumerate() {
                for (column_index, tile) in row.iter().enumerate() {
                    show_tile(self, tile, ui, (row_index, column_index), interactive);
                }
                ui.end_row();
            }
        });
    }

    fn show_pawn_selector(&mut self, ui: &mut Ui, tile_size: f32) {
        ui.label("\n\nSelect Pawn:");
        egui::Grid::new("Demo Grid2").show(ui, |ui| {
            ui.selectable_value(&mut self.chosen_tile, Blue, RichText::new("🌑").color(Color32::BLUE).size(tile_size));
            ui.selectable_value(&mut self.chosen_tile, Red, RichText::new("❌").color(Color32::RED).size(tile_size));
            ui.end_row();
        });
    }

    fn show_welcome_screen(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label(welcome_screen_layout());
            ui.add_space(64.0);
            if ui.add(egui::Button::new(RichText::new(" Continue ").size(32.0))).clicked() {
                self.screen = Game;
            }
        });
    }

    fn show_game_screen(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Order & Chaos");
            egui::Grid::new("Demo Grid3").show(ui, |ui| {
                ui.label(RichText::new("Current Active Player:").size(32.0));
                ui.label(player_fmt(&self.game_state.turn_player));
                ui.end_row();
                egui::Grid::new("Demo Grid4").show(ui, |ui| {
                    ui.label(RichText::new("Tile Size:").size(16.0));
                    ui.add(egui::Slider::new(&mut self.tile_size, 5.0..=250.0));
                });
            });
            self.show_grid(ui, !config::get().ai_vs_ai_demo);
            self.show_pawn_selector(ui, self.tile_size);
        });

        if config::get().ai_vs_ai_demo {
            make_a_random_move(&mut self.game_state);
            evaluate_game_state(&self.game_state, &mut self.winner);
        }
    }
    fn show_end_screen(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label(end_screen_layout(self));
            self.show_grid(ui, false);
            ui.add_space(32.0);
            if ui.add(egui::Button::new(RichText::new(" Exit Game ").size(32.0))).clicked() {
                frame.close();
            }
        });
    }
}

impl App for MainWindow {
    fn update(&mut self, context: &Context, frame: &mut Frame) {
        if self.winner.is_some() {
            self.screen = End;
        }

        match self.screen {
            Welcome => self.show_welcome_screen(context),
            Game => self.show_game_screen(context),
            End => self.show_end_screen(context, frame)
        }
    }
}

pub fn show_main_screen() {
    let native_options = NativeOptions { maximized: true, ..Default::default() };
    run_native("Order & Chaos", native_options, Box::new(|cc| Box::new(MainWindow::new(cc)))).unwrap();
}


fn show_tile(main_window: &mut MainWindow, tile: &Tile, ui: &mut Ui, coordinates: (usize, usize), interactive: bool) {
    match tile {
        Empty => add_empty(main_window, ui, coordinates, interactive),
        Blue => add_blue(main_window, ui, coordinates, interactive),
        Red => add_red(main_window, ui, coordinates, interactive)
    }
}

fn add_empty(main_window: &mut MainWindow,ui: &mut Ui, coordinates: (usize, usize), interactive: bool) {
    add_button(main_window, ui, coordinates, interactive, "⬛".to_owned(), Color32::WHITE);
}

fn add_blue(main_window: &mut MainWindow,ui: &mut Ui, coordinates: (usize, usize), interactive: bool) {
    add_button(main_window, ui, coordinates, interactive, "🌑".to_owned(), Color32::BLUE);
}

fn add_red(main_window: &mut MainWindow, ui: &mut Ui, coordinates: (usize, usize), interactive: bool) {
    add_button(main_window, ui, coordinates, interactive, "❌".to_owned(), Color32::RED);
}

fn add_button(main_window: &mut MainWindow, ui: &mut Ui, coordinates: (usize, usize), interactive: bool, text: String, color: Color32){
    if ui.add(Button::new(RichText::new(text).color(color).size(main_window.tile_size).background_color(get_tile_color(&main_window.game_state, &main_window.winner)))
        .fill(get_tile_color(&main_window.game_state, &main_window.winner))
        .min_size(Vec2 { x: main_window.tile_size, y: main_window.tile_size })
        .rounding(Rounding::none()))
    .clicked() && interactive{
        let _ = main_window.game_state.play(coordinates, main_window.chosen_tile);
        evaluate_game_state(&main_window.game_state, &mut main_window.winner);
    }
}

fn get_tile_color(game_state: &GameState, winner: &Option<Player>) -> Color32 {
    match winner {
        Some(winner) => get_player_color(*winner),
        None => get_player_color(game_state.turn_player)
    }
}

fn evaluate_game_state(game_state: &GameState, winner: &mut Option<Player>) {
    if game_state.is_in_order() {
        *winner = Some(Order);
    }
    if !game_state.can_order_win() {
        *winner = Some(Chaos);
    }
}

fn player_fmt(player: &Player) -> RichText {
    match *player {
        Order => RichText::new("Order").color(get_player_color(Order)).size(32.0),
        Chaos => RichText::new("Chaos").color(get_player_color(Chaos)).size(32.0)
    }
}

fn get_player_color(player: Player) -> Color32 {
    match player {
        Order => Color32::from_rgb(204, 170, 0),
        Chaos => Color32::from_rgb(51, 153, 0)
    }
}

fn welcome_screen_layout() -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.append("Welcome to the ", 0.0, default_text());
    job.append("ORDER", 0.0, color_text(get_player_color(Order)));
    job.append(" & ", 0.0, default_text());
    job.append("CHAOS", 0.0, color_text(get_player_color(Chaos)));
    job.append(" electronic simulator.\n\n\n", 0.0, default_text());
    job.append("RULES:\n\n\n", 0.0, default_text());

    job.append("• Order plays first, then turns alternate.\n\n", 0.0, default_text());

    job.append("• Both players control both sets of pieces (", 0.0, default_text());
    job.append("❌", 0.0, color_text(Color32::RED));
    job.append(" and ", 0.0, default_text());
    job.append("🌑", 0.0, color_text(Color32::BLUE));
    job.append("). The game starts with the board empty.\n\n", 0.0, default_text());

    job.append("• On each turn, a player places either an ", 0.0, default_text());
    job.append("❌", 0.0, color_text(Color32::RED));
    job.append(" or an ", 0.0, default_text());
    job.append("🌑", 0.0, color_text(Color32::BLUE));
    job.append(" on any open square. Once played, pieces cannot be moved\n\n", 0.0, default_text());

    job.append("• ", 0.0, default_text());
    job.append("ORDER", 0.0, color_text(get_player_color(Order)));
    job.append(" aims to get exactly five like pieces in a row either vertically, horizontally, or diagonally.\n\n", 0.0, default_text());

    job.append("• ", 0.0, default_text());
    job.append("CHAOS", 0.0, color_text(get_player_color(Chaos)));
    job.append(" aims to fill the board without completion of a line of five like pieces.\n\n", 0.0, default_text());

    job.append("• Six-in-a-row does not qualify as a win", 0.0, default_text());
    job
}

fn end_screen_layout(state: &MainWindow) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    match state.winner {
        Some(Order) => job.append("ORDER Won!", 0.0, color_text(get_player_color(Order))),
        Some(Chaos) => job.append("CHAOS Won!", 0.0, color_text(get_player_color(Chaos))),
        None => job.append("I don't know what happened, but it's a DRAW!", 0.0, color_text(Color32::from_rgb(255, 25, 217)))
    }
    job
}

fn default_text() -> TextFormat {
    TextFormat { font_id: FontId::new(32.0, FontFamily::Proportional), ..Default::default() }
}

fn color_text(color: Color32) -> TextFormat {
    TextFormat { font_id: FontId::new(32.0, FontFamily::Proportional), color, ..Default::default() }
}




