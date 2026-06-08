use anyhow::Result;
use chrono::{Datelike, Duration as ChronoDuration, Local, Timelike};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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

use crate::account::load_account_info;
use crate::checkpoints::{
    delete_checkpoint, infer_project_path, load_checkpoints, save_checkpoint, write_context_md,
};
use crate::config::load_claux_config;
use crate::format;
use crate::metrics::{record_empty_state, record_refresh_latency};
use crate::models::{
    agent_level, AccountInfo, AgentRun, Checkpoint, ClaudeSession, ClaudemdAnalysis, ClauxConfig,
    SkillInfo,
};
use crate::monitor::{
    compute_agent_type_counts, load_agents_for_session, load_sessions, SessionCache,
};
use crate::parser::{find_claudemd_path, score_claudemd_detailed};
use crate::skills::load_skills;
use crate::spend::{
    compute_daily_spend, compute_model_breakdown, compute_monthly_forecast,
    compute_project_breakdown, compute_spend,
};
use crate::tags;
use crate::usage::{five_hour_state, weekly_state, ProgressReason};

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

// ── Screens ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Dashboard = 0,
    Sessions = 1,
    Analytics = 2,
    Agents = 3,
    Skills = 4,
    History = 5,
}

impl Tab {
    fn next(self) -> Tab {
        match self {
            Tab::Dashboard => Tab::Sessions,
            Tab::Sessions => Tab::Analytics,
            Tab::Analytics => Tab::Agents,
            Tab::Agents => Tab::Skills,
            Tab::Skills => Tab::History,
            Tab::History => Tab::Dashboard,
        }
    }
    fn prev(self) -> Tab {
        match self {
            Tab::Dashboard => Tab::History,
            Tab::Sessions => Tab::Dashboard,
            Tab::Analytics => Tab::Sessions,
            Tab::Agents => Tab::Analytics,
            Tab::Skills => Tab::Agents,
            Tab::History => Tab::Skills,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Sessions => "Sessions",
            Tab::Analytics => "Analytics",
            Tab::Agents => "Agents",
            Tab::Skills => "Skills",
            Tab::History => "History",
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    tab: Tab,
    // Sessions tab
    session_cursor: usize,
    session_scroll: usize,
    // Analytics tab
    analytics_scroll: usize,
    // Session detail overlay
    detail_open: Option<usize>,
    detail_analysis: Option<ClaudemdAnalysis>,
    // Agents tab
    agents: Vec<AgentRun>,
    agent_cursor: usize,
    agent_scroll: usize,
    agent_type_counts: HashMap<String, usize>,
    agent_counts_dirty: bool,
    // Skills tab
    skills: Vec<SkillInfo>,
    skill_cursor: usize,
    skill_scroll: usize,
    skills_dirty: bool,
    // History tab
    checkpoints: Vec<Checkpoint>,
    checkpoint_cursor: usize,
    checkpoint_scroll: usize,
    checkpoints_dirty: bool,
    cp_name_editing: bool,
    cp_name_buf: String,
    // Tag editing (inside session detail overlay)
    tag_editing: bool,
    tag_input_buf: String,
    // Account / config (loaded once)
    account_info: Option<AccountInfo>,
    claux_config: ClauxConfig,
    // Shared data
    sessions: Vec<ClaudeSession>,
    cache: SessionCache,
    last_refresh: Instant,
}

impl App {
    fn new() -> Self {
        let mut cache = SessionCache::new();
        let sessions = load_sessions(&mut cache);
        Self {
            tab: Tab::Dashboard,
            session_cursor: 0,
            session_scroll: 0,
            analytics_scroll: 0,
            detail_open: None,
            detail_analysis: None,
            agents: vec![],
            agent_cursor: 0,
            agent_scroll: 0,
            agent_type_counts: HashMap::new(),
            agent_counts_dirty: true,
            skills: vec![],
            skill_cursor: 0,
            skill_scroll: 0,
            skills_dirty: true,
            checkpoints: vec![],
            checkpoint_cursor: 0,
            checkpoint_scroll: 0,
            checkpoints_dirty: true,
            cp_name_editing: false,
            cp_name_buf: String::new(),
            tag_editing: false,
            tag_input_buf: String::new(),
            account_info: load_account_info(),
            claux_config: load_claux_config(),
            sessions,
            cache,
            last_refresh: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        let started = Instant::now();
        self.sessions = load_sessions(&mut self.cache);
        if self.sessions.is_empty() {
            record_empty_state("source_unavailable");
        } else if !self.sessions.iter().any(|s| s.is_active) {
            record_empty_state("no_active_session");
        }
        if self.session_cursor >= self.sessions.len() && !self.sessions.is_empty() {
            self.session_cursor = self.sessions.len() - 1;
        }
        if self.tab == Tab::Agents {
            self.reload_agents();
        }
        if self.tab == Tab::Skills {
            self.reload_skills();
        }
        if self.tab == Tab::History {
            self.reload_checkpoints();
        }
        self.last_refresh = Instant::now();
        record_refresh_latency(started.elapsed());
    }

    fn reload_skills(&mut self) {
        self.skills = load_skills();
        if self.skill_cursor >= self.skills.len() && !self.skills.is_empty() {
            self.skill_cursor = self.skills.len() - 1;
        }
        self.skills_dirty = false;
    }

    fn reload_checkpoints(&mut self) {
        let path = infer_project_path(&self.sessions);
        self.checkpoints = load_checkpoints(&path);
        if self.checkpoint_cursor >= self.checkpoints.len() && !self.checkpoints.is_empty() {
            self.checkpoint_cursor = self.checkpoints.len() - 1;
        }
        self.checkpoints_dirty = false;
    }

    fn reload_agents(&mut self) {
        if let Some(active) = self.sessions.iter().find(|s| s.is_active) {
            self.agents = load_agents_for_session(active);
        } else {
            self.agents.clear();
        }
        if self.agent_cursor >= self.agents.len() && !self.agents.is_empty() {
            self.agent_cursor = self.agents.len() - 1;
        }
        if self.agent_counts_dirty {
            self.agent_type_counts = compute_agent_type_counts(&self.sessions);
            self.agent_counts_dirty = false;
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
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
    // Checkpoint name input mode — highest priority on History tab
    if app.cp_name_editing {
        match code {
            KeyCode::Enter => {
                let name = app.cp_name_buf.trim().to_string();
                if !name.is_empty() {
                    let path = infer_project_path(&app.sessions);
                    let _ = save_checkpoint(&path, &app.sessions, &name);
                    app.reload_checkpoints();
                }
                app.cp_name_editing = false;
                app.cp_name_buf = String::new();
            }
            KeyCode::Esc => {
                app.cp_name_editing = false;
                app.cp_name_buf = String::new();
            }
            KeyCode::Backspace => {
                app.cp_name_buf.pop();
            }
            KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => {
                if app.cp_name_buf.len() < 50 {
                    app.cp_name_buf.push(c);
                }
            }
            _ => {}
        }
        return false;
    }

    // Tag input mode gets highest priority
    if app.tag_editing {
        match code {
            KeyCode::Enter => {
                if let Some(idx) = app.detail_open {
                    if let Some(s) = app.sessions.get(idx) {
                        let _ = tags::save_tag(&s.id, &app.tag_input_buf);
                        // Reload sessions to pick up the new tag
                        app.sessions = load_sessions(&mut app.cache);
                    }
                }
                app.tag_editing = false;
            }
            KeyCode::Esc => {
                app.tag_editing = false;
                app.tag_input_buf = String::new();
            }
            KeyCode::Backspace => {
                app.tag_input_buf.pop();
            }
            KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => {
                if app.tag_input_buf.len() < 30 {
                    app.tag_input_buf.push(c);
                }
            }
            _ => {}
        }
        return false;
    }

    // Detail overlay gets second priority
    if app.detail_open.is_some() {
        match code {
            KeyCode::Esc | KeyCode::Backspace => {
                app.detail_open = None;
            }
            KeyCode::Char('q') => return true,
            KeyCode::Char('t') => {
                if let Some(idx) = app.detail_open {
                    if let Some(s) = app.sessions.get(idx) {
                        app.tag_input_buf = s.tag.clone().unwrap_or_default();
                        app.tag_editing = true;
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(idx) = app.detail_open {
                    if let Some(s) = app.sessions.get(idx) {
                        let _ = Command::new("pbcopy")
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                use std::io::Write;
                                if let Some(stdin) = child.stdin.as_mut() {
                                    stdin.write_all(s.project_path.as_bytes())?;
                                }
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
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return true,

        (KeyCode::Char('r'), _) => {
            app.refresh();
        }

        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
            app.tab = app.tab.next();
            app.analytics_scroll = 0;
            if app.tab == Tab::Agents {
                app.agent_scroll = 0;
                app.reload_agents();
            }
            if app.tab == Tab::Skills && app.skills_dirty {
                app.skill_scroll = 0;
                app.reload_skills();
            }
            if app.tab == Tab::History && app.checkpoints_dirty {
                app.checkpoint_scroll = 0;
                app.reload_checkpoints();
            }
        }
        (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
            app.tab = app.tab.prev();
            app.analytics_scroll = 0;
            if app.tab == Tab::Agents {
                app.agent_scroll = 0;
                app.reload_agents();
            }
            if app.tab == Tab::Skills && app.skills_dirty {
                app.skill_scroll = 0;
                app.reload_skills();
            }
            if app.tab == Tab::History && app.checkpoints_dirty {
                app.checkpoint_scroll = 0;
                app.reload_checkpoints();
            }
        }

        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => match app.tab {
            Tab::Sessions => {
                if app.session_cursor > 0 {
                    app.session_cursor -= 1;
                    if app.session_cursor < app.session_scroll {
                        app.session_scroll = app.session_cursor;
                    }
                }
            }
            Tab::Analytics => {
                if app.analytics_scroll > 0 {
                    app.analytics_scroll -= 1;
                }
            }
            Tab::Agents => {
                if app.agent_cursor > 0 {
                    app.agent_cursor -= 1;
                    if app.agent_cursor < app.agent_scroll {
                        app.agent_scroll = app.agent_cursor;
                    }
                }
            }
            Tab::Skills => {
                if app.skill_cursor > 0 {
                    app.skill_cursor -= 1;
                    if app.skill_cursor < app.skill_scroll {
                        app.skill_scroll = app.skill_cursor;
                    }
                }
            }
            Tab::History => {
                if app.checkpoint_cursor > 0 {
                    app.checkpoint_cursor -= 1;
                    if app.checkpoint_cursor < app.checkpoint_scroll {
                        app.checkpoint_scroll = app.checkpoint_cursor;
                    }
                }
            }
            Tab::Dashboard => {}
        },
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => match app.tab {
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
            Tab::Analytics => {
                app.analytics_scroll += 1;
            }
            Tab::Agents => {
                let max = app.agents.len().saturating_sub(1);
                if app.agent_cursor < max {
                    app.agent_cursor += 1;
                    // ~38% of viewport minus borders/header
                    let visible = (viewport_h * 38 / 100).saturating_sub(3).max(1);
                    if app.agent_cursor >= app.agent_scroll + visible {
                        app.agent_scroll = app.agent_cursor + 1 - visible;
                    }
                }
            }
            Tab::Skills => {
                let max = app.skills.len().saturating_sub(1);
                if app.skill_cursor < max {
                    app.skill_cursor += 1;
                    // ~40% of viewport minus borders/header
                    let visible = (viewport_h * 40 / 100).saturating_sub(3).max(1);
                    if app.skill_cursor >= app.skill_scroll + visible {
                        app.skill_scroll = app.skill_cursor + 1 - visible;
                    }
                }
            }
            Tab::History => {
                let max = app.checkpoints.len().saturating_sub(1);
                if app.checkpoint_cursor < max {
                    app.checkpoint_cursor += 1;
                    // ~40% of viewport minus borders/header
                    let visible = (viewport_h * 40 / 100).saturating_sub(3).max(1);
                    if app.checkpoint_cursor >= app.checkpoint_scroll + visible {
                        app.checkpoint_scroll = app.checkpoint_cursor + 1 - visible;
                    }
                }
            }
            Tab::Dashboard => {}
        },

        (KeyCode::Enter, _) => {
            if app.tab == Tab::Sessions && !app.sessions.is_empty() {
                let idx = app.session_cursor;
                app.detail_open = Some(idx);
                // Pre-compute CLAUDE.md detailed analysis for this session
                app.detail_analysis = app
                    .sessions
                    .get(idx)
                    .and_then(|s| find_claudemd_path(&s.project_path))
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|c| score_claudemd_detailed(&c));
            }
        }

        // History tab actions
        (KeyCode::Char('s'), _) if app.tab == Tab::History => {
            app.cp_name_editing = true;
            app.cp_name_buf = String::new();
        }
        (KeyCode::Char('d'), _) if app.tab == Tab::History => {
            if let Some(cp) = app.checkpoints.get(app.checkpoint_cursor) {
                let id = cp.id.clone();
                let path = infer_project_path(&app.sessions);
                let _ = delete_checkpoint(&path, &id);
                app.reload_checkpoints();
            }
        }
        (KeyCode::Char('w'), _) if app.tab == Tab::History => {
            if let Some(cp) = app.checkpoints.get(app.checkpoint_cursor) {
                let path = infer_project_path(&app.sessions);
                let _ = write_context_md(&path, cp);
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
        Tab::Sessions => draw_sessions_screen(f, chunks[1], app),
        Tab::Analytics => draw_analytics_screen(f, chunks[1], app),
        Tab::Agents => draw_agents_screen(f, chunks[1], app),
        Tab::Skills => draw_skills_screen(f, chunks[1], app),
        Tab::History => draw_history_screen(f, chunks[1], app),
    }

    draw_footer(f, chunks[2], app);

    if let Some(idx) = app.detail_open {
        if let Some(session) = app.sessions.get(idx) {
            draw_detail_overlay(
                f,
                area,
                session,
                app.tag_editing,
                &app.tag_input_buf,
                app.detail_analysis.as_ref(),
            );
        }
    }
}

// ── Tab bar ───────────────────────────────────────────────────────────────────

fn draw_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let has_active = app.sessions.iter().any(|s| s.is_active);
    let has_running_agent = app.agents.iter().any(|a| !a.completed);

    let mut spans = vec![Span::styled(
        "  CLAUX  ",
        Style::default().fg(Color::DarkGray),
    )];

    for tab in [
        Tab::Dashboard,
        Tab::Sessions,
        Tab::Analytics,
        Tab::Agents,
        Tab::Skills,
        Tab::History,
    ] {
        let is_selected = app.tab == tab;
        let label = match tab {
            Tab::Dashboard if has_active => format!("  ● {}  ", tab.label()),
            Tab::Agents if has_running_agent => format!("  ● {}  ", tab.label()),
            _ => format!("  {}  ", tab.label()),
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

    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let version = format!("  v{}  ", env!("CARGO_PKG_VERSION"));
    let pad = (area.width as usize).saturating_sub(used + version.len());
    spans.push(Span::styled(" ".repeat(pad), Style::default()));
    spans.push(Span::styled(version, Style::default().fg(Color::DarkGray)));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Reset)),
        area,
    );
}

// ── Dashboard screen ──────────────────────────────────────────────────────────

fn draw_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let active = app.sessions.iter().find(|s| s.is_active);
    let active_h = if active.is_some() { 8u16 } else { 4u16 };
    let spend_h = 5u16;
    let mid_h = area.height.saturating_sub(active_h + spend_h).max(6);

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

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(mid[0]);

    draw_token_breakdown(f, left_chunks[0], active);
    draw_usage_panel(
        f,
        left_chunks[1],
        active,
        &app.sessions,
        &app.claux_config,
        app.account_info.as_ref(),
    );
    draw_insights_panel(f, mid[1], active, &app.sessions);

    draw_spend_panel(f, chunks[2], &app.sessions);
}

// ── Sessions screen ───────────────────────────────────────────────────────────

fn draw_sessions_screen(f: &mut Frame, area: Rect, app: &App) {
    draw_sessions_list(
        f,
        area,
        &app.sessions,
        Some(app.session_cursor),
        app.session_scroll,
        true,
    );
}

// ── Analytics screen ──────────────────────────────────────────────────────────

fn draw_analytics_screen(f: &mut Frame, area: Rect, app: &App) {
    let daily = compute_daily_spend(&app.sessions);
    let projects = compute_project_breakdown(&app.sessions);
    let models = compute_model_breakdown(&app.sessions);
    let forecast = compute_monthly_forecast(&app.sessions);

    let mut model_output: HashMap<String, u64> = HashMap::new();
    for s in &app.sessions {
        *model_output.entry(s.model.clone()).or_insert(0) += s.token_usage.output_tokens;
    }

    let chart7_h = area.height * 45 / 100;
    let sparkline_h = 3u16;
    let forecast_h = 4u16;
    let tables_h = area
        .height
        .saturating_sub(chart7_h + sparkline_h + forecast_h);
    let proj_h = tables_h * 55 / 100;
    let model_h = tables_h.saturating_sub(proj_h);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(chart7_h.max(8)),
            Constraint::Length(sparkline_h),
            Constraint::Length(forecast_h),
            Constraint::Length(proj_h.max(4)),
            Constraint::Length(model_h.max(4)),
        ])
        .split(area);

    draw_7day_chart(f, chunks[0], &daily);
    draw_30day_sparkline(f, chunks[1], &daily);
    draw_forecast_panel(f, chunks[2], &forecast);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[3]);

    draw_project_table(f, bottom_cols[0], &projects, app.analytics_scroll);
    draw_model_table(f, bottom_cols[1], &models, &model_output);
}

// ── Agents screen ─────────────────────────────────────────────────────────────
//
//  ┌─ Agents ── 3 spawned · 2 done · 1 running · $0.23 ──────────────────────┐
//  │  ●  Type          Lv  XP bar           Task               ★    Cost  Dur │
//  │  ●  Explore       3   Lv3 [████████░░] Explore desktop…  ★★★★★ $0.05 1m │
//  │     Plan          2   Lv2 [██████░░░░] Design TUI nav…   ★★★★☆ $0.04 45s│
//  └──────────────────────────────────────────────────────────────────────────┘
//  ┌─ Explore — detail ───────────────────────────────────────────────────────┐
//  │  Task:   Explore the directory /Users/snow/Desktop/...                   │
//  │  Status: ✓ Completed  ·  1m 23s  ·  Sonnet 4.6  ·  $0.05               │
//  │                                                                           │
//  │  Tokens (% of session):                                                  │
//  │  Input     12.3K  [████████████░░░░░░░░]  34%                            │
//  │  Output     4.2K  [█████░░░░░░░░░░░░░░░]  12%                            │
//  │  Cache R    5.1K  [██████░░░░░░░░░░░░░░]  14%                            │
//  │                                                                           │
//  │  Output preview:                                                          │
//  │  "Found 47 Swift files across 3 directories..."                          │
//  │                                                                           │
//  │  Quality: ★★★★★  Rich output, task completed cleanly                    │
//  └──────────────────────────────────────────────────────────────────────────┘

fn draw_agents_screen(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    draw_agent_list(f, chunks[0], app);
    draw_agent_detail(f, chunks[1], app);
}

fn draw_agent_list(f: &mut Frame, area: Rect, app: &App) {
    let agents = &app.agents;
    let spawned = agents.len();
    let done = agents.iter().filter(|a| a.completed).count();
    let running = spawned - done;
    let total_cost: f64 = agents.iter().map(|a| a.total_cost).sum();

    let title = if spawned > 0 {
        format!(
            " Agents ── {} spawned · {} done · {} running · {} ",
            spawned,
            done,
            running,
            format::cost(total_cost)
        )
    } else {
        " Agents ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if agents.is_empty() {
        let active_exists = app.sessions.iter().any(|s| s.is_active);
        let msg = if active_exists {
            "  No agents spawned in active session yet"
        } else {
            "  ○  No active session — agents appear here when Claude is running"
        };
        f.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    // Column widths: dot(2) type(14) lv+xp(14) task(Min) stars(6) cost(7) dur(6)
    let widths = [
        Constraint::Length(2),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Min(16),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(6),
    ];

    let max_rows = inner.height as usize;
    let scroll = app.agent_scroll;
    let rows: Vec<Row> = agents
        .iter()
        .enumerate()
        .skip(scroll)
        .take(max_rows)
        .map(|(idx, agent)| {
            let is_selected = idx == app.agent_cursor;
            let row_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            // Status dot
            let (dot, dot_color) = if !agent.completed {
                ("●", Color::Green)
            } else if agent.quality_score <= 2 {
                ("✗", Color::Red)
            } else {
                ("✓", Color::DarkGray)
            };

            // Agent type (truncated)
            let type_str = if agent.subagent_type.len() > 13 {
                format!(
                    "{}…",
                    agent.subagent_type.chars().take(12).collect::<String>()
                )
            } else {
                agent.subagent_type.clone()
            };

            // XP / level
            let global_count = app
                .agent_type_counts
                .get(&agent.subagent_type)
                .copied()
                .unwrap_or(0);
            let (lv, prog) = agent_level(global_count);
            let xp_str = format!("Lv{} {}", lv, xp_bar(prog, 6));

            // Task description (truncated)
            let task_max = 20usize;
            let task_str = if agent.description.len() > task_max {
                format!(
                    "{}…",
                    agent
                        .description
                        .chars()
                        .take(task_max - 1)
                        .collect::<String>()
                )
            } else {
                agent.description.clone()
            };

            // Duration
            let dur_str = agent_duration_str(agent);

            Row::new(vec![
                Cell::from(dot).style(Style::default().fg(dot_color)),
                Cell::from(type_str),
                Cell::from(xp_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from(task_str),
                Cell::from(stars(agent.quality_score)).style(quality_style(agent.quality_score)),
                Cell::from(format::cost(agent.total_cost)).style(Style::default().fg(Color::White)),
                Cell::from(dur_str).style(Style::default().fg(Color::DarkGray)),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("Type"),
                Cell::from("Lv / XP"),
                Cell::from("Task"),
                Cell::from("★"),
                Cell::from("Cost"),
                Cell::from("Dur"),
            ])
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1);

    f.render_widget(table, inner);
}

fn draw_agent_detail(f: &mut Frame, area: Rect, app: &App) {
    let selected = app.agents.get(app.agent_cursor);

    let title = selected
        .map(|a| {
            let t = if a.subagent_type.len() > 20 {
                format!("{}…", a.subagent_type.chars().take(19).collect::<String>())
            } else {
                a.subagent_type.clone()
            };
            format!(" {} — detail ", t)
        })
        .unwrap_or_else(|| " Detail ".to_string());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(agent) = selected else {
        f.render_widget(
            Paragraph::new("  Select an agent above with ↑/↓")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    // Find active session for token % share computation
    let session_usage = app
        .sessions
        .iter()
        .find(|s| s.is_active)
        .map(|s| &s.token_usage);

    let dur_str = agent_duration_str(agent);
    let model_str = agent
        .model
        .as_deref()
        .map(format::model_short_name)
        .unwrap_or_else(|| "—".to_string());
    let model_col = agent
        .model
        .as_deref()
        .map(model_color_for)
        .unwrap_or(Color::DarkGray);
    let cost_str = format::cost(agent.total_cost);

    let has_tokens = agent.token_usage.input_tokens > 0
        || agent.token_usage.output_tokens > 0
        || agent.token_usage.cache_read_tokens > 0;

    // bar_w: label(9) + count(7) + brackets+bar(bar_w+2) + pct(5) + spaces ≈ inner.width - 26
    let bar_w = (inner.width as usize).saturating_sub(28).max(4);

    let mut lines: Vec<Line> = vec![];

    // ── Task & prompt ────────────────────────────────────────────────────────
    let desc = &agent.description;
    let desc_max = inner.width as usize - 10;
    let desc_str = if desc.len() > desc_max {
        format!("{}…", desc.chars().take(desc_max - 1).collect::<String>())
    } else {
        desc.clone()
    };
    lines.push(Line::from(vec![
        Span::styled("  Task    ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            desc_str,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    if !agent.prompt.is_empty() {
        let prompt_max = inner.width as usize - 12;
        let prompt_preview: String = agent
            .prompt
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(prompt_max)
            .collect();
        let prompt_str = if agent.prompt.len() > prompt_max {
            format!("{}…", prompt_preview)
        } else {
            prompt_preview
        };
        lines.push(Line::from(vec![
            Span::styled("  Prompt  ", Style::default().fg(Color::DarkGray)),
            Span::styled(prompt_str, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // ── Status ───────────────────────────────────────────────────────────────
    if agent.completed {
        lines.push(Line::from(vec![
            Span::styled("  Status  ", Style::default().fg(Color::DarkGray)),
            Span::styled("✓ Completed", Style::default().fg(Color::Green)),
            Span::raw("  ·  "),
            Span::styled(dur_str, Style::default().fg(Color::White)),
            Span::raw("  ·  "),
            Span::styled(model_str, Style::default().fg(model_col)),
            Span::raw("  ·  "),
            Span::styled(cost_str, Style::default().fg(Color::White)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  Status  ", Style::default().fg(Color::DarkGray)),
            Span::styled("● Running…", Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled(dur_str, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));

    // ── Token breakdown ───────────────────────────────────────────────────────
    if has_tokens {
        lines.push(Line::from(Span::styled(
            "  Tokens (% of session):",
            Style::default().fg(Color::DarkGray),
        )));

        let entries: &[(&str, u64, Color)] = &[
            ("Input  ", agent.token_usage.input_tokens, Color::White),
            ("Output ", agent.token_usage.output_tokens, Color::Cyan),
            ("Cache R", agent.token_usage.cache_read_tokens, Color::Blue),
        ];

        for (label, count, color) in entries {
            let share = session_usage
                .map(|su| {
                    let denom = match *label {
                        "Input  " => su.input_tokens,
                        "Output " => su.output_tokens,
                        _ => su.cache_read_tokens,
                    };
                    if denom == 0 {
                        0.0
                    } else {
                        (*count as f64 / denom as f64).min(1.0)
                    }
                })
                .unwrap_or(0.0);

            let filled = (share * bar_w as f64).round() as usize;
            let empty = bar_w.saturating_sub(filled);

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<8}", label),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:>6} ", format::tokens(*count)),
                    Style::default().fg(*color),
                ),
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled("█".repeat(filled), Style::default().fg(*color)),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::styled("]", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("  {:>3.0}%", share * 100.0),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  (token data unavailable — sub-agent file not found)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));

    // ── Output preview ────────────────────────────────────────────────────────
    if !agent.output_preview.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Output preview:",
            Style::default().fg(Color::DarkGray),
        )));

        let line_w = (inner.width as usize).saturating_sub(4).max(10);
        let preview_lines = wrap_text(&agent.output_preview, line_w, 3);
        for pl in &preview_lines {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(pl.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // ── Quality ───────────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("  Quality ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            stars(agent.quality_score),
            quality_style(agent.quality_score),
        ),
        Span::raw("  "),
        Span::styled(
            quality_label(agent.quality_score),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let render_h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(
        Paragraph::new(lines),
        Rect {
            height: render_h,
            ..inner
        },
    );
}

// ── 7-day vertical bar chart ─────────────────────────────────────────────────

fn draw_7day_chart(f: &mut Frame, area: Rect, daily: &[crate::models::DailySpend]) {
    let today = Local::now().date_naive();
    let days: Vec<(chrono::NaiveDate, f64)> = (0i64..7)
        .map(|i| today - ChronoDuration::days(6 - i))
        .map(|date| {
            let cost = daily
                .iter()
                .find(|d| d.date == date)
                .map(|d| d.cost)
                .unwrap_or(0.0);
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

    let costs: Vec<f64> = days.iter().map(|(_, c)| *c).collect();
    let total: f64 = costs.iter().sum();
    let max_cost: f64 = costs.iter().cloned().fold(0.0f64, f64::max);
    let avg = total / 7.0;

    let peak_str = days
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(d, c)| format!("{}  ({})", format::cost(*c), d.format("%a %b %d")))
        .unwrap_or_default();

    let summary = format!(
        "  Total {}    Avg {}/day    Peak {}",
        format::cost(total),
        format::cost(avg),
        peak_str,
    );

    let bar_rows = (inner.height as usize).saturating_sub(5).max(1);
    let n = days.len();
    let col_w = (inner.width as usize) / n;
    if col_w == 0 {
        return;
    }

    let bar_w = col_w.saturating_sub(2).min(5).max(1);
    let pad_l = (col_w.saturating_sub(bar_w)) / 2;
    let pad_r = col_w.saturating_sub(pad_l + bar_w);

    let fill: Vec<usize> = costs
        .iter()
        .map(|&c| {
            if max_cost > 0.0 {
                ((c / max_cost) * bar_rows as f64).round() as usize
            } else {
                0
            }
        })
        .collect();

    let mut lines: Vec<Line> = vec![];
    lines.push(Line::from(Span::styled(
        summary,
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for r in 0..bar_rows {
        let threshold = bar_rows - r;
        let mut spans: Vec<Span> = vec![];
        for (i, &fill_h) in fill.iter().enumerate() {
            let is_today = days[i].0 == today;
            if fill_h >= threshold {
                let bar_color = if is_today { Color::Blue } else { Color::Cyan };
                if pad_l > 0 {
                    spans.push(Span::raw(" ".repeat(pad_l)));
                }
                spans.push(Span::styled(
                    "█".repeat(bar_w),
                    Style::default().fg(bar_color),
                ));
                if pad_r > 0 {
                    spans.push(Span::raw(" ".repeat(pad_r)));
                }
            } else {
                spans.push(Span::raw(" ".repeat(col_w)));
            }
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(Span::styled(
        "─".repeat(col_w * n),
        Style::default().fg(Color::DarkGray),
    )));

    let day_spans: Vec<Span> = days
        .iter()
        .map(|(d, _)| {
            let is_today = *d == today;
            let lbl = format!("{:^width$}", d.format("%a"), width = col_w);
            Span::styled(
                lbl,
                Style::default().fg(if is_today {
                    Color::White
                } else {
                    Color::DarkGray
                }),
            )
        })
        .collect();
    lines.push(Line::from(day_spans));

    let cost_spans: Vec<Span> = days
        .iter()
        .map(|(d, c)| {
            let is_today = *d == today;
            let lbl = format!("{:^width$}", format::cost(*c), width = col_w);
            Span::styled(
                lbl,
                Style::default().fg(if is_today {
                    Color::White
                } else {
                    Color::DarkGray
                }),
            )
        })
        .collect();
    lines.push(Line::from(cost_spans));

    f.render_widget(
        Paragraph::new(lines),
        Rect {
            height: (inner.height),
            ..inner
        },
    );
}

// ── 30-day compact sparkline ─────────────────────────────────────────────────

fn draw_30day_sparkline(f: &mut Frame, area: Rect, daily: &[crate::models::DailySpend]) {
    let costs: Vec<f64> = daily.iter().map(|d| d.cost).collect();
    let max_cost = costs.iter().cloned().fold(0.0f64, f64::max);
    let total: f64 = costs.iter().sum();

    let block = Block::default()
        .title(format!(" 30-Day Trend  ({} total) ", format::cost(total)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if max_cost > 0.0 && inner.height >= 2 {
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let bar_line: String = costs
            .iter()
            .map(|&c| {
                let level = ((c / max_cost) * (bars.len() - 1) as f64).round() as usize;
                bars[level.min(bars.len() - 1)]
            })
            .collect();

        let first = daily
            .first()
            .map(|d| d.date.format("%b %d").to_string())
            .unwrap_or_default();
        let last = daily
            .last()
            .map(|d| d.date.format("%b %d").to_string())
            .unwrap_or_default();
        let pad = (inner.width as usize).saturating_sub(first.len() + last.len() + 2);

        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(bar_line, Style::default().fg(Color::DarkGray))),
                Line::from(vec![
                    Span::styled(first, Style::default().fg(Color::DarkGray)),
                    Span::raw(" ".repeat(pad.max(1))),
                    Span::styled(last, Style::default().fg(Color::DarkGray)),
                ]),
            ]),
            inner,
        );
    } else {
        f.render_widget(
            Paragraph::new("No spend data.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            inner,
        );
    }
}

// ── Monthly forecast panel ────────────────────────────────────────────────────

fn draw_forecast_panel(f: &mut Frame, area: Rect, fc: &crate::models::MonthlyForecast) {
    let block = Block::default()
        .title(" Forecast ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let col_w = (inner.width / 4) as usize;

    let items: &[(&str, String, Color)] = &[
        (
            "Daily avg (7d)",
            format::cost(fc.avg_per_day_7d),
            Color::DarkGray,
        ),
        (
            "Month to date",
            format::cost(fc.month_to_date),
            Color::White,
        ),
        (
            "Est. end of mo",
            format::cost(fc.projected_eom),
            Color::Yellow,
        ),
        (
            "Annual proj.",
            format::cost(fc.projected_annual),
            Color::DarkGray,
        ),
    ];

    let label_line: Vec<Span> = items
        .iter()
        .map(|(lbl, _, _)| {
            Span::styled(
                format!("{:<width$}", lbl, width = col_w),
                Style::default().fg(Color::DarkGray),
            )
        })
        .collect();
    let value_line: Vec<Span> = items
        .iter()
        .map(|(_, val, color)| {
            Span::styled(
                format!("{:<width$}", val, width = col_w),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )
        })
        .collect();

    f.render_widget(
        Paragraph::new(vec![Line::from(label_line), Line::from(value_line)]),
        Rect {
            height: inner.height.min(2),
            ..inner
        },
    );
}

// ── Project table ─────────────────────────────────────────────────────────────

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

    let visible: Vec<&crate::models::ProjectSpend> = projects
        .iter()
        .skip(scroll)
        .take(inner.height as usize)
        .collect();

    let rows: Vec<Row> = visible
        .iter()
        .map(|p| {
            let max_path = (inner.width as usize).saturating_sub(22).max(10);
            let path = if p.display_path.len() > max_path {
                format!(
                    "…{}",
                    &p.display_path[p.display_path.len().saturating_sub(max_path - 1)..]
                )
            } else {
                p.display_path.clone()
            };
            Row::new(vec![
                Cell::from(path),
                Cell::from(p.session_count.to_string()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format::cost(p.total_cost)).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Min(20),
                Constraint::Length(5),
                Constraint::Length(8),
            ],
        )
        .header(
            Row::new(["Project", "Sess", "Cost"]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1),
        inner,
    );
}

// ── Model table with efficiency ───────────────────────────────────────────────

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

    let rows: Vec<Row> = models
        .iter()
        .take(inner.height as usize)
        .map(|m| {
            let color = model_color_for(&m.model);
            let output_tok = model_output.get(&m.model).copied().unwrap_or(0);
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
        })
        .collect();

    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Min(12),
                Constraint::Length(5),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
        )
        .header(
            Row::new(["Model", "Sess", "Cost", "Effic."]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1),
        inner,
    );
}

// ── Token breakdown panel (Dashboard) ────────────────────────────────────────

fn draw_token_breakdown(f: &mut Frame, area: Rect, session: Option<&ClaudeSession>) {
    let block = Block::default()
        .title(" Tokens ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(s) = session else {
        f.render_widget(
            Paragraph::new("  No active session").style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let tu = &s.token_usage;
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
        ("Input   ", tu.input_tokens, Color::White),
        ("Output  ", tu.output_tokens, Color::Cyan),
        ("Cache R ", tu.cache_read_tokens, Color::Blue),
        ("Cache W ", tu.cache_write_tokens, Color::DarkGray),
        ("Thinking", tu.thinking_tokens, Color::Magenta),
    ];

    let mut lines: Vec<Line> = entries
        .iter()
        .map(|(label, count, color)| {
            let filled = ((*count as f64 / max_tok as f64) * bar_w as f64).round() as usize;
            let empty = bar_w.saturating_sub(filled);
            Line::from(vec![
                Span::styled(
                    format!("  {:<9}", label),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:>6} ", format::tokens(*count)),
                    Style::default().fg(*color),
                ),
                Span::styled("█".repeat(filled), Style::default().fg(*color)),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let total = tu.input_tokens
        + tu.output_tokens
        + tu.cache_read_tokens
        + tu.cache_write_tokens
        + tu.thinking_tokens;
    let cache_pct = tu.cache_hit_rate() * 100.0;

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Total   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format::tokens(total),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Cache hit ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:.0}%", cache_pct),
            Style::default()
                .fg(if cache_pct >= 60.0 {
                    Color::Green
                } else if cache_pct >= 30.0 {
                    Color::Yellow
                } else {
                    Color::Red
                })
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}

// ── Insights panel (Dashboard) ────────────────────────────────────────────────

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
        let ctx_pct = s.context_health_fraction() * 100.0;
        let burn = s.burn_rate_per_hour();

        let (grade, grade_color, tip) = if cache_pct >= 70.0 {
            ("A  Excellent", Color::Green, None)
        } else if cache_pct >= 50.0 {
            ("B  Good", Color::Green, None)
        } else if cache_pct >= 30.0 {
            ("C  Fair", Color::Yellow, Some("→ reuse system prompts"))
        } else {
            ("D  Low", Color::Red, Some("→ add persistent system prompt"))
        };

        lines.push(Line::from(vec![
            Span::styled("  Cache  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%  ", cache_pct),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
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

        let (ctx_label, ctx_color) = if ctx_pct >= 90.0 {
            ("⚠  Run /compact now!", Color::Red)
        } else if ctx_pct >= 75.0 {
            ("↑  Consider /compact", Color::Yellow)
        } else {
            ("✓  Healthy", Color::Green)
        };

        lines.push(Line::from(vec![
            Span::styled("  Context ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%  ", ctx_pct),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(ctx_label, Style::default().fg(ctx_color)),
        ]));
        lines.push(Line::from(""));

        if burn > 0.0 {
            let now = Local::now();
            let hours_left = 24.0 - now.hour() as f64 - now.minute() as f64 / 60.0;
            let projected = s.total_cost + burn * hours_left;
            let week_proj = projected * 5.0;

            lines.push(Line::from(vec![
                Span::styled("  Session    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format::cost(s.total_cost),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
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
                Span::styled(
                    format::cost(week_proj),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(""));
        }

        let model_color = model_color_for(&s.model);
        lines.push(Line::from(vec![
            Span::styled("  Model  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format::model_short_name(&s.model),
                Style::default()
                    .fg(model_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

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

        // CLAUDE.md quality
        if let Some(score) = s.claudemd_score {
            lines.push(Line::from(""));
            let sc = if score >= 70 {
                Color::Green
            } else if score >= 40 {
                Color::Yellow
            } else {
                Color::Red
            };
            let bar_w = 10usize;
            let filled = (score as usize * bar_w / 100).min(bar_w);
            let label = if score >= 70 {
                "Good"
            } else if score >= 40 {
                "Fair"
            } else {
                "Weak"
            };
            lines.push(Line::from(vec![
                Span::styled("  CLAUDE.md ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:>3}/100  ", score),
                    Style::default().fg(sc).add_modifier(Modifier::BOLD),
                ),
                Span::styled("█".repeat(filled), Style::default().fg(sc)),
                Span::styled(
                    "░".repeat(bar_w - filled),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("  {}", label), Style::default().fg(sc)),
            ]));
        }

        // Context quality grade (combines cache efficiency + fill)
        {
            let grade = if cache_pct >= 60.0 && ctx_pct < 75.0 {
                "A"
            } else if cache_pct >= 30.0 && ctx_pct < 90.0 {
                "B"
            } else if ctx_pct >= 90.0 {
                "D"
            } else {
                "C"
            };
            let grade_col = match grade {
                "A" => Color::Green,
                "B" => Color::Cyan,
                "C" => Color::Yellow,
                _ => Color::Red,
            };
            lines.push(Line::from(vec![
                Span::styled("  Context q.", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {}  ", grade),
                    Style::default().fg(grade_col).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("cache {:.0}%  fill {:.0}%", cache_pct, ctx_pct),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        let total_sess = all.len();
        let lifetime_cost: f64 = all.iter().map(|s| s.total_cost).sum();
        let avg_cost = if total_sess > 0 {
            lifetime_cost / total_sess as f64
        } else {
            0.0
        };
        let total_output: u64 = all.iter().map(|s| s.token_usage.output_tokens).sum();
        let total_input: u64 = all.iter().map(|s| s.token_usage.input_tokens).sum();
        let total_cache_r: u64 = all.iter().map(|s| s.token_usage.cache_read_tokens).sum();
        let total_cache_w: u64 = all.iter().map(|s| s.token_usage.cache_write_tokens).sum();
        let overall_cache = {
            let d = total_input + total_cache_r + total_cache_w;
            if d > 0 {
                total_cache_r as f64 / d as f64 * 100.0
            } else {
                0.0
            }
        };

        lines.push(Line::from(Span::styled(
            "  ○  No active session",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Sessions    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                total_sess.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Lifetime    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format::cost(lifetime_cost),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Avg/session ", Style::default().fg(Color::DarkGray)),
            Span::styled(format::cost(avg_cost), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Output tok  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format::tokens(total_output),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Cache hit   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}%", overall_cache),
                Style::default()
                    .fg(if overall_cache >= 50.0 {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if let Some(best) = all
            .iter()
            .filter(|s| s.token_usage.cache_hit_rate() > 0.0)
            .max_by(|a, b| {
                a.token_usage
                    .cache_hit_rate()
                    .total_cmp(&b.token_usage.cache_hit_rate())
            })
        {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Best cache  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{:.0}%  {}",
                        best.token_usage.cache_hit_rate() * 100.0,
                        best.display_path()
                    ),
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
            f.render_widget(
                Paragraph::new("○  No active session")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center),
                inner,
            );
        }
        Some(s) => {
            let path = s.display_path();
            let model = format::model_short_name(&s.model);
            let dur = format::duration(s.duration_secs());
            let cost = format::cost(s.total_cost);
            let burn = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
            let frac = s.context_health_fraction();
            let ctx = format!("{:.0}%", frac * 100.0);
            let cache = format!("{:.0}%", s.token_usage.cache_hit_rate() * 100.0);
            let color = context_color(frac);
            let turns = s.token_usage.output_tokens;

            let rows = vec![
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Green)),
                    Span::styled(
                        path,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  ·  "),
                    Span::styled(model, Style::default().fg(model_color_for(&s.model))),
                    Span::raw("  ·  "),
                    Span::styled(dur, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("     Cost "),
                    Span::styled(
                        &cost,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("   Burn "),
                    Span::styled(&burn, Style::default().fg(Color::DarkGray)),
                    Span::raw("   Context "),
                    Span::styled(
                        &ctx,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("   Cache "),
                    Span::styled(&cache, Style::default().fg(Color::DarkGray)),
                    Span::raw("   Output "),
                    Span::styled(format::tokens(turns), Style::default().fg(Color::DarkGray)),
                ]),
            ];

            let text_area = Rect { height: 2, ..inner };
            let gauge_area = Rect {
                x: inner.x,
                y: inner.y + 3,
                width: inner.width,
                height: 1,
            };

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

    render_spend_cell(f, cols[0], "Today", s.today, s.yesterday);
    render_spend_cell(f, cols[1], "This week", s.this_week, s.prev_week);
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

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(label, Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(
                format::cost(current),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(trend_str, Style::default().fg(trend_color))),
        ])
        .alignment(Alignment::Center),
        area,
    );
}

// ── Session list ──────────────────────────────────────────────────────────────

fn draw_sessions_list(
    f: &mut Frame,
    area: Rect,
    sessions: &[ClaudeSession],
    cursor: Option<usize>,
    scroll: usize,
    show_tags: bool,
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

    let max_rows = (inner.height as usize).saturating_sub(1);
    let visible: Vec<(usize, &ClaudeSession)> = sessions
        .iter()
        .enumerate()
        .skip(scroll)
        .take(max_rows)
        .collect();

    let rows: Vec<Row> = visible
        .iter()
        .map(|(abs_idx, s)| {
            let is_selected = cursor == Some(*abs_idx);
            let dot = if s.is_active { "●" } else { "○" };
            let dot_s = Style::default().fg(if s.is_active {
                Color::Green
            } else {
                Color::DarkGray
            });
            let when = format::relative_time(&s.start_time);
            let dur = format::duration(s.duration_secs());
            let model = format::model_short_name(&s.model);
            let path = s.display_path();
            let title = s.title.as_deref().unwrap_or(path.as_str());
            let label = if title.len() > 34 {
                format!("{}…", &title[..33])
            } else {
                title.to_string()
            };
            let cost = format::cost(s.total_cost);
            let tag_str = if show_tags {
                s.tag
                    .as_deref()
                    .map(|t| {
                        let t = if t.len() > 8 {
                            format!("{}…", &t[..7])
                        } else {
                            t.to_string()
                        };
                        format!("[{}]", t)
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if show_tags {
                Row::new(vec![
                    Cell::from(dot).style(dot_s),
                    Cell::from(when).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(dur).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(model).style(Style::default().fg(model_color_for(&s.model))),
                    Cell::from(label),
                    Cell::from(tag_str).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(cost).style(Style::default().fg(Color::White)),
                ])
            } else {
                Row::new(vec![
                    Cell::from(dot).style(dot_s),
                    Cell::from(when).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(dur).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(model).style(Style::default().fg(model_color_for(&s.model))),
                    Cell::from(label),
                    Cell::from(String::new()),
                    Cell::from(cost).style(Style::default().fg(Color::White)),
                ])
            }
            .style(if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    let hint = if cursor.is_some() {
        " [Enter] detail"
    } else {
        ""
    };
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(2),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Min(18),
                Constraint::Length(11),
                Constraint::Length(8),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("When"),
                Cell::from("Time"),
                Cell::from("Model"),
                Cell::from(format!("Session{}", hint)),
                Cell::from("Tag"),
                Cell::from("Cost"),
            ])
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1),
        inner,
    );

    // Scroll indicator
    if sessions.len() > max_rows && max_rows > 0 {
        let pct = scroll as f64 / (sessions.len() - max_rows).max(1) as f64;
        let thumb = inner.y + (pct * (inner.height as f64 - 1.0)).round() as u16;
        if thumb < inner.y + inner.height {
            f.render_widget(
                Paragraph::new("▐").style(Style::default().fg(Color::DarkGray)),
                Rect {
                    x: inner.x + inner.width - 1,
                    y: thumb,
                    width: 1,
                    height: 1,
                },
            );
        }
    }
}

// ── Session detail overlay ────────────────────────────────────────────────────

fn draw_detail_overlay(
    f: &mut Frame,
    area: Rect,
    s: &ClaudeSession,
    tag_editing: bool,
    tag_input: &str,
    analysis: Option<&ClaudemdAnalysis>,
) {
    let popup = centered_rect(80, 85, area);
    f.render_widget(Clear, popup);

    let title = s.title.as_deref().unwrap_or("Session Detail");
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let model = format::model_short_name(&s.model);
    let model_color = model_color_for(&s.model);
    let dur = format::duration(s.duration_secs());
    let cost = format::cost(s.total_cost);
    let burn = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
    let ctx_pct = format!("{:.0}%", s.context_health_fraction() * 100.0);
    let cache_pct = format!("{:.0}%", s.token_usage.cache_hit_rate() * 100.0);
    let ctx_color = context_color(s.context_health_fraction());

    let source_str = match s.entrypoint.as_deref() {
        Some("claude-vscode") => "VSCode Extension",
        Some("cli") => "Terminal CLI",
        Some("claude-desktop") => "Desktop App",
        Some("claude-jetbrains") => "JetBrains Plugin",
        _ => "—",
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                s.display_path(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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
            stat_span("Cost", &cost, Color::White),
            Span::raw("   "),
            stat_span("Duration", &dur, Color::White),
            Span::raw("   "),
            stat_span("Burn", &burn, Color::DarkGray),
        ]),
        Line::from(vec![
            stat_span("Context", &ctx_pct, ctx_color),
            Span::raw("   "),
            stat_span("Cache hit", &cache_pct, Color::DarkGray),
        ]),
        Line::from(vec![
            Span::styled("Source    ", Style::default().fg(Color::DarkGray)),
            Span::styled(source_str, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Tokens",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )),
        token_line("Input", s.token_usage.input_tokens, Color::White),
        token_line("Output", s.token_usage.output_tokens, Color::Cyan),
        token_line("Cache read", s.token_usage.cache_read_tokens, Color::Blue),
        token_line(
            "Cache write",
            s.token_usage.cache_write_tokens,
            Color::DarkGray,
        ),
    ];

    if s.token_usage.thinking_tokens > 0 {
        lines.push(token_line(
            "Thinking",
            s.token_usage.thinking_tokens,
            Color::Magenta,
        ));
    }

    // Detailed CLAUDE.md block
    lines.push(Line::from(""));
    if let Some(a) = analysis {
        let sc = if a.score >= 70 {
            Color::Green
        } else if a.score >= 40 {
            Color::Yellow
        } else {
            Color::Red
        };
        let label = if a.score >= 70 {
            "Good"
        } else if a.score >= 40 {
            "Fair"
        } else {
            "Weak"
        };
        let bar_w = 8usize;
        let filled = (a.score as usize * bar_w / 100).min(bar_w);
        lines.push(Line::from(vec![
            Span::styled("CLAUDE.md  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>3}/100  ", a.score),
                Style::default().fg(sc).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(sc)),
            Span::styled(
                "░".repeat(bar_w - filled),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("  {}", label), Style::default().fg(sc)),
        ]));

        // Category check row
        let checks: Vec<(&str, bool)> = vec![
            ("Build", a.has_build),
            ("Tests", a.has_tests),
            ("Run", a.has_run),
            ("Structure", a.has_structure),
            ("Conventions", a.has_conventions),
            ("Commands", a.has_commands),
        ];
        let check_str: String = checks
            .iter()
            .map(|(lbl, ok)| {
                if *ok {
                    format!("✓ {}  ", lbl)
                } else {
                    format!("✗ {}  ", lbl)
                }
            })
            .collect();
        let pass_spans: Vec<Span> = checks
            .iter()
            .flat_map(|(lbl, ok)| {
                let col = if *ok { Color::Green } else { Color::Red };
                vec![
                    Span::styled(if *ok { "✓ " } else { "✗ " }, Style::default().fg(col)),
                    Span::styled(format!("{}  ", lbl), Style::default().fg(Color::DarkGray)),
                ]
            })
            .collect();
        let _ = check_str; // suppress unused
        let mut check_line = vec![Span::raw("  ")];
        check_line.extend(pass_spans);
        lines.push(Line::from(check_line));

        // Suggestions
        if !a.suggestions.is_empty() {
            let tip = a
                .suggestions
                .iter()
                .map(|s| *s)
                .collect::<Vec<&str>>()
                .join("  ·  ");
            lines.push(Line::from(vec![
                Span::styled("  → ", Style::default().fg(Color::DarkGray)),
                Span::styled(tip, Style::default().fg(Color::DarkGray)),
            ]));
        }
    } else if let Some(score) = s.claudemd_score {
        let sc = if score >= 70 {
            Color::Green
        } else if score >= 40 {
            Color::Yellow
        } else {
            Color::Red
        };
        lines.push(Line::from(vec![
            Span::styled("CLAUDE.md  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} / 100", score),
                Style::default().fg(sc).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Tag row
    lines.push(Line::from(""));
    if tag_editing {
        lines.push(Line::from(vec![
            Span::styled("Tag  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[{}▌]", tag_input),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Enter=save  Esc=cancel",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        let tag_display = s.tag.as_deref().unwrap_or("(none)");
        lines.push(Line::from(vec![
            Span::styled("Tag  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if s.tag.is_some() {
                    format!("[{}]", tag_display)
                } else {
                    tag_display.to_string()
                },
                Style::default().fg(if s.tag.is_some() {
                    Color::Cyan
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled("  [t] edit", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let gauge_y = inner.y + lines.len() as u16 + 1;
    if gauge_y < inner.y + inner.height {
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(ctx_color).bg(Color::DarkGray))
                .ratio(s.context_health_fraction()),
            Rect {
                x: inner.x,
                y: gauge_y,
                width: inner.width,
                height: 1,
            },
        );
    }

    let footer_y = inner.y + inner.height.saturating_sub(1);
    if footer_y > inner.y {
        let path_short = if s.project_path.len() > (inner.width as usize).saturating_sub(20) {
            format!(
                "…{}",
                &s.project_path[s
                    .project_path
                    .len()
                    .saturating_sub(inner.width as usize - 22)..]
            )
        } else {
            s.project_path.clone()
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(path_short, Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "  [t] tag  [c] copy path  [Esc] back",
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
            Rect {
                x: inner.x,
                y: footer_y,
                width: inner.width,
                height: 1,
            },
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
    let spans: Vec<Span> = if app.cp_name_editing {
        vec![
            Span::styled("  Checkpoint name  ", Style::default().fg(Color::Green)),
            Span::styled(
                "[Enter] save  [Esc] cancel  [Backspace] delete char",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    } else if app.tag_editing {
        vec![
            Span::styled("  Tag edit  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "[Enter] save  [Esc] cancel  [Backspace] delete char",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    } else if app.detail_open.is_some() {
        vec![
            Span::styled("  [Esc] back  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[t] tag  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[c] copy path  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        let mut v = vec![Span::styled(
            "  [←/→] switch  ",
            Style::default().fg(Color::DarkGray),
        )];
        let extra: &[(&str, Color)] = match app.tab {
            Tab::Dashboard => &[
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
            Tab::Sessions => &[
                ("[↑/↓] select  ", Color::DarkGray),
                ("[Enter] detail  ", Color::DarkGray),
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
            Tab::Analytics => &[
                ("[↑/↓] scroll  ", Color::DarkGray),
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
            Tab::Agents => &[
                ("[↑/↓] select  ", Color::DarkGray),
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
            Tab::Skills => &[
                ("[↑/↓] select  ", Color::DarkGray),
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
            Tab::History => &[
                ("[↑/↓] select  ", Color::DarkGray),
                ("[s] save  ", Color::Green),
                ("[w] write ctx  ", Color::Cyan),
                ("[d] delete  ", Color::DarkGray),
                ("[r] refresh  ", Color::DarkGray),
                ("[q] quit", Color::DarkGray),
            ],
        };
        for (txt, col) in extra {
            v.push(Span::styled(*txt, Style::default().fg(*col)));
        }
        v
    };

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Usage panel (Dashboard left-bottom) ──────────────────────────────────────

fn draw_usage_panel(
    f: &mut Frame,
    area: Rect,
    session: Option<&ClaudeSession>,
    all: &[ClaudeSession],
    config: &ClauxConfig,
    account: Option<&AccountInfo>,
) {
    let block = Block::default()
        .title(" Usage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let bar_w = (inner.width as usize).saturating_sub(30).max(6);
    let mut lines: Vec<Line> = vec![];

    // Context window
    if let Some(s) = session {
        let frac = s.context_health_fraction();
        let pct = frac * 100.0;
        let ctx_tok = if s.token_usage.context_window_tokens > 0 {
            s.token_usage.context_window_tokens
        } else {
            s.token_usage.total_context_tokens()
        };
        let filled = (frac * bar_w as f64).round() as usize;
        let empty = bar_w.saturating_sub(filled);
        let col = context_color(frac);
        lines.push(Line::from(vec![
            Span::styled("  Context win  ", Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(col)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("  {:>3.0}%  {}/200k", pct, format::tokens(ctx_tok)),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "  Context win  (no active session)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let now = Local::now();
    let today = now.date_naive();
    let has_active = session.is_some();
    let state_5h = five_hour_state(all, now, config.plan_5h_limit_usd, has_active);
    let state_week = weekly_state(all, now, config.weekly_budget_usd);

    if let Some(limit) = state_5h.limit {
        let frac = state_5h.fraction;
        let filled = (frac * bar_w as f64).round() as usize;
        let empty = bar_w.saturating_sub(filled);
        let col = if frac >= 0.9 {
            Color::Red
        } else if frac >= 0.7 {
            Color::Yellow
        } else {
            Color::Green
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12} ", state_5h.label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(col)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("  {} / ${:.0}", format::cost(state_5h.current), limit),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12} ", state_5h.label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format::cost(state_5h.current),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  (limit unset — claux config set plan-5h-limit N)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    if let Some(reset_at) = state_5h.reset_at {
        lines.push(Line::from(vec![Span::styled(
            format!("               resets {}", reset_at.format("%H:%M")),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if let Some(limit) = state_week.limit {
        let frac = state_week.fraction;
        let filled = (frac * bar_w as f64).round() as usize;
        let empty = bar_w.saturating_sub(filled);
        let col = if frac >= 0.9 {
            Color::Red
        } else if frac >= 0.7 {
            Color::Yellow
        } else {
            Color::Green
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12} ", state_week.label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(col)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("  {}  / ${:.0}", format::cost(state_week.current), limit),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12} ", state_week.label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format::cost(state_week.current),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  (limit unset — claux config set weekly-budget N)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    if let Some(reset_at) = state_week.reset_at {
        lines.push(Line::from(vec![Span::styled(
            format!("               resets {}", reset_at.format("Mon %Y-%m-%d")),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if matches!(state_5h.reason, Some(ProgressReason::SourceUnavailable))
        || matches!(state_week.reason, Some(ProgressReason::SourceUnavailable))
    {
        lines.push(Line::from(Span::styled(
            "  Status       source unavailable or no logs found",
            Style::default().fg(Color::DarkGray),
        )));
    } else if matches!(state_5h.reason, Some(ProgressReason::NoDataYet))
        || matches!(state_week.reason, Some(ProgressReason::NoDataYet))
    {
        lines.push(Line::from(Span::styled(
            "  Status       no data yet in current windows",
            Style::default().fg(Color::DarkGray),
        )));
    } else if matches!(state_5h.reason, Some(ProgressReason::NoActiveSession)) {
        lines.push(Line::from(Span::styled(
            "  Status       no active session (bars based on recent logs)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Credit status
    let credit_str = match account {
        Some(a) if a.has_extra_usage => {
            let monthly_now: f64 = all
                .iter()
                .flat_map(|s| s.daily_costs.iter())
                .filter(|(d, _)| d.year() == today.year() && d.month() == today.month())
                .map(|(_, c)| c)
                .sum();
            match config.monthly_credit_usd {
                Some(cap) => format!(
                    "{}  /  ${:.0} cap  ({:.0}%)",
                    format::cost(monthly_now),
                    cap,
                    monthly_now / cap * 100.0
                ),
                None => format!("enabled  ({} this month)", format::cost(monthly_now)),
            }
        }
        _ => "disabled".to_string(),
    };
    lines.push(Line::from(vec![
        Span::styled("  Credit       ", Style::default().fg(Color::DarkGray)),
        Span::styled(credit_str, Style::default().fg(Color::DarkGray)),
    ]));

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}

// ── Skills screen ─────────────────────────────────────────────────────────────

fn draw_skills_screen(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    draw_skill_list(f, chunks[0], app);
    draw_skill_detail(f, chunks[1], app);
}

fn draw_skill_list(f: &mut Frame, area: Rect, app: &App) {
    let skills = &app.skills;
    let custom = skills
        .iter()
        .filter(|s| s.source == crate::models::SkillSource::Custom)
        .count();
    let builtin = skills.len() - custom;

    let title = format!(
        " Skills ── {} total · {} custom · {} built-in ",
        skills.len(),
        custom,
        builtin
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if skills.is_empty() {
        f.render_widget(
            Paragraph::new("  No skills found — use 'claux skills new <name>' to create one")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let widths = [
        Constraint::Length(2),
        Constraint::Length(18),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Length(6),
    ];

    let scroll = app.skill_scroll;
    let rows: Vec<Row> = skills
        .iter()
        .enumerate()
        .skip(scroll)
        .take(inner.height as usize)
        .map(|(idx, skill)| {
            let is_selected = idx == app.skill_cursor;
            let dot = if skill.source == crate::models::SkillSource::Custom {
                "●"
            } else {
                "○"
            };
            let dot_col = if skill.source == crate::models::SkillSource::Custom {
                Color::Cyan
            } else {
                Color::DarkGray
            };
            let kind = if skill.source == crate::models::SkillSource::Custom {
                "custom"
            } else {
                "builtin"
            };
            let last = skill
                .last_used_ms
                .map(|ms| {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let secs = (now_ms.saturating_sub(ms)) / 1_000;
                    if secs < 3_600 {
                        format!("{}m ago", secs / 60)
                    } else if secs < 86_400 {
                        format!("{}h ago", secs / 3_600)
                    } else {
                        format!("{}d ago", secs / 86_400)
                    }
                })
                .unwrap_or_else(|| "never".to_string());

            Row::new(vec![
                Cell::from(dot).style(Style::default().fg(dot_col)),
                Cell::from(skill.name.clone()),
                Cell::from(kind).style(Style::default().fg(Color::DarkGray)),
                Cell::from(skill.usage_count.to_string())
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(last).style(Style::default().fg(Color::DarkGray)),
                Cell::from(stars(skill.rating)).style(quality_style(skill.rating)),
            ])
            .style(if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("Skill"),
                Cell::from("Type"),
                Cell::from("Uses"),
                Cell::from("Last used"),
                Cell::from("Rating"),
            ])
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1);

    f.render_widget(table, inner);
}

fn draw_skill_detail(f: &mut Frame, area: Rect, app: &App) {
    let selected = app.skills.get(app.skill_cursor);

    let title = selected
        .map(|s| format!(" {} ", s.name))
        .unwrap_or_else(|| " Detail ".to_string());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(skill) = selected else {
        f.render_widget(
            Paragraph::new("  Select a skill above with ↑/↓")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let kind = if skill.source == crate::models::SkillSource::Custom {
        "Custom"
    } else {
        "Built-in"
    };
    let kind_col = if skill.source == crate::models::SkillSource::Custom {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let last_str = skill
        .last_used_ms
        .map(|ms| {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let secs = (now_ms.saturating_sub(ms)) / 1_000;
            if secs < 3_600 {
                format!("{}m ago", secs / 60)
            } else if secs < 86_400 {
                format!("{}h ago", secs / 3_600)
            } else {
                format!("{}d ago", secs / 86_400)
            }
        })
        .unwrap_or_else(|| "never".to_string());

    let mut lines: Vec<Line> = vec![];

    lines.push(Line::from(vec![
        Span::styled("  Name      ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            skill.name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(format!("[{}]", kind), Style::default().fg(kind_col)),
    ]));
    lines.push(Line::from(""));

    if let Some(desc) = &skill.description {
        let max_w = (inner.width as usize).saturating_sub(14).max(10);
        let d = if desc.len() > max_w {
            format!("{}…", desc.chars().take(max_w - 1).collect::<String>())
        } else {
            desc.clone()
        };
        lines.push(Line::from(vec![
            Span::styled("  Desc      ", Style::default().fg(Color::DarkGray)),
            Span::styled(d, Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled("  Uses      ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            skill.usage_count.to_string(),
            Style::default().fg(Color::White),
        ),
        Span::styled("   Last used  ", Style::default().fg(Color::DarkGray)),
        Span::styled(last_str, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Rating    ", Style::default().fg(Color::DarkGray)),
        Span::styled(stars(skill.rating), quality_style(skill.rating)),
    ]));
    lines.push(Line::from(""));

    if skill.source == crate::models::SkillSource::Custom {
        if let Some(content) = &skill.content {
            lines.push(Line::from(Span::styled(
                "  Content preview:",
                Style::default().fg(Color::DarkGray),
            )));
            let line_w = (inner.width as usize).saturating_sub(4).max(10);
            let preview_lines =
                wrap_text(&content.chars().take(300).collect::<String>(), line_w, 5);
            for pl in &preview_lines {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(pl.clone(), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  Built-in skill — content managed by Claude Code",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn model_color_for(model: &str) -> Color {
    let l = model.to_lowercase();
    if l.contains("opus") {
        Color::Magenta
    } else if l.contains("haiku") {
        Color::Green
    } else {
        Color::Blue
    }
}

fn context_color(fraction: f64) -> Color {
    if fraction < 0.70 {
        Color::Blue
    } else if fraction < 0.90 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// `"★★★★☆"` — filled stars then empty stars.
fn stars(score: u8) -> String {
    let n = score.min(5) as usize;
    format!("{}{}", "★".repeat(n), "☆".repeat(5 - n))
}

/// `"[████████░░]"` — progress bar with `width` interior chars.
fn xp_bar(progress: f64, width: usize) -> String {
    let filled = (progress.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!(
        "[{}{}]",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}

fn quality_style(score: u8) -> Style {
    let color = match score {
        5 => Color::Green,
        4 => Color::Cyan,
        3 => Color::Yellow,
        _ => Color::Red,
    };
    Style::default().fg(color)
}

fn quality_label(score: u8) -> &'static str {
    match score {
        5 => "Rich output, task completed cleanly",
        4 => "Good output, task completed",
        3 => "Moderate output, completed",
        2 => "Minimal output or errors detected",
        _ => "Did not complete",
    }
}

/// Human-readable duration for an agent run.
fn agent_duration_str(agent: &AgentRun) -> String {
    let end = agent.end_time.unwrap_or_else(Local::now);
    let secs = (end - agent.start_time).num_seconds().max(0);
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format::duration(secs)
    }
}

/// Simple word-wrap: split `text` into lines of at most `width` chars, max `max_lines`.
fn wrap_text(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(width)
        .take(max_lines)
        .map(|c| c.iter().collect())
        .collect()
}

fn stat_span<'a>(label: &'a str, value: &'a str, color: Color) -> Span<'a> {
    Span::styled(format!("{}  {}", label, value), Style::default().fg(color))
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
    let popup_w = area.width * percent_x / 100;
    let popup_h = area.height * percent_y / 100;
    Rect {
        x: area.x + (area.width - popup_w) / 2,
        y: area.y + (area.height - popup_h) / 2,
        width: popup_w,
        height: popup_h,
    }
}

// ── History screen ────────────────────────────────────────────────────────────

fn draw_history_screen(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    draw_checkpoint_list(f, chunks[0], app);
    draw_checkpoint_detail(f, chunks[1], app);
}

fn draw_checkpoint_list(f: &mut Frame, area: Rect, app: &App) {
    let cps = &app.checkpoints;
    let title = format!(
        " History ── {} checkpoint{} ",
        cps.len(),
        if cps.len() == 1 { "" } else { "s" }
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Reserve bottom row for name-input when editing
    let list_height = if app.cp_name_editing {
        inner.height.saturating_sub(2) as usize
    } else {
        inner.height as usize
    };

    if cps.is_empty() && !app.cp_name_editing {
        f.render_widget(
            Paragraph::new("  No checkpoints — press [s] to save the current project state")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        if app.cp_name_editing {
            draw_cp_name_input(f, inner);
        }
        return;
    }

    let widths = [
        Constraint::Length(10),
        Constraint::Length(22),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(9),
        Constraint::Length(6),
    ];

    let scroll = app.checkpoint_scroll;
    let rows: Vec<Row> = cps
        .iter()
        .enumerate()
        .skip(scroll)
        .take(list_height)
        .map(|(idx, cp)| {
            let is_sel = idx == app.checkpoint_cursor;
            let date = cp.created_at.split('T').next().unwrap_or("").to_string();
            let branch = cp.git_branch.clone().unwrap_or_else(|| "—".to_string());
            let cost = format!("${:.2}", cp.cost_total_usd);
            let files = if cp.files_changed.is_empty() {
                "—".to_string()
            } else {
                cp.files_changed.len().to_string()
            };

            Row::new(vec![
                Cell::from(cp.id.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(cp.name.clone()),
                Cell::from(date).style(Style::default().fg(Color::DarkGray)),
                Cell::from(branch).style(Style::default().fg(Color::Cyan)),
                Cell::from(cost).style(Style::default().fg(Color::Yellow)),
                Cell::from(files).style(Style::default().fg(Color::DarkGray)),
            ])
            .style(if is_sel {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    let table = Table::new(rows, widths).header(
        Row::new(vec![
            Cell::from("ID"),
            Cell::from("Name"),
            Cell::from("Saved"),
            Cell::from("Branch"),
            Cell::from("Cost"),
            Cell::from("Files"),
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    );

    let list_area = if app.cp_name_editing {
        Rect {
            height: list_height as u16,
            ..inner
        }
    } else {
        inner
    };
    f.render_widget(table, list_area);

    if app.cp_name_editing {
        draw_cp_name_input(f, inner);
    }
}

fn draw_cp_name_input(f: &mut Frame, inner: Rect) {
    if inner.height < 2 {
        return;
    }
    let input_y = inner.y + inner.height - 2;
    let input_area = Rect {
        y: input_y,
        height: 1,
        ..inner
    };
    let prompt_line = Line::from(vec![
        Span::styled("  Name: ", Style::default().fg(Color::Green)),
        Span::styled(
            // placeholder — actual buf drawn by caller via app.cp_name_buf
            "".to_string(),
            Style::default().fg(Color::White),
        ),
    ]);
    f.render_widget(Paragraph::new(prompt_line), input_area);
}

fn draw_checkpoint_detail(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Show name input prompt in detail panel when editing
    if app.cp_name_editing {
        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Checkpoint name:  ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}▌]", app.cp_name_buf),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "  Enter to save  ·  Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        f.render_widget(Paragraph::new(lines), inner);
        return;
    }

    let Some(cp) = app.checkpoints.get(app.checkpoint_cursor) else {
        f.render_widget(
            Paragraph::new("  Select a checkpoint above, or press [s] to save one.")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    let date = cp
        .created_at
        .replace('T', "  ")
        .split('+')
        .next()
        .unwrap_or(&cp.created_at)
        .to_string();
    let date = &date[..date.len().min(19)];

    let mut lines: Vec<Line> = vec![Line::from("  ")];

    // Name + ID
    lines.push(Line::from(vec![
        Span::styled("  Name     ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            cp.name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   ({})", cp.id),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Saved    ", Style::default().fg(Color::DarkGray)),
        Span::styled(date.to_string(), Style::default().fg(Color::DarkGray)),
    ]));

    // Git info
    match (&cp.git_branch, &cp.git_commit) {
        (Some(b), Some(c)) => lines.push(Line::from(vec![
            Span::styled("  Branch   ", Style::default().fg(Color::DarkGray)),
            Span::styled(b.clone(), Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("  ·  {}", &c[..c.len().min(8)]),
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        (Some(b), None) => lines.push(Line::from(vec![
            Span::styled("  Branch   ", Style::default().fg(Color::DarkGray)),
            Span::styled(b.clone(), Style::default().fg(Color::Cyan)),
        ])),
        _ => {}
    }

    lines.push(Line::from("  "));

    // Cost + sessions
    lines.push(Line::from(vec![
        Span::styled("  Cost     ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("${:.2} total", cp.cost_total_usd),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!("  ·  ${:.2} this session", cp.session_cost_usd),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("  ·  {} sessions", cp.total_sessions),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // CLAUDE.md score
    if let Some(score) = cp.claudemd_score {
        let (label, col) = if score >= 70 {
            ("Good", Color::Green)
        } else if score >= 40 {
            ("Fair", Color::Yellow)
        } else {
            ("Weak", Color::Red)
        };
        lines.push(Line::from(vec![
            Span::styled("  CLAUDE.md ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}/100", score), Style::default().fg(col)),
            Span::styled(format!("  {}", label), Style::default().fg(col)),
        ]));
    }

    // Files changed
    if !cp.files_changed.is_empty() {
        lines.push(Line::from("  "));
        lines.push(Line::from(Span::styled(
            format!(
                "  Files changed since prior checkpoint  ({})",
                cp.files_changed.len()
            ),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));
        for f_path in cp.files_changed.iter().take(8) {
            lines.push(Line::from(Span::styled(
                format!("    {}", f_path),
                Style::default().fg(Color::DarkGray),
            )));
        }
        if cp.files_changed.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("    … and {} more", cp.files_changed.len() - 8),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Summary
    if !cp.summary.is_empty() {
        lines.push(Line::from("  "));
        lines.push(Line::from(Span::styled(
            "  Summary",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));
        for l in wrap_text(&cp.summary, inner.width.saturating_sub(4) as usize, 3) {
            lines.push(Line::from(Span::styled(
                format!("    {}", l),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from("  "));
    lines.push(Line::from(vec![
        Span::styled(
            "  [w] write .claux/CONTEXT.md  ",
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("[d] delete  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[s] new checkpoint", Style::default().fg(Color::Green)),
    ]));

    let h = lines.len().min(inner.height as usize) as u16;
    f.render_widget(Paragraph::new(lines), Rect { height: h, ..inner });
}
