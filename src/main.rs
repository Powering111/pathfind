use std::collections::BinaryHeap;

use macroquad::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone)]
struct Pos(i64, i64);

impl Pos {
    fn distance(&self, other: &Self) -> u64 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

impl std::ops::Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

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

#[derive(PartialEq, Eq)]
struct CellData {
    pos: Pos,
    fscore: u64,
}

impl Ord for CellData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // min heap
        other.fscore.cmp(&self.fscore).then_with(|| {
            other
                .pos
                .0
                .cmp(&self.pos.0)
                .then_with(|| other.pos.1.cmp(&self.pos.1))
        })
    }
}

impl PartialOrd for CellData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

    stat_numcalc: u64,
}

impl Context {
    fn set_control_state(&mut self, control_state: ControlState) {
        if self.control_state != control_state {
            self.control_state = control_state;
        }
    }

    fn is_passable(&self, pos: Pos) -> bool {
        pos.0 >= 0
            && pos.0 < ROWS as i64
            && pos.1 >= 0
            && pos.1 < COLS as i64
            && !self.is_wall[pos.0 as usize][pos.1 as usize]
    }

    fn calculate(&mut self) {
        self.stat_numcalc = 0;
        if let (Some(start), Some(end)) = (self.start, self.end) {
            self.path = Vec::new();

            // A* algorithm
            let mut q: BinaryHeap<CellData> = BinaryHeap::new();

            q.push(CellData {
                pos: start,
                fscore: start.distance(&end),
            });

            if !(self.is_passable(start) && self.is_passable(end)) {
                return;
            }

            let mut gscore: [[Option<u64>; COLS as usize]; ROWS as usize] =
                [[None; COLS as usize]; ROWS as usize];
            gscore[start.0 as usize][start.1 as usize] = Some(0);
            let mut parent = [[None; COLS as usize]; ROWS as usize];
            let mut visited = [[false; COLS as usize]; ROWS as usize];
            while !q.is_empty() {
                let curr = q.pop().unwrap().pos;
                if visited[curr.0 as usize][curr.1 as usize] {
                    continue;
                }

                self.stat_numcalc += 1;
                if curr == end {
                    // reconstruct path
                    let mut p = end;
                    while p != start {
                        self.path.push(p);
                        p = parent[p.0 as usize][p.1 as usize].unwrap();
                    }

                    self.path.reverse();
                    break;
                }

                for direction in [(-1, 0), (1, 0), (0, 1), (0, -1)] {
                    let next_pos = curr + Pos(direction.0, direction.1);
                    if self.is_passable(next_pos)
                        && parent[next_pos.0 as usize][next_pos.1 as usize].is_none()
                    {
                        parent[next_pos.0 as usize][next_pos.1 as usize] = Some(curr);

                        let tentative_gscore =
                            gscore[curr.0 as usize][curr.1 as usize].unwrap() + 1;
                        let next_gscore = gscore[next_pos.0 as usize][next_pos.1 as usize];
                        if next_gscore.is_none() || tentative_gscore < next_gscore.unwrap() {
                            gscore[next_pos.0 as usize][next_pos.1 as usize] =
                                Some(tentative_gscore);

                            q.push(CellData {
                                pos: next_pos,
                                fscore: tentative_gscore + next_pos.distance(&end),
                            });
                            visited[next_pos.0 as usize][next_pos.1 as usize] = false;
                        }
                    }
                }
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

        stat_numcalc: 0,
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
            Some(Pos(mouse_pos_world.y as i64, mouse_pos_world.x as i64))
        } else {
            None
        };

        match context.control_state {
            ControlState::Grid => 'l: {
                if is_mouse_button_pressed(MouseButton::Middle) {
                    context.set_control_state(ControlState::Panning);
                    break 'l;
                }

                if let Some(Pos(r, c)) = context.mouse_grid
                    && is_mouse_button_pressed(MouseButton::Left)
                {
                    context.set_control_state(ControlState::Drawing(
                        !context.is_wall[r as usize][c as usize],
                    ));
                    break 'l;
                }

                if is_key_down(KeyCode::S) {
                    if context.mouse_grid != context.start {
                        context.start = context.mouse_grid;
                        context.calculate();
                    }
                }
                if is_key_down(KeyCode::E) {
                    if context.mouse_grid != context.end {
                        context.end = context.mouse_grid;
                        context.calculate();
                    }
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

                if let Some(Pos(r, c)) = context.mouse_grid {
                    if context.is_wall[r as usize][c as usize] != is_draw {
                        context.is_wall[r as usize][c as usize] = is_draw;
                        context.calculate()
                    }
                }
            }
        }

        set_camera(&context.camera);

        for r in 0..ROWS as i64 {
            for c in 0..COLS as i64 {
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
                if context.mouse_grid == Some(Pos(r, c)) {
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
        draw_text(
            &format!("pathlen: {:?}", context.path.len()),
            10.0,
            60.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("numcalc: {:?}", context.stat_numcalc),
            10.0,
            80.0,
            20.0,
            WHITE,
        );

        draw_text(
            &format!("[S] set start"),
            10.0,
            screen_height() - 80.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("[E] set end"),
            10.0,
            screen_height() - 60.0,
            20.0,
            WHITE,
        );
        next_frame().await;
    }
}
