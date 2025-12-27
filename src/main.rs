use macroquad::prelude::*;

type Pos = (u64, u64);
const ROWS: u64 = 20;
const COLS: u64 = 20;

fn conf() -> miniquad::conf::Conf {
    miniquad::conf::Conf {
        window_title: "Pathfinding!".to_owned(),
        window_width: 1600,
        window_height: 900,
        high_dpi: true,
        ..Default::default()
    }
}

#[derive(Debug, PartialEq)]
enum ControlState {
    Grid,
    Panning,
    Drawing(bool),
}

struct Context {
    mouse_grid: Option<Pos>,
    control_state: ControlState,
    zoom: f32,
    camera: Camera2D,
    is_wall: [[bool; COLS as usize]; ROWS as usize],

    start: Option<Pos>,
    end: Option<Pos>,
    path: Vec<Pos>,
}

impl Context {
    fn set_control_state(&mut self, control_state: ControlState) {
        if self.control_state != control_state {
            self.control_state = control_state;
        }
    }

    fn calculate(&mut self) {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            self.path = Vec::new();
            let mut p = start;
            while p.0 != end.0 {
                if end.0 > p.0 {
                    p.0 += 1;
                } else {
                    p.0 -= 1;
                }
                self.path.push(p);
            }
            while p.1 != end.1 {
                if end.1 > p.1 {
                    p.1 += 1;
                } else {
                    p.1 -= 1;
                }
                self.path.push(p);
            }
        } else {
            self.path = Vec::new();
        }
    }
}

pub(crate) fn draw_text_centered(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    font_scale: f32,
    color: Color,
) {
    let center = get_text_center(text, None, font_size as u16, font_scale, 0.0);
    draw_text_ex(
        text,
        x - center.x,
        y - center.y,
        TextParams {
            font_size: font_size as u16,
            font_scale,
            color,
            ..Default::default()
        },
    );
}

#[macroquad::main(conf)]
async fn main() {
    clear_background(BLACK);

    let mut context = Context {
        mouse_grid: None,
        control_state: ControlState::Grid,
        zoom: 0.1,
        camera: Camera2D {
            zoom: vec2(0.1 * screen_height() / screen_width(), 0.1),
            target: vec2(COLS as f32 / 2.0, ROWS as f32 / 2.0),
            offset: vec2(0.0, 0.0),
            ..Default::default()
        },
        is_wall: [[false; COLS as usize]; ROWS as usize],
        start: None,
        end: None,
        path: Vec::new(),
    };

    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        let mouse_wheel_y = mouse_wheel().1;
        if mouse_wheel_y > 0.0 {
            context.zoom = f32::max(0.01, context.zoom * 1.1);
        } else if mouse_wheel_y < 0.0 {
            context.zoom = f32::min(1.0, context.zoom * 0.9);
        }
        context.camera.zoom = vec2(
            context.zoom * screen_height() / screen_width(),
            context.zoom,
        );

        let mouse_pos_world = context.camera.screen_to_world(mouse_position().into());
        context.mouse_grid = if mouse_pos_world.x >= 0.0
            && mouse_pos_world.x < COLS as f32
            && mouse_pos_world.y >= 0.0
            && mouse_pos_world.y < ROWS as f32
        {
            Some((mouse_pos_world.y as u64, mouse_pos_world.x as u64))
        } else {
            None
        };

        match context.control_state {
            ControlState::Grid => 'l: {
                if is_mouse_button_pressed(MouseButton::Middle) {
                    context.set_control_state(ControlState::Panning);
                    break 'l;
                }

                if let Some((r, c)) = context.mouse_grid
                    && is_mouse_button_pressed(MouseButton::Left)
                {
                    context.set_control_state(ControlState::Drawing(
                        !context.is_wall[r as usize][c as usize],
                    ));
                    break 'l;
                }

                if is_key_down(KeyCode::S) {
                    context.start = context.mouse_grid;
                    context.calculate();
                }
                if is_key_down(KeyCode::E) {
                    context.end = context.mouse_grid;
                    context.calculate();
                }
            }
            ControlState::Panning => 'l: {
                if is_mouse_button_released(MouseButton::Middle) {
                    context.set_control_state(ControlState::Grid);
                    break 'l;
                }

                let delta = mouse_delta_position() / context.camera.zoom;
                context.camera.target += delta;
            }
            ControlState::Drawing(is_draw) => 'l: {
                if is_mouse_button_released(MouseButton::Left) {
                    context.set_control_state(ControlState::Grid);
                    break 'l;
                }

                if let Some((r, c)) = context.mouse_grid {
                    context.is_wall[r as usize][c as usize] = is_draw;
                }
            }
        }

        set_camera(&context.camera);

        for r in 0..ROWS {
            for c in 0..COLS {
                if context.is_wall[r as usize][c as usize] {
                    draw_rectangle(
                        c as f32,
                        r as f32,
                        1.0,
                        1.0,
                        Color::new(0.9, 0.9, 0.9, 1.00),
                    );
                }
                draw_rectangle_lines(c as f32, r as f32, 1.0, 1.0, 0.05, WHITE);

                // outline
                if context.mouse_grid == Some((r, c)) {
                    draw_rectangle_lines(c as f32, r as f32, 1.0, 1.0, 0.1, YELLOW);
                }
            }
        }

        if let Some(start) = context.start {
            draw_text_centered(
                "S",
                start.1 as f32 + 0.5,
                start.0 as f32 + 0.5,
                50.0,
                0.02,
                WHITE,
            );

            let mut prev_point = start;
            for p in context.path.iter() {
                let p1 = vec2(prev_point.1 as f32 + 0.5, prev_point.0 as f32 + 0.5);
                let p2 = vec2(p.1 as f32 + 0.5, p.0 as f32 + 0.5);
                draw_line(p1.x, p1.y, p2.x, p2.y, 0.1, GREEN);
                prev_point = *p;
            }
        }
        if let Some(end) = context.end {
            draw_text_centered(
                "E",
                end.1 as f32 + 0.5,
                end.0 as f32 + 0.5,
                50.0,
                0.02,
                WHITE,
            );
        }

        draw_circle(0.0, 0.0, 0.1, RED);
        draw_circle(mouse_pos_world.x, mouse_pos_world.y, 0.1, BLUE);

        // UI
        set_default_camera();
        draw_text(
            &format!("{:?}", context.control_state),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        next_frame().await;
    }
}
