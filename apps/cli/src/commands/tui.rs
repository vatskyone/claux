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
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::models::ClaudeSession;
use crate::monitor::{load_sessions, SessionCache};
use crate::spend::compute_spend;
use crate::format;

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let result = event_loop(&mut term);

    // Restore terminal
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}

fn event_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut cache    = SessionCache::new();
    let mut sessions = load_sessions(&mut cache);
    let mut last_refresh = Instant::now();

    loop {
        term.draw(|f| draw(f, &sessions))?;

        // Poll for input with 250ms timeout for responsive key handling
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) |
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('r'), _) => {
                        sessions = load_sessions(&mut cache);
                        last_refresh = Instant::now();
                    }
                    _ => {}
                }
            }
        }

        // Auto-refresh every REFRESH_INTERVAL
        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            sessions = load_sessions(&mut cache);
            last_refresh = Instant::now();
        }
    }
}

fn draw(f: &mut Frame, sessions: &[ClaudeSession]) {
    let area = f.area();
    let active = sessions.iter().find(|s| s.is_active);

    // Outer layout: active card | spend | sessions list | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if active.is_some() { 7 } else { 3 }),
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    draw_active(f, chunks[0], active);
    draw_spend(f, chunks[1], sessions);
    draw_sessions(f, chunks[2], sessions);
    draw_footer(f, chunks[3]);
}

// ── Active session panel ──────────────────────────────────────────────────────

fn draw_active(f: &mut Frame, area: Rect, session: Option<&ClaudeSession>) {
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
            let ctx_pct = format!("{:.0}%", frac * 100.0);
            let cache_pct = format!("{:.0}% cache", s.token_usage.cache_hit_rate() * 100.0);

            let model_color = model_color_for(&s.model);
            let ctx_color   = context_color(frac);

            let rows = vec![
                Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Green)),
                    Span::styled(path.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw("  ·  "),
                    Span::styled(model.clone(), Style::default().fg(model_color)),
                    Span::raw("  ·  "),
                    Span::styled(dur.clone(), Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("  Cost "),
                    Span::styled(cost.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("   Burn "),
                    Span::styled(burn.clone(), Style::default().fg(Color::DarkGray)),
                    Span::raw("   Cache "),
                    Span::styled(cache_pct.clone(), Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("  Context "),
                    Span::styled(ctx_pct.clone(), Style::default().fg(ctx_color).add_modifier(Modifier::BOLD)),
                ]),
            ];

            // Context gauge (row below)
            let gauge_area = Rect {
                x: inner.x,
                y: inner.y + 3,
                width: inner.width,
                height: 1,
            };
            let text_area = Rect { height: 3, ..inner };

            let p = Paragraph::new(rows);
            f.render_widget(p, text_area);

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(ctx_color).bg(Color::DarkGray))
                .ratio(frac);
            if gauge_area.y < inner.y + inner.height {
                f.render_widget(gauge, gauge_area);
            }
        }
    }
}

// ── Spend summary panel ───────────────────────────────────────────────────────

fn draw_spend(f: &mut Frame, area: Rect, sessions: &[ClaudeSession]) {
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
    let trend_str = if prev > 0.0 {
        let diff = current - prev;
        if diff >= 0.0 {
            format!("↑ {}", format::cost(diff))
        } else {
            format!("↓ {}", format::cost(-diff))
        }
    } else {
        String::new()
    };
    let trend_color = if current > prev { Color::Red } else { Color::Green };

    let lines = vec![
        Line::from(Span::styled(label, Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(
            format::cost(current),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(trend_str, Style::default().fg(trend_color))),
    ];
    let p = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(p, area);
}

// ── Session list panel ────────────────────────────────────────────────────────

fn draw_sessions(f: &mut Frame, area: Rect, sessions: &[ClaudeSession]) {
    let block = Block::default()
        .title(" Recent Sessions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if sessions.is_empty() {
        let p = Paragraph::new("No sessions found.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(p, inner);
        return;
    }

    let max_rows = inner.height as usize;
    let visible: Vec<&ClaudeSession> = sessions.iter().take(max_rows).collect();

    let rows: Vec<Row> = visible.iter().map(|s| {
        let dot   = if s.is_active { "●" } else { "○" };
        let dot_s = Style::default().fg(if s.is_active { Color::Green } else { Color::DarkGray });
        let when  = format::relative_time(&s.start_time);
        let model = format::model_short_name(&s.model);
        let path  = s.display_path();
        let title = s.title.as_deref().unwrap_or(path.as_str());
        let label = if title.len() > 35 { format!("{}…", &title[..34]) } else { title.to_string() };
        let cost  = format::cost(s.total_cost);
        let dur   = format::duration(s.duration_secs());
        let model_color = model_color_for(&s.model);

        Row::new(vec![
            Cell::from(dot).style(dot_s),
            Cell::from(when).style(Style::default().fg(Color::DarkGray)),
            Cell::from(dur).style(Style::default().fg(Color::DarkGray)),
            Cell::from(model).style(Style::default().fg(model_color)),
            Cell::from(label),
            Cell::from(cost).style(Style::default().fg(Color::White)),
        ])
    }).collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Min(20),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(["", "When", "Dur", "Model", "Session", "Cost"])
                .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))
        )
        .column_spacing(1);

    f.render_widget(table, inner);
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn draw_footer(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" [q] quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[r] refresh  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Ctrl+C] exit", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("  CLAUX v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let p = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(p, area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn model_color_for(model: &str) -> Color {
    let l = model.to_lowercase();
    if l.contains("opus")   { Color::Magenta }
    else if l.contains("haiku") { Color::Green   }
    else                    { Color::Blue     }
}

fn context_color(fraction: f64) -> Color {
    if fraction < 0.70 { Color::Blue }
    else if fraction < 0.90 { Color::Yellow }
    else { Color::Red }
}
