use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position},
    prelude::*,
    style::{Color, Modifier, Style},
    symbols::Marker,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph,
    },
};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, stdout, Stdout};
use std::time::Instant;

#[derive(Clone)]
struct Theme {
    correct_fg: Color,
    incorrect_fg: Color,
    pending_fg: Color,
    bg_color: Color,
    stats_fg: Color,
    chart_fg: Color,
    axis_fg: Color,
    grid_fg: Color,
    label_fg: Color,
}

impl Theme {
    fn default_theme() -> Self {
        Self {
            correct_fg: Color::Green,
            incorrect_fg: Color::Red,
            pending_fg: Color::DarkGray,
            bg_color: Color::Reset,
            stats_fg: Color::Green,
            chart_fg: Color::Cyan,
            axis_fg: Color::White,
            grid_fg: Color::DarkGray,
            label_fg: Color::White,
        }
    }

    fn dracula() -> Self {
        Self {
            correct_fg: Color::Rgb(80, 250, 123),
            incorrect_fg: Color::Rgb(255, 85, 85),
            pending_fg: Color::Rgb(98, 114, 164),
            bg_color: Color::Rgb(40, 42, 54),
            stats_fg: Color::Rgb(80, 250, 123),
            chart_fg: Color::Rgb(139, 233, 253),
            axis_fg: Color::Rgb(248, 248, 242),
            grid_fg: Color::Rgb(98, 114, 164),
            label_fg: Color::Rgb(248, 248, 242),
        }
    }

    fn monokai() -> Self {
        Self {
            correct_fg: Color::Rgb(166, 226, 46),
            incorrect_fg: Color::Rgb(249, 38, 114),
            pending_fg: Color::Rgb(117, 113, 94),
            bg_color: Color::Rgb(39, 40, 34),
            stats_fg: Color::Rgb(166, 226, 46),
            chart_fg: Color::Rgb(253, 151, 31),
            axis_fg: Color::Rgb(248, 248, 242),
            grid_fg: Color::Rgb(117, 113, 94),
            label_fg: Color::Rgb(248, 248, 242),
        }
    }

    fn nord() -> Self {
        Self {
            correct_fg: Color::Rgb(163, 190, 140),
            incorrect_fg: Color::Rgb(191, 97, 106),
            pending_fg: Color::Rgb(76, 86, 106),
            bg_color: Color::Rgb(46, 52, 64),
            stats_fg: Color::Rgb(163, 190, 140),
            chart_fg: Color::Rgb(136, 192, 208),
            axis_fg: Color::Rgb(216, 222, 233),
            grid_fg: Color::Rgb(76, 86, 106),
            label_fg: Color::Rgb(216, 222, 233),
        }
    }

    fn gruvbox() -> Self {
        Self {
            correct_fg: Color::Rgb(184, 187, 38),
            incorrect_fg: Color::Rgb(204, 36, 29),
            pending_fg: Color::Rgb(102, 92, 84),
            bg_color: Color::Rgb(40, 40, 40),
            stats_fg: Color::Rgb(184, 187, 38),
            chart_fg: Color::Rgb(215, 153, 33),
            axis_fg: Color::Rgb(235, 219, 178),
            grid_fg: Color::Rgb(102, 92, 84),
            label_fg: Color::Rgb(235, 219, 178),
        }
    }

    fn rose_pine() -> Self {
        Self {
            correct_fg: Color::Rgb(152, 203, 161),
            incorrect_fg: Color::Rgb(235, 111, 111),
            pending_fg: Color::Rgb(127, 124, 122),
            bg_color: Color::Rgb(31, 28, 45),
            stats_fg: Color::Rgb(152, 203, 161),
            chart_fg: Color::Rgb(197, 164, 230),
            axis_fg: Color::Rgb(224, 222, 244),
            grid_fg: Color::Rgb(127, 124, 122),
            label_fg: Color::Rgb(224, 222, 244),
        }
    }
}

enum AppMode {
    Typing,
    Results,
}

struct App {
    target_lines: Vec<Vec<char>>,
    input_text: String,
    should_quit: bool,
    theme: Theme,
    theme_index: usize,
    themes: Vec<Theme>,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    mode: AppMode,
    wpm_history: VecDeque<(f64, f64)>,
    last_wpm_sample: Option<Instant>,
    last_wpm_value: f64,
    peak_wpm: f64,
    raw_wpm: f64,
}

impl App {
    fn new(target_text: String) -> Self {
        let themes = vec![
            Theme::default_theme(),
            Theme::dracula(),
            Theme::monokai(),
            Theme::nord(),
            Theme::gruvbox(),
            Theme::rose_pine(),
        ];
        let target_lines: Vec<Vec<char>> =
            target_text.lines().map(|l| l.chars().collect()).collect();
        Self {
            target_lines,
            input_text: String::new(),
            should_quit: false,
            theme: themes[0].clone(),
            theme_index: 0,
            themes,
            start_time: None,
            end_time: None,
            mode: AppMode::Typing,
            wpm_history: VecDeque::new(),
            last_wpm_sample: None,
            last_wpm_value: 0.0,
            peak_wpm: 0.0,
            raw_wpm: 0.0,
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.theme_index].clone();
    }

    fn reset(&mut self) {
        self.input_text = String::new();
        self.start_time = None;
        self.end_time = None;
        self.mode = AppMode::Typing;
        self.wpm_history.clear();
        self.last_wpm_sample = None;
        self.last_wpm_value = 0.0;
        self.peak_wpm = 0.0;
        self.raw_wpm = 0.0;
    }

    fn total_chars(&self) -> usize {
        self.target_lines.iter().map(|l| l.len()).sum::<usize>()
            + self.target_lines.len().saturating_sub(1)
    }

    fn cursor_pos(&self) -> (usize, usize) {
        let input_len = self.input_text.chars().count();
        let mut count = 0;
        for (line_idx, line) in self.target_lines.iter().enumerate() {
            if line_idx > 0 {
                if count + 1 > input_len {
                    return (line_idx, 0);
                }
                count += 1;
            }
            if count + line.len() >= input_len {
                return (line_idx, input_len - count);
            }
            count += line.len();
        }
        let last = self.target_lines.len().saturating_sub(1);
        (last, self.target_lines.last().map(|l| l.len()).unwrap_or(0))
    }

    fn elapsed_secs(&self) -> f64 {
        if let Some(start) = self.start_time {
            let end = self.end_time.unwrap_or_else(Instant::now);
            end.duration_since(start).as_secs_f64()
        } else {
            0.0
        }
    }

    fn count_correct(&self, input_chars: &[char]) -> usize {
        let mut correct = 0;
        let mut offset = 0;
        for line in &self.target_lines {
            for (i, &tc) in line.iter().enumerate() {
                let idx = offset + i;
                if idx < input_chars.len() && tc == input_chars[idx] {
                    correct += 1;
                }
            }
            offset += line.len() + 1;
        }
        correct
    }

    fn wpm(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed < 0.01 {
            return 0.0;
        }
        let input_chars: Vec<char> = self.input_text.chars().collect();
        let correct = self.count_correct(&input_chars);
        let words = correct as f64 / 5.0;
        let minutes = elapsed / 60.0;
        words / minutes
    }

    fn accuracy(&self) -> f64 {
        let input_chars: Vec<char> = self.input_text.chars().collect();
        if input_chars.is_empty() {
            return 0.0;
        }
        let correct = self.count_correct(&input_chars);
        (correct as f64 / input_chars.len() as f64) * 100.0
    }

    fn sample_wpm(&mut self) {
        if let Some(start) = self.start_time {
            let now = Instant::now();
            let elapsed = now.duration_since(start).as_secs_f64();
            if elapsed < 0.5 {
                return;
            }
            if let Some(last_sample) = self.last_wpm_sample {
                let delta = now.duration_since(last_sample).as_secs_f64();
                if delta < 1.0 {
                    return;
                }
            }
            let input_chars: Vec<char> = self.input_text.chars().collect();
            let correct = self.count_correct(&input_chars);
            let words = correct as f64 / 5.0;
            let minutes = elapsed / 60.0;
            let current_wpm = if minutes > 0.0 {
                words / minutes
            } else {
                0.0
            };
            self.last_wpm_value = current_wpm;
            if current_wpm > self.peak_wpm {
                self.peak_wpm = current_wpm;
            }
            self.wpm_history.push_back((elapsed, current_wpm));
            self.last_wpm_sample = Some(now);
        }
    }

    fn calc_raw_wpm(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed < 0.01 {
            return 0.0;
        }
        let total_typed = self.input_text.chars().count() as f64;
        let words = total_typed / 5.0;
        let minutes = elapsed / 60.0;
        words / minutes
    }

    fn avg_wpm(&self) -> f64 {
        if self.wpm_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.wpm_history.iter().map(|(_, w)| w).sum();
        sum / self.wpm_history.len() as f64
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_text = if args.len() > 1 {
        let file_path = &args[1];
        fs::read_to_string(file_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file '{}': {}", file_path, e),
            )
        })?
    } else {
        "the quick brown fox jumps over the lazy dog".to_string()
    };

    let mut terminal = setup_terminal()?;
    let mut app = App::new(target_text);

    let result = run_app(&mut terminal, &mut app);

    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| {
            match app.mode {
                AppMode::Typing => ui_typing(frame, app),
                AppMode::Results => ui_results(frame, &*app),
            }
        })?;

        if let Event::Key(key) = event::read()?
            && (key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat) {
                match app.mode {
                    AppMode::Typing => {
                        match key.code {
                            KeyCode::Esc => app.should_quit = true,
                            KeyCode::Backspace => {
                                app.input_text.pop();
                            }
                            KeyCode::Delete => {
                                app.reset();
                            }
                            KeyCode::Tab => {
                                app.cycle_theme();
                            }
                            KeyCode::Enter => {
                                let total = app.total_chars();
                                if app.input_text.chars().count() < total {
                                    if app.start_time.is_none() {
                                        app.start_time = Some(Instant::now());
                                    }
                                    app.input_text.push('\n');
                                }
                            }
                            KeyCode::Char(c) => {
                                let total = app.total_chars();
                                if app.input_text.chars().count() < total {
                                    if app.start_time.is_none() {
                                        app.start_time = Some(Instant::now());
                                    }
                                    app.input_text.push(c);
                                }
                            }
                            _ => {}
                        }

                        let total = app.total_chars();
                        if app.input_text.chars().count() >= total && app.end_time.is_none() {
                            app.end_time = Some(Instant::now());
                            app.raw_wpm = app.calc_raw_wpm();
                            app.mode = AppMode::Results;
                        }
                    }
                    AppMode::Results => {
                        match key.code {
                            KeyCode::Esc => app.should_quit = true,
                            KeyCode::Delete | KeyCode::Enter | KeyCode::Char(' ') => {
                                app.reset();
                            }
                            KeyCode::Tab => {
                                app.cycle_theme();
                            }
                            _ => {}
                        }
                    }
                }
            }
    }
    Ok(())
}

fn format_time(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.1}s", secs)
    } else {
        let mins = secs.floor() as u64 / 60;
        let s = secs - (mins as f64 * 60.0);
        format!("{}m {:.1}s", mins, s)
    }
}

fn ui_typing(frame: &mut Frame, app: &mut App) {
    app.sample_wpm();

    if app.theme.bg_color != Color::Reset {
        frame.render_widget(
            Block::default().style(Style::default().bg(app.theme.bg_color)),
            frame.area(),
        );
    }

    let num_lines = app.target_lines.len();
    let text_area_height = (num_lines + 2) as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(text_area_height),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let text_area = chunks[1];
    let stats_area = chunks[3];
    let input: Vec<char> = app.input_text.chars().collect();
    let input_len = input.len();
    let total = app.total_chars();
    let is_complete = input_len >= total;

    let mut lines: Vec<Line> = Vec::new();
    let mut char_offset = 0;

    let max_line_width = app.target_lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let area_width = text_area.width;
    let base_padding = area_width.saturating_sub(max_line_width as u16) / 2;

    for target_line in app.target_lines.iter() {
        let mut spans: Vec<Span> = Vec::new();

        if base_padding > 0 {
            spans.push(Span::raw(" ".repeat(base_padding as usize)));
        }

        for (i, &target_char) in target_line.iter().enumerate() {
            let global_idx = char_offset + i;
            if global_idx < input_len {
                let input_char = input[global_idx];
                if target_char == input_char {
                    spans.push(Span::styled(
                        target_char.to_string(),
                        Style::default().fg(app.theme.correct_fg),
                    ));
                } else {
                    spans.push(Span::styled(
                        target_char.to_string(),
                        Style::default()
                            .fg(app.theme.incorrect_fg)
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                }
            } else {
                spans.push(Span::styled(
                    target_char.to_string(),
                    Style::default().fg(app.theme.pending_fg),
                ));
            }
        }

        lines.push(Line::from(spans));
        char_offset += target_line.len() + 1;
    }

    let (cursor_line, cursor_col) = app.cursor_pos();
    let cursor_x = text_area.x + base_padding + cursor_col as u16;
    let cursor_y = text_area.y + cursor_line as u16;

    let text_widget = ratatui::text::Text::from(lines);
    let paragraph = Paragraph::new(text_widget);
    frame.render_widget(paragraph, text_area);

    if !is_complete {
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    let elapsed = app.elapsed_secs();
    let wpm = app.wpm();
    let accuracy = app.accuracy();

    let stats_text = if is_complete {
        format!(
            "  WPM: {:.0}  |  Time: {}  |  Accuracy: {:.1}%  |  Chars: {}  ",
            wpm,
            format_time(elapsed),
            accuracy,
            total
        )
    } else if app.start_time.is_some() {
        format!(
            "  WPM: {:.0}  |  Time: {}  |  Accuracy: {:.1}%  ",
            wpm,
            format_time(elapsed),
            accuracy
        )
    } else {
        String::new()
    };

    if !stats_text.is_empty() {
        let stats = Paragraph::new(stats_text).style(
            Style::default()
                .fg(app.theme.stats_fg)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(stats, stats_area);
    }
}

fn ui_results(frame: &mut Frame, app: &App) {
    if app.theme.bg_color != Color::Reset {
        frame.render_widget(
            Block::default().style(Style::default().bg(app.theme.bg_color)),
            frame.area(),
        );
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let title_area = chunks[0];
    let stats_area = chunks[1];
    let chart_area = chunks[2];
    let footer_area = chunks[3];

    let title = Paragraph::new(" Results ").style(
        Style::default()
            .fg(app.theme.label_fg)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, title_area);

    let elapsed = app.elapsed_secs();
    let wpm = app.wpm();
    let accuracy = app.accuracy();
    let total = app.total_chars();
    let input_chars: Vec<char> = app.input_text.chars().collect();
    let correct = app.count_correct(&input_chars);
    let errors = input_chars.len().saturating_sub(correct);

    let stats_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:.0}", wpm),
                Style::default()
                    .fg(app.theme.chart_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" WPM", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:.0}", app.avg_wpm()),
                Style::default().fg(app.theme.stats_fg),
            ),
            Span::styled(" avg WPM", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:.0}", app.peak_wpm),
                Style::default().fg(app.theme.chart_fg),
            ),
            Span::styled(" peak WPM", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:.0}", app.raw_wpm),
                Style::default().fg(app.theme.stats_fg),
            ),
            Span::styled(" raw WPM", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:.1}%", accuracy),
                Style::default().fg(app.theme.correct_fg),
            ),
            Span::styled(" accuracy", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}", format_time(elapsed)),
                Style::default().fg(app.theme.stats_fg),
            ),
            Span::styled(" time", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}", total),
                Style::default().fg(app.theme.label_fg),
            ),
            Span::styled(" chars", Style::default().fg(app.theme.label_fg)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{}", correct),
                Style::default().fg(app.theme.correct_fg),
            ),
            Span::styled(" correct  ", Style::default().fg(app.theme.label_fg)),
            Span::styled(
                format!("{}", errors),
                Style::default().fg(app.theme.incorrect_fg),
            ),
            Span::styled(" errors", Style::default().fg(app.theme.label_fg)),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.grid_fg))
                .title(" Stats "),
        )
        .alignment(Alignment::Center);
    frame.render_widget(stats_widget, stats_area);

    if !app.wpm_history.is_empty() {
        let max_wpm = app
            .wpm_history
            .iter()
            .map(|(_, w)| *w)
            .fold(0.0_f64, f64::max)
            .ceil();
        let max_time = app
            .wpm_history
            .iter()
            .map(|(t, _)| *t)
            .fold(0.0_f64, f64::max);

        let data_points: Vec<(f64, f64)> = app
            .wpm_history
            .iter()
            .map(|(t, w)| (*t, *w))
            .collect();

        let datasets = vec![Dataset::default()
            .name("WPM")
            .data(&data_points)
            .graph_type(GraphType::Line)
            .marker(Marker::Braille)
            .style(Style::default().fg(app.theme.chart_fg))];

        let x_step = if max_time.ceil() as usize > 5 {
            (max_time.ceil() as usize / 5).max(1)
        } else {
            1
        };
        let y_step = if max_wpm as usize > 5 {
            (max_wpm as usize / 5).max(1)
        } else {
            1
        };

        let x_axis = Axis::default()
            .title("Time (s)")
            .style(Style::default().fg(app.theme.axis_fg))
            .bounds([0.0, max_time])
            .labels(
                (0..=max_time.ceil() as usize)
                    .step_by(x_step)
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>(),
            );

        let y_axis = Axis::default()
            .title("WPM")
            .style(Style::default().fg(app.theme.axis_fg))
            .bounds([0.0, max_wpm])
            .labels(
                (0..=max_wpm as usize)
                    .step_by(y_step)
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>(),
            );

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.grid_fg))
                    .title(" WPM Over Time "),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        frame.render_widget(chart, chart_area);
    }

    let footer = Paragraph::new(" Delete/Enter/Space: retry  |  Tab: theme  |  Esc: quit ")
        .style(
            Style::default()
                .fg(app.theme.label_fg)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(footer, footer_area);
}
