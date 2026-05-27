use anyhow::Result;
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
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::models::ClaudeSession;
use crate::monitor::{load_sessions, SessionCache};
use crate::spend::{compute_daily_spend, compute_model_breakdown, compute_project_breakdown, compute_spend};
use crate::format;

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

// ── Screens ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab { Dashboard = 0, Sessions = 1, Analytics = 2 }

impl Tab {
    fn next(self) -> Tab {
        match self { Tab::Dashboard => Tab::Sessions, Tab::Sessions => Tab::Analytics, Tab::Analytics => Tab::Dashboard }
    }
    fn prev(self) -> Tab {
        match self { Tab::Dashboard => Tab::Analytics, Tab::Sessions => Tab::Dashboard, Tab::Analytics => Tab::Sessions }
    }
    fn label(self) -> &'static str {
        match self { Tab::Dashboard => "Dashboard", Tab::Sessions => "Sessions", Tab::Analytics => "Analytics" }
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    tab:              Tab,
    session_cursor:   usize,
    session_scroll:   usize,
    analytics_scroll: usize,
    detail_open:      Option<usize>,   // index into sessions[]
    sessions:         Vec<ClaudeSession>,
    cache:            SessionCache,
    last_refresh:     Instant,
}

impl App {
    fn new() -> Self {
        let mut cache = SessionCache::new();
        let sessions  = load_sessions(&mut cache);
        Self {
            tab: Tab::Dashboard,
            session_cursor: 0,
            session_scroll: 0,
            analytics_scroll: 0,
            detail_open: None,
            sessions,
            cache,
            last_refresh: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        self.sessions = load_sessions(&mut self.cache);
        // Keep cursor in bounds after refresh
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
        // Compute viewport height for scroll calculations before borrowing term
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

    // Global keys
    match (code, mods) {
        (KeyCode::Char('q'), _) |
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return true,

        (KeyCode::Char('r'), _) => { app.refresh(); }

        // Tab switching
        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
            app.tab = app.tab.next();
            app.analytics_scroll = 0;
        }
        (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
            app.tab = app.tab.prev();
            app.analytics_scroll = 0;
        }

        // Up / down — context-sensitive
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
            match app.tab {
                Tab::Sessions => {
                    if app.session_cursor > 0 {
                        app.session_cursor -= 1;
                        // Scroll up if cursor went above visible area
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
                        // Subtract header row + borders from usable height
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

        // Enter — open detail on Sessions screen
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
            Constraint::Length(1),  // tab bar
            Constraint::Min(1),     // screen content
            Constraint::Length(1),  // footer
        ])
        .split(area);

    draw_tab_bar(f, chunks[0], app);

    match app.tab {
        Tab::Dashboard => draw_dashboard(f, chunks[1], app),
        Tab::Sessions  => draw_sessions_screen(f, chunks[1], app),
        Tab::Analytics => draw_analytics_screen(f, chunks[1], app),
    }

    draw_footer(f, chunks[2], app);

    // Detail overlay drawn on top of everything
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
            Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(label, style));
    }

    // Version flush right — fill gap with spaces
    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let version = format!("  v{}  ", env!("CARGO_PKG_VERSION"));
    let pad = (area.width as usize).saturating_sub(used + version.len());
    spans.push(Span::styled(" ".repeat(pad), Style::default()));
    spans.push(Span::styled(version, Style::default().fg(Color::DarkGray)));

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Reset));
    f.render_widget(p, area);
}

// ── Dashboard screen ──────────────────────────────────────────────────────────

fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let active = app.sessions.iter().find(|s| s.is_active);
    let active_h = if active.is_some() { 7u16 } else { 3u16 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(active_h),
            Constraint::Length(5),
            Constraint::Min(3),
        ])
        .split(area);

    draw_active_panel(f, chunks[0], active);
    draw_spend_panel(f, chunks[1], &app.sessions);
    draw_sessions_list(f, chunks[2], &app.sessions, None, 0);
}

// ── Sessions screen ───────────────────────────────────────────────────────────

fn draw_sessions_screen(f: &mut Frame, area: Rect, app: &App) {
    draw_sessions_list(f, area, &app.sessions, Some(app.session_cursor), app.session_scroll);
}

// ── Analytics screen ──────────────────────────────────────────────────────────

fn draw_analytics_screen(f: &mut Frame, area: Rect, app: &App) {
    let daily    = compute_daily_spend(&app.sessions);
    let projects = compute_project_breakdown(&app.sessions);
    let models   = compute_model_breakdown(&app.sessions);

    // Layout: chart (6) | projects | models
    let proj_h = (projects.len() as u16 + 3).min(area.height / 3);
    let model_h = (models.len() as u16 + 3).min(area.height / 4);
    let chart_h = area.height.saturating_sub(proj_h + model_h).max(6);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(chart_h),
            Constraint::Length(proj_h),
            Constraint::Length(model_h),
        ])
        .split(area);

    // ── Daily chart ──────────────────────────────────────────────────────
    let costs: Vec<f64> = daily.iter().map(|d| d.cost).collect();
    let max_cost = costs.iter().cloned().fold(0.0f64, f64::max);
    let total: f64 = costs.iter().sum();

    let chart_block = Block::default()
        .title(format!(" Daily Spend  (30 days, total {}) ", format::cost(total)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = chart_block.inner(chunks[0]);
    f.render_widget(chart_block, chunks[0]);

    // Build bar chart lines
    if max_cost > 0.0 {
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let bar_line: String = costs.iter().map(|&c| {
            let level = ((c / max_cost) * (bars.len() - 1) as f64).round() as usize;
            bars[level.min(bars.len() - 1)]
        }).collect();

        // Date labels for first and last day
        let first_label = daily.first().map(|d| d.date.format("%b %d").to_string()).unwrap_or_default();
        let last_label  = daily.last().map(|d| d.date.format("%b %d").to_string()).unwrap_or_default();
        let label_pad   = inner.width as usize - first_label.len() - last_label.len();

        let lines = vec![
            Line::from(Span::styled(bar_line, Style::default().fg(Color::Blue))),
            Line::from(vec![
                Span::styled(first_label, Style::default().fg(Color::DarkGray)),
                Span::raw(" ".repeat(label_pad.max(1))),
                Span::styled(last_label, Style::default().fg(Color::DarkGray)),
            ]),
        ];
        let p = Paragraph::new(lines);
        f.render_widget(p, inner);
    } else {
        let p = Paragraph::new("No spend data.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(p, inner);
    }

    // ── By Project ────────────────────────────────────────────────────────
    let proj_block = Block::default()
        .title(" By Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let proj_inner = proj_block.inner(chunks[1]);
    f.render_widget(proj_block, chunks[1]);

    let scroll = app.analytics_scroll;
    let visible_projs: Vec<&crate::models::ProjectSpend> = projects.iter().skip(scroll).collect();
    let proj_rows: Vec<Row> = visible_projs.iter().take(proj_inner.height as usize).map(|p| {
        Row::new(vec![
            Cell::from(p.display_path.clone()),
            Cell::from(p.session_count.to_string()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format::cost(p.total_cost)).style(Style::default().fg(Color::White)),
        ])
    }).collect();

    let proj_table = Table::new(proj_rows, [
        Constraint::Min(30),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(Row::new(["Project", "Sessions", "Cost"])
        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
    .column_spacing(2);
    f.render_widget(proj_table, proj_inner);

    // ── By Model ──────────────────────────────────────────────────────────
    let model_block = Block::default()
        .title(" By Model ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let model_inner = model_block.inner(chunks[2]);
    f.render_widget(model_block, chunks[2]);

    let model_rows: Vec<Row> = models.iter().take(model_inner.height as usize).map(|m| {
        let color = model_color_for(&m.model);
        Row::new(vec![
            Cell::from(m.display_name.clone()).style(Style::default().fg(color)),
            Cell::from(m.session_count.to_string()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format::cost(m.total_cost)).style(Style::default().fg(Color::White)),
        ])
    }).collect();

    let model_table = Table::new(model_rows, [
        Constraint::Min(15),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(Row::new(["Model", "Sessions", "Cost"])
        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
    .column_spacing(2);
    f.render_widget(model_table, model_inner);
}

// ── Session detail overlay ────────────────────────────────────────────────────

fn draw_detail_overlay(f: &mut Frame, area: Rect, s: &ClaudeSession) {
    // 80% width, 80% height centered
    let popup = centered_rect(80, 85, area);

    // Clear area behind popup
    f.render_widget(Clear, popup);

    let title = s.title.as_deref().unwrap_or("Session Detail");
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let model      = format::model_short_name(&s.model);
    let model_color = model_color_for(&s.model);
    let dur        = format::duration(s.duration_secs());
    let cost       = format::cost(s.total_cost);
    let burn       = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
    let ctx_pct    = format!("{:.0}%", s.context_health_fraction() * 100.0);
    let cache_pct  = format!("{:.0}%", s.token_usage.cache_hit_rate() * 100.0);
    let ctx_color  = context_color(s.context_health_fraction());

    let mut lines: Vec<Line> = vec![
        // Header
        Line::from(vec![
            Span::styled(s.display_path(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  ·  "),
            Span::styled(model, Style::default().fg(model_color)),
            if s.is_active {
                Span::styled("  ● Active", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        // Stats
        Line::from(vec![
            stat_span("Cost",      &cost,    Color::White),
            Span::raw("   "),
            stat_span("Duration",  &dur,     Color::White),
            Span::raw("   "),
            stat_span("Burn rate", &burn,    Color::DarkGray),
        ]),
        Line::from(vec![
            stat_span("Context",   &ctx_pct,   ctx_color),
            Span::raw("   "),
            stat_span("Cache hit", &cache_pct, Color::DarkGray),
        ]),
        Line::from(""),
        // Tokens section
        Line::from(Span::styled("Tokens", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        token_line("Input",       s.token_usage.input_tokens,       Color::White),
        token_line("Output",      s.token_usage.output_tokens,      Color::DarkGray),
        token_line("Cache read",  s.token_usage.cache_read_tokens,  Color::Blue),
        token_line("Cache write", s.token_usage.cache_write_tokens, Color::Cyan),
    ];

    if s.token_usage.thinking_tokens > 0 {
        lines.push(token_line("Thinking", s.token_usage.thinking_tokens, Color::Magenta));
    }

    if let Some(score) = s.claudemd_score {
        lines.push(Line::from(""));
        let sc = if score >= 70 { Color::Green } else if score >= 40 { Color::Yellow } else { Color::Red };
        lines.push(Line::from(vec![
            Span::styled("CLAUDE.md  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} / 100", score), Style::default().fg(sc).add_modifier(Modifier::BOLD)),
        ]));
    }

    // Context gauge
    let gauge_y = inner.y + lines.len() as u16 + 1;
    if gauge_y < inner.y + inner.height {
        let gauge_area = Rect { x: inner.x, y: gauge_y, width: inner.width, height: 1 };
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(ctx_color).bg(Color::DarkGray))
            .ratio(s.context_health_fraction());
        f.render_widget(gauge, gauge_area);
    }

    // Footer line (path + copy hint)
    let footer_y = inner.y + inner.height.saturating_sub(1);
    if footer_y > inner.y {
        let footer_area = Rect { x: inner.x, y: footer_y, width: inner.width, height: 1 };
        let path_short = if s.project_path.len() > (inner.width as usize).saturating_sub(20) {
            format!("…{}", &s.project_path[s.project_path.len().saturating_sub(inner.width as usize - 22)..])
        } else {
            s.project_path.clone()
        };
        let footer = Line::from(vec![
            Span::styled(path_short, Style::default().fg(Color::DarkGray)),
            Span::styled("  [c] copy path  [Esc] back", Style::default().fg(Color::DarkGray)),
        ]);
        f.render_widget(Paragraph::new(footer), footer_area);
    }

    let text_area = Rect {
        height: lines.len().min(inner.height as usize) as u16,
        ..inner
    };
    f.render_widget(Paragraph::new(lines), text_area);
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

// ── Shared: active session panel ──────────────────────────────────────────────

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
            let path  = s.display_path();
            let model = format::model_short_name(&s.model);
            let dur   = format::duration(s.duration_secs());
            let cost  = format::cost(s.total_cost);
            let burn  = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
            let frac  = s.context_health_fraction();
            let ctx   = format!("{:.0}%", frac * 100.0);
            let cache = format!("{:.0}% cache", s.token_usage.cache_hit_rate() * 100.0);
            let color = context_color(frac);

            let rows = vec![
                Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Green)),
                    Span::styled(path, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("  ·  "),
                    Span::styled(model, Style::default().fg(model_color_for(&s.model))),
                    Span::raw("  ·  "),
                    Span::styled(dur, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("  Cost "),
                    Span::styled(cost, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("   Burn "),
                    Span::styled(burn, Style::default().fg(Color::DarkGray)),
                    Span::raw("   Context "),
                    Span::styled(ctx, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw("   Cache "),
                    Span::styled(cache, Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let text_area = Rect { height: 2, ..inner };
            let gauge_area = Rect { x: inner.x, y: inner.y + 3, width: inner.width, height: 1 };

            f.render_widget(Paragraph::new(rows), text_area);

            if gauge_area.y < area.y + area.height {
                let gauge = Gauge::default()
                    .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                    .ratio(frac);
                f.render_widget(gauge, gauge_area);
            }
        }
    }
}

// ── Shared: spend summary panel ───────────────────────────────────────────────

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
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
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

// ── Shared: session list ──────────────────────────────────────────────────────

/// `cursor` = Some(idx) for Sessions screen (with highlight); None for Dashboard (no highlight).
fn draw_sessions_list(f: &mut Frame, area: Rect, sessions: &[ClaudeSession], cursor: Option<usize>, scroll: usize) {
    let block = Block::default()
        .title(" Recent Sessions ")
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
        let row_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let dot     = if s.is_active { "●" } else { "○" };
        let dot_s   = Style::default().fg(if s.is_active { Color::Green } else { Color::DarkGray });
        let when    = format::relative_time(&s.start_time);
        let dur     = format::duration(s.duration_secs());
        let model   = format::model_short_name(&s.model);
        let path    = s.display_path();
        let title   = s.title.as_deref().unwrap_or(path.as_str());
        let label   = if title.len() > 38 { format!("{}…", &title[..37]) } else { title.to_string() };
        let cost    = format::cost(s.total_cost);

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

    let hint = if cursor.is_some() { " [Enter] detail" } else { "" };
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
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))
        )
        .column_spacing(1);

    f.render_widget(table, inner);

    // Scroll indicator
    if sessions.len() > max_rows && max_rows > 0 {
        let pct = scroll as f64 / (sessions.len() - max_rows).max(1) as f64;
        let thumb_y = inner.y + (pct * (inner.height as f64 - 1.0)).round() as u16;
        if thumb_y < inner.y + inner.height {
            let indicator = Paragraph::new("▐").style(Style::default().fg(Color::DarkGray));
            let ind_area = Rect { x: inner.x + inner.width - 1, y: thumb_y, width: 1, height: 1 };
            f.render_widget(indicator, ind_area);
        }
    }
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
        let common = vec![
            Span::styled("  [←/→] switch tab  ", Style::default().fg(Color::DarkGray)),
        ];
        let screen_hints: Vec<Span> = match app.tab {
            Tab::Dashboard => vec![
                Span::styled("[r] refresh  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
            ],
            Tab::Sessions => vec![
                Span::styled("[↑/↓] select  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Enter] detail  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[r] refresh  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
            ],
            Tab::Analytics => vec![
                Span::styled("[↑/↓] scroll  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[r] refresh  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
            ],
        };
        [common, screen_hints].concat()
    };

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn model_color_for(model: &str) -> Color {
    let l = model.to_lowercase();
    if l.contains("opus")        { Color::Magenta }
    else if l.contains("haiku")  { Color::Green   }
    else                         { Color::Blue     }
}

fn context_color(fraction: f64) -> Color {
    if fraction < 0.70      { Color::Blue   }
    else if fraction < 0.90 { Color::Yellow }
    else                    { Color::Red    }
}

/// Returns a centered Rect of the given percentage of `area`.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_w = area.width  * percent_x / 100;
    let popup_h = area.height * percent_y / 100;
    Rect {
        x: area.x + (area.width  - popup_w) / 2,
        y: area.y + (area.height - popup_h) / 2,
        width:  popup_w,
        height: popup_h,
    }
}
