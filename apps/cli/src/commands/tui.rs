use anyhow::Result;
use chrono::{Duration as ChronoDuration, Local, Timelike};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::models::ClaudeSession;
use crate::monitor::{load_sessions, SessionCache};
use crate::spend::{
    compute_daily_spend, compute_model_breakdown, compute_project_breakdown, compute_spend,
};
use crate::format;

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

// ── Screens ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Dashboard = 0,
    Sessions  = 1,
    Analytics = 2,
}

impl Tab {
    fn next(self) -> Tab {
        match self {
            Tab::Dashboard => Tab::Sessions,
            Tab::Sessions  => Tab::Analytics,
            Tab::Analytics => Tab::Dashboard,
        }
    }
    fn prev(self) -> Tab {
        match self {
            Tab::Dashboard => Tab::Analytics,
            Tab::Sessions  => Tab::Dashboard,
            Tab::Analytics => Tab::Sessions,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Sessions  => "Sessions",
            Tab::Analytics => "Analytics",
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    tab:              Tab,
    session_cursor:   usize,
    session_scroll:   usize,
    analytics_scroll: usize,
    detail_open:      Option<usize>,
    sessions:         Vec<ClaudeSession>,
    cache:            SessionCache,
    last_refresh:     Instant,
}

impl App {
    fn new() -> Self {
        let mut cache    = SessionCache::new();
        let sessions     = load_sessions(&mut cache);
        Self {
            tab: Tab::Dashboard,
            session_cursor:   0,
            session_scroll:   0,
            analytics_scroll: 0,
            detail_open:      None,
            sessions,
            cache,
            last_refresh: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        self.sessions = load_sessions(&mut self.cache);
        if self.session_cursor >= self.sessions.len() && !self.sessions.is_empty() {
            self.session_cursor = self.sessions.len() - 1;
        }
        self.last_refresh = Instant::now();
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let result = event_loop(&mut term);

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}

// ── Event loop ────────────────────────────────────────────────────────────────

fn event_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        let viewport_height = term.size().map(|r| r.height as usize).unwrap_or(24);
        term.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key.code, key.modifiers, viewport_height) {
                    return Ok(());
                }
            }
        }

        if app.last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh();
        }
    }
}

/// Returns `true` if the app should quit.
fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers, viewport_h: usize) -> bool {
    // Detail overlay gets first priority
    if app.detail_open.is_some() {
        match code {
            KeyCode::Esc | KeyCode::Backspace => { app.detail_open = None; }
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') => {
                if let Some(idx) = app.detail_open {
                    if let Some(s) = app.sessions.get(idx) {
                        let _ = Command::new("pbcopy")
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                use std::io::Write;
                                child.stdin.as_mut().unwrap().write_all(s.project_path.as_bytes())?;
                                child.wait()?;
                                Ok(())
                            });
                    }
                }
            }
            _ => {}
        }
        return false;
    }

    match (code, mods) {
        (KeyCode::Char('q'), _) |
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return true,

        (KeyCode::Char('r'), _) => { app.refresh(); }

        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
            app.tab = app.tab.next();
            app.analytics_scroll = 0;
        }
        (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
            app.tab = app.tab.prev();
            app.analytics_scroll = 0;
        }

        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
            match app.tab {
                Tab::Sessions => {
                    if app.session_cursor > 0 {
                        app.session_cursor -= 1;
                        if app.session_cursor < app.session_scroll {
                            app.session_scroll = app.session_cursor;
                        }
                    }
                }
                Tab::Analytics => {
                    if app.analytics_scroll > 0 { app.analytics_scroll -= 1; }
                }
                Tab::Dashboard => {}
            }
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
            match app.tab {
                Tab::Sessions => {
                    let max = app.sessions.len().saturating_sub(1);
                    if app.session_cursor < max {
                        app.session_cursor += 1;
                        let visible = viewport_h.saturating_sub(5);
                        if app.session_cursor >= app.session_scroll + visible {
                            app.session_scroll = app.session_cursor + 1 - visible;
                        }
                    }
                }
                Tab::Analytics => { app.analytics_scroll += 1; }
                Tab::Dashboard => {}
            }
        }

        (KeyCode::Enter, _) => {
            if app.tab == Tab::Sessions && !app.sessions.is_empty() {
                app.detail_open = Some(app.session_cursor);
            }
        }

        _ => {}
    }
    false
}

// ── Top-level draw ────────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_tab_bar(f, chunks[0], app);

    match app.tab {
        Tab::Dashboard => draw_dashboard(f, chunks[1], app),
        Tab::Sessions  => draw_sessions_screen(f, chunks[1], app),
        Tab::Analytics => draw_analytics_screen(f, chunks[1], app),
    }

    draw_footer(f, chunks[2], app);

    if let Some(idx) = app.detail_open {
        if let Some(session) = app.sessions.get(idx) {
            draw_detail_overlay(f, area, session);
        }
    }
}

// ── Tab bar ───────────────────────────────────────────────────────────────────

fn draw_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let has_active = app.sessions.iter().any(|s| s.is_active);

    let mut spans = vec![Span::styled("  CLAUX  ", Style::default().fg(Color::DarkGray))];

    for tab in [Tab::Dashboard, Tab::Sessions, Tab::Analytics] {
        let is_selected = app.tab == tab;
        let label = if tab == Tab::Dashboard && has_active {
            format!("  ● {}  ", tab.label())
        } else {
            format!("  {}  ", tab.label())
        };
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(label, style));
    }

    let used: usize    = spans.iter().map(|s| s.content.len()).sum();
    let version        = format!("  v{}  ", env!("CARGO_PKG_VERSION"));
    let pad            = (area.width as usize).saturating_sub(used + version.len());
    spans.push(Span::styled(" ".repeat(pad), Style::default()));
    spans.push(Span::styled(version, Style::default().fg(Color::DarkGray)));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Reset)),
        area,
    );
}

// ── Dashboard screen ──────────────────────────────────────────────────────────
//
//  ┌─ Active Session ──────────────────────────────┐
//  │  ● ~/project  ·  Sonnet 4.6  ·  2h 14m       │  (7 or 4 rows)
//  │  $2.34  ·  $0.89/hr  ·  Context 67%  ·  Cache │
//  │  [═══════════════════════════════╌╌╌╌╌╌╌╌] 67%│
//  └───────────────────────────────────────────────┘
//  ┌─ Tokens ──────────────┐  ┌─ Insights ─────────┐
//  │  Input    1.2M  ████  │  │  Cache 42%  C Fair  │  (min 6 rows)
//  │  Output  345K   ███   │  │  Context 67%  ✓     │
//  │  Cache R  789K  ████  │  │  Est. today  $4.20  │
//  └───────────────────────┘  └────────────────────┘
//  ┌─ Spend ───────────────────────────────────────┐
//  │    Today $2.34  ↑  │  Week $12.56  ↓  │ Month │  (5 rows)
//  └───────────────────────────────────────────────┘

fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let active   = app.sessions.iter().find(|s| s.is_active);
    let active_h = if active.is_some() { 8u16 } else { 4u16 };
    let spend_h  = 5u16;
    let mid_h    = area.height.saturating_sub(active_h + spend_h).max(6);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(active_h),
            Constraint::Length(mid_h),
            Constraint::Length(spend_h),
        ])
        .split(area);

    draw_active_panel(f, chunks[0], active);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[1]);

    draw_token_breakdown(f, mid[0], active);
    draw_insights_panel(f, mid[1], active, &app.sessions);

    draw_spend_panel(f, chunks[2], &app.sessions);
}

// ── Sessions screen ───────────────────────────────────────────────────────────

fn draw_sessions_screen(f: &mut Frame, area: Rect, app: &App) {
    draw_sessions_list(f, area, &app.sessions, Some(app.session_cursor), app.session_scroll);
}

// ── Analytics screen ──────────────────────────────────────────────────────────
//
//  ┌─ Last 7 Days ─────────────────────────────────────────────────────────────┐
//  │  Total $X  Avg $Y/day  Peak $Z (Mon)                                      │
//  │                                                                            │
//  │        ██                                                                  │
//  │   ██   ██        ██                                                        │
//  │   ██   ██   ██   ██   ██   ██   ██                                         │
//  │  ─────────────────────────────────                                         │
//  │   Mon  Tue  Wed  Thu  Fri  Sat  Sun                                        │
//  │  $0.45 $1.23 ...                                                           │
//  └────────────────────────────────────────────────────────────────────────────┘
//  ┌─ 30-Day Trend ──────────────────────────────────────────────────────────── ┐
//  └────────────────────────────────────────────────────────────────────────────┘
//  ┌─ By Project ────────────────────┐  ┌─ By Model ─────────────────────────── ┐
//  └─────────────────────────────────┘  └────────────────────────────────────── ┘

fn draw_analytics_screen(f: &mut Frame, area: Rect, app: &App) {
    let daily    = compute_daily_spend(&app.sessions);
    let projects = compute_project_breakdown(&app.sessions);
    let models   = compute_model_breakdown(&app.sessions);

    // Model output-token map for efficiency calculation
    let mut model_output: HashMap<String, u64> = HashMap::new();
    for s in &app.sessions {
        *model_output.entry(s.model.clone()).or_insert(0) += s.token_usage.output_tokens;
    }

    let chart7_h    = area.height * 50 / 100;
    let sparkline_h = 4u16;
    let tables_h    = area.height.saturating_sub(chart7_h + sparkline_h);
    let proj_h      = tables_h * 55 / 100;
    let model_h     = tables_h.saturating_sub(proj_h);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(chart7_h.max(8)),
            Constraint::Length(sparkline_h),
            Constraint::Length(proj_h.max(4)),
            Constraint::Length(model_h.max(4)),
        ])
        .split(area);

    draw_7day_chart(f, chunks[0], &daily);
    draw_30day_sparkline(f, chunks[1], &daily);

    // Project | Model side by side
    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[2]);

    draw_project_table(f, bottom_cols[0], &projects, app.analytics_scroll);
    draw_model_table(f, bottom_cols[1], &models, &model_output);
}

// ── 7-day vertical bar chart ─────────────────────────────────────────────────

fn draw_7day_chart(f: &mut Frame, area: Rect, daily: &[crate::models::DailySpend]) {
    let today = Local::now().date_naive();

    // Build exactly 7 days: [today-6 .. today], filling missing days with $0
    let days: Vec<(chrono::NaiveDate, f64)> = (0i64..7)
        .map(|i| today - ChronoDuration::days(6 - i))
        .map(|date| {
            let cost = daily.iter().find(|d| d.date == date).map(|d| d.cost).unwrap_or(0.0);
            (date, cost)
        })
        .collect();

    let block = Block::default()
        .title(" Last 7 Days ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 6 || inner.width < 28 {
        return;
    }

    let costs: Vec<f64>  = days.iter().map(|(_, c)| *c).collect();
    let total: f64       = costs.iter().sum();
    let max_cost: f64    = costs.iter().cloned().fold(0.0f64, f64::max);
    let avg              = total / 7.0;

    let peak_str = days.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(d, c)| format!("{}  ({})", format::cost(*c), d.format("%a %b %d")))
        .unwrap_or_default();

    let summary = format!(
        "  Total {}    Avg {}/day    Peak {}",
        format::cost(total),
        format::cost(avg),
        peak_str,
    );

    // Layout within inner:
    //   row 0      : summary
    //   row 1      : blank gap
    //   row 2..(2+bar_rows-1): bars
    //   row 2+bar_rows       : axis ─────
    //   row 2+bar_rows+1     : day labels
    //   row 2+bar_rows+2     : cost labels
    // Total used = bar_rows + 5

    let bar_rows = (inner.height as usize).saturating_sub(5).max(1);
    let n        = days.len(); // always 7
    let col_w    = (inner.width as usize) / n;
    if col_w == 0 { return; }

    let bar_w   = col_w.saturating_sub(2).min(5).max(1);
    let pad_l   = (col_w.saturating_sub(bar_w)) / 2;
    let pad_r   = col_w.saturating_sub(pad_l + bar_w);

    // Fill height for each day
    let fill: Vec<usize> = costs.iter().map(|&c| {
        if max_cost > 0.0 {
            ((c / max_cost) * bar_rows as f64).round() as usize
        } else { 0 }
    }).collect();

    let mut lines: Vec<Line> = vec![];

    // Summary
    lines.push(Line::from(Span::styled(summary, Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(""));

    // Bar rows (row 0 = top → highest values)
    for r in 0..bar_rows {
        let threshold = bar_rows - r; // fill must be >= threshold to be visible at this height
        let mut spans: Vec<Span> = vec![];

        for (i, &fill_h) in fill.iter().enumerate() {
            let is_today = days[i].0 == today;

            if fill_h >= threshold {
                let bar_color = if is_today { Color::Blue } else { Color::Cyan };
                if pad_l > 0 { spans.push(Span::raw(" ".repeat(pad_l))); }
                spans.push(Span::styled("█".repeat(bar_w), Style::default().fg(bar_color)));
                if pad_r > 0 { spans.push(Span::raw(" ".repeat(pad_r))); }
            } else {
                spans.push(Span::raw(" ".repeat(col_w)));
            }
        }
        lines.push(Line::from(spans));
    }

    // Axis
    lines.push(Line::from(Span::styled(
        "─".repeat(col_w * n),
        Style::default().fg(Color::DarkGray),
    )));

    // Day labels — e.g. "Mon"
    let day_spans: Vec<Span> = days.iter().map(|(d, _)| {
        let is_today = *d == today;
        let lbl = format!("{:^width$}", d.format("%a"), width = col_w);
        let style = if is_today {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        Span::styled(lbl, style)
    }).collect();
    lines.push(Line::from(day_spans));

    // Cost labels — e.g. "$1.23"
    let cost_spans: Vec<Span> = days.iter().map(|(d, c)| {
        let is_today = *d == today;
        let lbl = format!("{:^width$}", format::cost(*c), width = col_w);
        let style = if is_today {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        Span::styled(lbl, style)
    }).collect();
    lines.push(Line::from(cost_spans));

    let chart_area = Rect {
        height: (lines.len() as u16).min(inner.height),
        ..inner
    };
    f.render_widget(Paragraph::new(lines), chart_area);
}

// ── 30-day compact sparkline ─────────────────────────────────────────────────

fn draw_30day_sparkline(f: &mut Frame, area: Rect, daily: &[crate::models::DailySpend]) {
    let costs: Vec<f64> = daily.iter().map(|d| d.cost).collect();
    let max_cost        = costs.iter().cloned().fold(0.0f64, f64::max);
    let total: f64      = costs.iter().sum();

    let block = Block::default()
        .title(format!(" 30-Day Trend  ({} total) ", format::cost(total)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if max_cost > 0.0 && inner.height >= 2 {
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let bar_line: String = costs.iter().map(|&c| {
            let level = ((c / max_cost) * (bars.len() - 1) as f64).round() as usize;
            bars[level.min(bars.len() - 1)]
        }).collect();

        let first = daily.first().map(|d| d.date.format("%b %d").to_string()).unwrap_or_default();
        let last  = daily.last().map(|d| d.date.format("%b %d").to_string()).unwrap_or_default();
        let pad   = (inner.width as usize).saturating_sub(first.len() + last.len() + 2);

        let lines = vec![
            Line::from(Span::styled(bar_line, Style::default().fg(Color::DarkGray))),
            Line::from(vec![
                Span::styled(first, Style::default().fg(Color::DarkGray)),
                Span::raw(" ".repeat(pad.max(1))),
                Span::styled(last, Style::default().fg(Color::DarkGray)),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), inner);
    } else {
        f.render_widget(
            Paragraph::new("No spend data.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            inner,
        );
    }
}

// ── Project breakdown table ───────────────────────────────────────────────────

fn draw_project_table(
    f: &mut Frame,
    area: Rect,
    projects: &[crate::models::ProjectSpend],
    scroll: usize,
) {
    let block = Block::default()
        .title(" By Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_rows = inner.height as usize;
    let visible: Vec<&crate::models::ProjectSpend> = projects.iter().skip(scroll).take(max_rows).collect();

    let rows: Vec<Row> = visible.iter().map(|p| {
        let max_path = (inner.width as usize).saturating_sub(22).max(10);
        let path = if p.display_path.len() > max_path {
            format!("…{}", &p.display_path[p.display_path.len().saturating_sub(max_path - 1)..])
        } else {
            p.display_path.clone()
        };
        Row::new(vec![
            Cell::from(path),
            Cell::from(p.session_count.to_string()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format::cost(p.total_cost)).style(Style::default().fg(Color::White)),
        ])
    }).collect();

    let table = Table::new(rows, [Constraint::Min(20), Constraint::Length(5), Constraint::Length(8)])
        .header(
            Row::new(["Project", "Sess", "Cost"])
                .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        )
        .column_spacing(1);

    f.render_widget(table, inner);
}

// ── Model breakdown table with efficiency ─────────────────────────────────────

fn draw_model_table(
    f: &mut Frame,
    area: Rect,
    models: &[crate::models::ModelSpend],
    model_output: &HashMap<String, u64>,
) {
    let block = Block::default()
        .title(" By Model ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<Row> = models.iter().take(inner.height as usize).map(|m| {
        let color      = model_color_for(&m.model);
        let output_tok = model_output.get(&m.model).copied().unwrap_or(0);
        // K output tokens per dollar — higher is more efficient
        let efficiency = if m.total_cost > 0.0 {
            format!("{:.0}K/$", output_tok as f64 / m.total_cost / 1_000.0)
        } else {
            "—".to_string()
        };
        Row::new(vec![
            Cell::from(m.display_name.clone()).style(Style::default().fg(color)),
            Cell::from(m.session_count.to_string()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format::cost(m.total_cost)).style(Style::default().fg(Color::White)),
            Cell::from(efficiency).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Min(12),
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(8),
    ])
    .header(
        Row::new(["Model", "Sess", "Cost", "Effic."])
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
    )
    .column_spacing(1);

    f.render_widget(table, inner);
}

// ── Token breakdown panel ─────────────────────────────────────────────────────

fn draw_token_breakdown(f: &mut Frame, area: Rect, session: Option<&ClaudeSession>) {
    let block = Block::default()
        .title(" Tokens ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(s) = session else {
        f.render_widget(
            Paragraph::new("  No active session")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let tu = &s.token_usage;

    // label(10) + count(7) + space(1) + bar = inner.width
    let bar_w = (inner.width as usize).saturating_sub(19).max(2);
    let max_tok = [
        tu.input_tokens,
        tu.output_tokens,
        tu.cache_read_tokens,
        tu.cache_write_tokens,
        tu.thinking_tokens,
    ]
    .iter()
    .cloned()
    .max()
    .unwrap_or(1)
    .max(1);

    let entries: &[(&str, u64, Color)] = &[
        ("Input   ", tu.input_tokens,       Color::White),
        ("Output  ", tu.output_tokens,      Color::Cyan),
        ("Cache R ", tu.cache_read_tokens,  Color::Blue),
        ("Cache W ", tu.cache_write_tokens, Color::DarkGray),
        ("Thinking", tu.thinking_tokens,    Color::Magenta),
    ];

    let mut lines: Vec<Line> = entries.iter().map(|(label, count, color)| {
        let filled = if max_tok > 0 {
            ((*count as f64 / max_tok as f64) * bar_w as f64).round() as usize
        } else { 0 };
        let empty = bar_w.saturating_sub(filled);

        Line::from(vec![
            Span::styled(format!("  {:<9}", label), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:>6} ", format::tokens(*count)), Style::default().fg(*color)),
            Span::styled("█".repeat(filled), Style::default().fg(*color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        ])
    }).collect();

    // Summary stats
    let total = tu.input_tokens + tu.output_tokens + tu.cache_read_tokens
        + tu.cache_write_tokens + tu.thinking_tokens;
    let cache_pct = tu.cache_hit_rate() * 100.0;

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Total   ", Style::default().fg(Color::DarkGray)),
        Span::styled(format::tokens(total), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("   Cache hit ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:.0}%", cache_pct),
            Style::default()
                .fg(if cache_pct >= 60.0 { Color::Green } else if cache_pct >= 30.0 { Color::Yellow } else { Color::Red })
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}

// ── Insights panel ────────────────────────────────────────────────────────────

fn draw_insights_panel(
    f: &mut Frame,
    area: Rect,
    session: Option<&ClaudeSession>,
    all: &[ClaudeSession],
) {
    let block = Block::default()
        .title(" Insights ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = vec![];

    if let Some(s) = session {
        let cache_pct = s.token_usage.cache_hit_rate() * 100.0;
        let ctx_pct   = s.context_health_fraction() * 100.0;
        let burn      = s.burn_rate_per_hour();

        // ── Cache efficiency ──────────────────────────────────────────────
        let (grade, grade_color, tip) = if cache_pct >= 70.0 {
            ("A  Excellent", Color::Green,  None)
        } else if cache_pct >= 50.0 {
            ("B  Good",      Color::Green,  None)
        } else if cache_pct >= 30.0 {
            ("C  Fair",      Color::Yellow, Some("→ reuse system prompts"))
        } else {
            ("D  Low",       Color::Red,    Some("→ add persistent system prompt"))
        };

        lines.push(Line::from(vec![
            Span::styled("  Cache  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%  ", cache_pct),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(grade, Style::default().fg(grade_color)),
        ]));
        if let Some(t) = tip {
            lines.push(Line::from(Span::styled(
                format!("           {}", t),
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::from(""));

        // ── Context health ────────────────────────────────────────────────
        let (ctx_label, ctx_color) = if ctx_pct >= 90.0 {
            ("⚠  Run /compact now!", Color::Red)
        } else if ctx_pct >= 75.0 {
            ("↑  Consider /compact", Color::Yellow)
        } else {
            ("✓  Healthy",           Color::Green)
        };

        lines.push(Line::from(vec![
            Span::styled("  Context ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%  ", ctx_pct),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(ctx_label, Style::default().fg(ctx_color)),
        ]));
        lines.push(Line::from(""));

        // ── Cost projection ───────────────────────────────────────────────
        if burn > 0.0 {
            let now        = Local::now();
            let hours_left = 24.0 - now.hour() as f64 - now.minute() as f64 / 60.0;
            let projected  = s.total_cost + burn * hours_left;
            let week_proj  = projected * 5.0; // rough 5 working days

            lines.push(Line::from(vec![
                Span::styled("  Session    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format::cost(s.total_cost),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  @ {}/hr", format::cost(burn)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Est. today ", Style::default().fg(Color::DarkGray)),
                Span::styled(format::cost(projected), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Est. week  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format::cost(week_proj), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
        }

        // ── Model info ────────────────────────────────────────────────────
        let model       = format::model_short_name(&s.model);
        let model_color = model_color_for(&s.model);
        lines.push(Line::from(vec![
            Span::styled("  Model  ", Style::default().fg(Color::DarkGray)),
            Span::styled(model, Style::default().fg(model_color).add_modifier(Modifier::BOLD)),
        ]));

        // Thinking tokens %
        if s.token_usage.thinking_tokens > 0 {
            let think_pct = s.token_usage.thinking_tokens as f64
                / s.token_usage.output_tokens.max(1) as f64
                * 100.0;
            lines.push(Line::from(vec![
                Span::styled("  Extended ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0}% thinking", think_pct),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }

        // Output efficiency (K output tokens per dollar)
        if s.total_cost > 0.0 && s.token_usage.output_tokens > 0 {
            let efficiency = s.token_usage.output_tokens as f64 / s.total_cost / 1_000.0;
            lines.push(Line::from(vec![
                Span::styled("  Effic.   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0}K tok/$", efficiency),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

    } else {
        // ── No active session: lifetime stats ────────────────────────────
        let total_sess     = all.len();
        let lifetime_cost: f64 = all.iter().map(|s| s.total_cost).sum();
        let avg_cost       = if total_sess > 0 { lifetime_cost / total_sess as f64 } else { 0.0 };

        let total_input:   u64 = all.iter().map(|s| s.token_usage.input_tokens).sum();
        let total_output:  u64 = all.iter().map(|s| s.token_usage.output_tokens).sum();
        let total_cache_r: u64 = all.iter().map(|s| s.token_usage.cache_read_tokens).sum();
        let total_cache_w: u64 = all.iter().map(|s| s.token_usage.cache_write_tokens).sum();

        let overall_cache = {
            let denom = total_input + total_cache_r + total_cache_w;
            if denom > 0 { total_cache_r as f64 / denom as f64 * 100.0 } else { 0.0 }
        };

        let best_session = all.iter()
            .filter(|s| s.token_usage.cache_hit_rate() > 0.0)
            .max_by(|a, b| a.token_usage.cache_hit_rate().partial_cmp(&b.token_usage.cache_hit_rate()).unwrap());

        lines.push(Line::from(Span::styled("  ○  No active session", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Sessions    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                total_sess.to_string(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Lifetime    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format::cost(lifetime_cost),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Avg/session ", Style::default().fg(Color::DarkGray)),
            Span::styled(format::cost(avg_cost), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Output tok  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format::tokens(total_output), Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Cache hit   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%", overall_cache),
                Style::default()
                    .fg(if overall_cache >= 50.0 { Color::Green } else { Color::Yellow })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if let Some(best) = best_session {
            lines.push(Line::from(""));
            let best_pct = best.token_usage.cache_hit_rate() * 100.0;
            lines.push(Line::from(vec![
                Span::styled("  Best cache  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0}%  {}", best_pct, best.display_path()),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
    }

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}

// ── Active session panel ──────────────────────────────────────────────────────

fn draw_active_panel(f: &mut Frame, area: Rect, session: Option<&ClaudeSession>) {
    let block = Block::default()
        .title(" Active Session ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    match session {
        None => {
            let p = Paragraph::new("○  No active session")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(p, inner);
        }
        Some(s) => {
            let path    = s.display_path();
            let model   = format::model_short_name(&s.model);
            let dur     = format::duration(s.duration_secs());
            let cost    = format::cost(s.total_cost);
            let burn    = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
            let frac    = s.context_health_fraction();
            let ctx     = format!("{:.0}%", frac * 100.0);
            let cache   = format!("{:.0}%", s.token_usage.cache_hit_rate() * 100.0);
            let color   = context_color(frac);
            let turns   = s.token_usage.output_tokens; // rough proxy for turns

            let rows = vec![
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Green)),
                    Span::styled(path, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("  ·  "),
                    Span::styled(model, Style::default().fg(model_color_for(&s.model))),
                    Span::raw("  ·  "),
                    Span::styled(dur, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("     Cost "),
                    Span::styled(&cost, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("   Burn "),
                    Span::styled(&burn, Style::default().fg(Color::DarkGray)),
                    Span::raw("   Context "),
                    Span::styled(&ctx, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw("   Cache "),
                    Span::styled(&cache, Style::default().fg(Color::DarkGray)),
                    Span::raw("   Output "),
                    Span::styled(format::tokens(turns), Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let text_area  = Rect { height: 2, ..inner };
            let gauge_area = Rect { x: inner.x, y: inner.y + 3, width: inner.width, height: 1 };

            f.render_widget(Paragraph::new(rows), text_area);

            if gauge_area.y < area.y + area.height {
                f.render_widget(
                    Gauge::default()
                        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                        .ratio(frac),
                    gauge_area,
                );
            }
        }
    }
}

// ── Spend summary panel ───────────────────────────────────────────────────────

fn draw_spend_panel(f: &mut Frame, area: Rect, sessions: &[ClaudeSession]) {
    let s = compute_spend(sessions);

    let block = Block::default()
        .title(" Spend ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

    render_spend_cell(f, cols[0], "Today",      s.today,      s.yesterday);
    render_spend_cell(f, cols[1], "This week",  s.this_week,  s.prev_week);
    render_spend_cell(f, cols[2], "This month", s.this_month, s.prev_month);
}

fn render_spend_cell(f: &mut Frame, area: Rect, label: &str, current: f64, prev: f64) {
    let (trend_str, trend_color) = if prev > 0.0 {
        let diff = current - prev;
        if diff >= 0.0 {
            (format!("↑ {}", format::cost(diff)), Color::Red)
        } else {
            (format!("↓ {}", format::cost(-diff)), Color::Green)
        }
    } else {
        (String::new(), Color::DarkGray)
    };

    let lines = vec![
        Line::from(Span::styled(label, Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(
            format::cost(current),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(trend_str, Style::default().fg(trend_color))),
    ];
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

// ── Session list (shared: Dashboard + Sessions tab) ───────────────────────────

fn draw_sessions_list(
    f: &mut Frame,
    area: Rect,
    sessions: &[ClaudeSession],
    cursor: Option<usize>,
    scroll: usize,
) {
    let block = Block::default()
        .title(" Sessions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if sessions.is_empty() {
        f.render_widget(
            Paragraph::new("No sessions found.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let header_h = 1usize;
    let max_rows = (inner.height as usize).saturating_sub(header_h);
    let visible: Vec<(usize, &ClaudeSession)> = sessions
        .iter()
        .enumerate()
        .skip(scroll)
        .take(max_rows)
        .collect();

    let rows: Vec<Row> = visible.iter().map(|(abs_idx, s)| {
        let is_selected = cursor == Some(*abs_idx);
        let row_style   = if is_selected { Style::default().bg(Color::DarkGray) } else { Style::default() };

        let dot   = if s.is_active { "●" } else { "○" };
        let dot_s = Style::default().fg(if s.is_active { Color::Green } else { Color::DarkGray });
        let when  = format::relative_time(&s.start_time);
        let dur   = format::duration(s.duration_secs());
        let model = format::model_short_name(&s.model);
        let path  = s.display_path();
        let title = s.title.as_deref().unwrap_or(path.as_str());
        let label = if title.len() > 38 { format!("{}…", &title[..37]) } else { title.to_string() };
        let cost  = format::cost(s.total_cost);

        Row::new(vec![
            Cell::from(dot).style(dot_s),
            Cell::from(when).style(Style::default().fg(Color::DarkGray)),
            Cell::from(dur).style(Style::default().fg(Color::DarkGray)),
            Cell::from(model).style(Style::default().fg(model_color_for(&s.model))),
            Cell::from(label),
            Cell::from(cost).style(Style::default().fg(Color::White)),
        ])
        .style(row_style)
    }).collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Min(20),
        Constraint::Length(8),
    ];

    let hint  = if cursor.is_some() { " [Enter] detail" } else { "" };
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("When"),
                Cell::from("Dur"),
                Cell::from("Model"),
                Cell::from(format!("Session{}", hint)),
                Cell::from("Cost"),
            ])
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        )
        .column_spacing(1);

    f.render_widget(table, inner);

    // Scroll indicator
    if sessions.len() > max_rows && max_rows > 0 {
        let pct    = scroll as f64 / (sessions.len() - max_rows).max(1) as f64;
        let thumb  = inner.y + (pct * (inner.height as f64 - 1.0)).round() as u16;
        if thumb < inner.y + inner.height {
            f.render_widget(
                Paragraph::new("▐").style(Style::default().fg(Color::DarkGray)),
                Rect { x: inner.x + inner.width - 1, y: thumb, width: 1, height: 1 },
            );
        }
    }
}

// ── Session detail overlay ────────────────────────────────────────────────────

fn draw_detail_overlay(f: &mut Frame, area: Rect, s: &ClaudeSession) {
    let popup = centered_rect(80, 85, area);
    f.render_widget(Clear, popup);

    let title = s.title.as_deref().unwrap_or("Session Detail");
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let model       = format::model_short_name(&s.model);
    let model_color = model_color_for(&s.model);
    let dur         = format::duration(s.duration_secs());
    let cost        = format::cost(s.total_cost);
    let burn        = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
    let ctx_pct     = format!("{:.0}%", s.context_health_fraction() * 100.0);
    let cache_pct   = format!("{:.0}%", s.token_usage.cache_hit_rate() * 100.0);
    let ctx_color   = context_color(s.context_health_fraction());

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                s.display_path(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  "),
            Span::styled(model, Style::default().fg(model_color)),
            if s.is_active {
                Span::styled("  ● Active", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            stat_span("Cost",      &cost,    Color::White),
            Span::raw("   "),
            stat_span("Duration",  &dur,     Color::White),
            Span::raw("   "),
            stat_span("Burn",      &burn,    Color::DarkGray),
        ]),
        Line::from(vec![
            stat_span("Context",   &ctx_pct,   ctx_color),
            Span::raw("   "),
            stat_span("Cache hit", &cache_pct, Color::DarkGray),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Tokens",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )),
        token_line("Input",       s.token_usage.input_tokens,       Color::White),
        token_line("Output",      s.token_usage.output_tokens,      Color::Cyan),
        token_line("Cache read",  s.token_usage.cache_read_tokens,  Color::Blue),
        token_line("Cache write", s.token_usage.cache_write_tokens, Color::DarkGray),
    ];

    if s.token_usage.thinking_tokens > 0 {
        lines.push(token_line("Thinking", s.token_usage.thinking_tokens, Color::Magenta));
    }

    if let Some(score) = s.claudemd_score {
        lines.push(Line::from(""));
        let sc = if score >= 70 { Color::Green } else if score >= 40 { Color::Yellow } else { Color::Red };
        lines.push(Line::from(vec![
            Span::styled("CLAUDE.md  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} / 100", score),
                Style::default().fg(sc).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Context gauge
    let gauge_y = inner.y + lines.len() as u16 + 1;
    if gauge_y < inner.y + inner.height {
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(ctx_color).bg(Color::DarkGray))
                .ratio(s.context_health_fraction()),
            Rect { x: inner.x, y: gauge_y, width: inner.width, height: 1 },
        );
    }

    // Footer line
    let footer_y = inner.y + inner.height.saturating_sub(1);
    if footer_y > inner.y {
        let path_short = if s.project_path.len() > (inner.width as usize).saturating_sub(20) {
            format!(
                "…{}",
                &s.project_path[s.project_path.len().saturating_sub(inner.width as usize - 22)..]
            )
        } else {
            s.project_path.clone()
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(path_short, Style::default().fg(Color::DarkGray)),
                Span::styled("  [c] copy path  [Esc] back", Style::default().fg(Color::DarkGray)),
            ])),
            Rect { x: inner.x, y: footer_y, width: inner.width, height: 1 },
        );
    }

    f.render_widget(
        Paragraph::new(lines.clone()),
        Rect {
            height: (lines.len() as u16).min(inner.height),
            ..inner
        },
    );
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let spans: Vec<Span> = if app.detail_open.is_some() {
        vec![
            Span::styled("  [Esc] back  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[c] copy path  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        let mut v = vec![Span::styled(
            "  [←/→] switch  ",
            Style::default().fg(Color::DarkGray),
        )];
        let extra: &[(&str, Color)] = match app.tab {
            Tab::Dashboard => &[("[r] refresh  ", Color::DarkGray), ("[q] quit", Color::DarkGray)],
            Tab::Sessions  => &[("[↑/↓] select  ", Color::DarkGray), ("[Enter] detail  ", Color::DarkGray), ("[r] refresh  ", Color::DarkGray), ("[q] quit", Color::DarkGray)],
            Tab::Analytics => &[("[↑/↓] scroll  ", Color::DarkGray), ("[r] refresh  ", Color::DarkGray), ("[q] quit", Color::DarkGray)],
        };
        for (txt, col) in extra {
            v.push(Span::styled(*txt, Style::default().fg(*col)));
        }
        v
    };

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn model_color_for(model: &str) -> Color {
    let l = model.to_lowercase();
    if l.contains("opus")       { Color::Magenta }
    else if l.contains("haiku") { Color::Green   }
    else                        { Color::Blue     }
}

fn context_color(fraction: f64) -> Color {
    if fraction < 0.70      { Color::Blue   }
    else if fraction < 0.90 { Color::Yellow }
    else                    { Color::Red    }
}

fn stat_span<'a>(label: &'a str, value: &'a str, color: Color) -> Span<'a> {
    Span::styled(
        format!("{}  {}", label, value),
        Style::default().fg(color),
    )
}

fn token_line(label: &str, count: u64, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<14}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format::tokens(count),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_w = area.width  * percent_x / 100;
    let popup_h = area.height * percent_y / 100;
    Rect {
        x:      area.x + (area.width  - popup_w) / 2,
        y:      area.y + (area.height - popup_h) / 2,
        width:  popup_w,
        height: popup_h,
    }
}
