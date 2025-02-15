Directory Structure:

└── ./
    └── crates
        └── turborepo-ui
            └── src
                ├── tui
                │   ├── app.rs
                │   ├── clipboard.rs
                │   ├── debouncer.rs
                │   ├── event.rs
                │   ├── handle.rs
                │   ├── input.rs
                │   ├── mod.rs
                │   ├── pane.rs
                │   ├── popup.rs
                │   ├── preferences.rs
                │   ├── search.rs
                │   ├── size.rs
                │   ├── spinner.rs
                │   ├── state.rs
                │   ├── table.rs
                │   ├── task.rs
                │   └── term_output.rs
                ├── wui
                │   ├── event.rs
                │   ├── mod.rs
                │   ├── query.rs
                │   ├── sender.rs
                │   └── subscriber.rs
                ├── color_selector.rs
                ├── lib.rs
                ├── line.rs
                ├── logs.rs
                ├── output.rs
                ├── prefixed.rs
                └── sender.rs



---
File: /crates/turborepo-ui/src/tui/app.rs
---

use std::{
    collections::BTreeMap,
    io::{self, Stdout, Write},
    mem,
    time::Duration,
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    widgets::{Clear, TableState},
    Frame, Terminal,
};
use tokio::{
    sync::{mpsc, oneshot},
    time::Instant,
};
use tracing::{debug, trace};
use turbopath::AbsoluteSystemPathBuf;

use crate::tui::popup::{popup, popup_area};

pub const FRAMERATE: Duration = Duration::from_millis(3);
const RESIZE_DEBOUNCE_DELAY: Duration = Duration::from_millis(10);

use super::{
    event::{CacheResult, Direction, OutputLogs, PaneSize, TaskResult},
    input,
    preferences::PreferenceLoader,
    search::SearchResults,
    AppReceiver, Debouncer, Error, Event, InputOptions, SizeInfo, TaskTable, TerminalPane,
};
use crate::{
    tui::{
        task::{Task, TasksByStatus},
        term_output::TerminalOutput,
    },
    ColorConfig,
};

#[derive(Debug, Clone)]
pub enum LayoutSections {
    Pane,
    TaskList,
    Search {
        previous_selection: String,
        results: SearchResults,
    },
}

pub struct App<W> {
    size: SizeInfo,
    tasks: BTreeMap<String, TerminalOutput<W>>,
    tasks_by_status: TasksByStatus,
    section_focus: LayoutSections,
    task_list_scroll: TableState,
    selected_task_index: usize,
    is_task_selection_pinned: bool,
    showing_help_popup: bool,
    done: bool,
    preferences: PreferenceLoader,
}

impl<W> App<W> {
    pub fn new(rows: u16, cols: u16, tasks: Vec<String>, preferences: PreferenceLoader) -> Self {
        debug!("tasks: {tasks:?}");
        let size = SizeInfo::new(rows, cols, tasks.iter().map(|s| s.as_str()));

        // Initializes with the planned tasks
        // and will mutate as tasks change
        // to running, finished, etc.
        let mut task_list = tasks.clone().into_iter().map(Task::new).collect::<Vec<_>>();
        task_list.sort_unstable();
        task_list.dedup();

        let tasks_by_status = TasksByStatus {
            planned: task_list,
            finished: Vec::new(),
            running: Vec::new(),
        };

        let pane_rows = size.pane_rows();
        let pane_cols = size.pane_cols();

        // Attempt to load previous selection. If there isn't one, go to index 0.
        let selected_task_index = preferences
            .active_task()
            .and_then(|active_task| tasks_by_status.active_index(active_task))
            .unwrap_or(0);

        Self {
            size,
            done: false,
            section_focus: LayoutSections::TaskList,
            tasks: tasks_by_status
                .task_names_in_displayed_order()
                .map(|task_name| {
                    (
                        task_name.to_owned(),
                        TerminalOutput::new(pane_rows, pane_cols, None),
                    )
                })
                .collect(),
            selected_task_index,
            tasks_by_status,
            task_list_scroll: TableState::default().with_selected(selected_task_index),
            showing_help_popup: false,
            is_task_selection_pinned: preferences.active_task().is_some(),
            preferences,
        }
    }

    pub fn active_task(&self) -> Result<&str, Error> {
        self.tasks_by_status.task_name(self.selected_task_index)
    }

    fn input_options(&self) -> Result<InputOptions, Error> {
        let has_selection = self.get_full_task()?.has_selection();
        Ok(InputOptions {
            focus: &self.section_focus,
            has_selection,
            is_help_popup_open: self.showing_help_popup,
        })
    }

    fn update_sidebar_toggle(&mut self) {
        let value = !self.preferences.is_task_list_visible();
        self.preferences.set_is_task_list_visible(Some(value));
    }

    fn update_task_selection_pinned_state(&mut self) -> Result<(), Error> {
        // Preferences assume a pinned state when there is an active task.
        // This `None` creates "un-pinned-ness" on the next TUI startup.
        self.preferences.set_active_task(None)?;
        Ok(())
    }

    pub fn get_full_task(&self) -> Result<&TerminalOutput<W>, Error> {
        let active_task = self.active_task()?;
        self.tasks
            .get(active_task)
            .ok_or_else(|| Error::TaskNotFound {
                name: active_task.to_owned(),
            })
    }

    pub fn get_full_task_mut(&mut self) -> Result<&mut TerminalOutput<W>, Error> {
        // Clippy is wrong here, we need this to avoid a borrow checker error
        #[allow(clippy::unnecessary_to_owned)]
        let active_task = self.active_task()?.to_owned();
        self.tasks
            .get_mut(&active_task)
            .ok_or(Error::TaskNotFound { name: active_task })
    }

    fn persist_active_task(&mut self) -> Result<(), Error> {
        let active_task = self.active_task()?;
        self.preferences.set_active_task(
            self.is_task_selection_pinned
                .then(|| active_task.to_owned()),
        )?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn next(&mut self) {
        let num_rows = self.tasks_by_status.count_all();
        if num_rows > 0 {
            self.selected_task_index = (self.selected_task_index + 1) % num_rows;
            self.task_list_scroll.select(Some(self.selected_task_index));
            self.is_task_selection_pinned = true;
            self.persist_active_task().ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn previous(&mut self) {
        let num_rows = self.tasks_by_status.count_all();
        if num_rows > 0 {
            self.selected_task_index = self
                .selected_task_index
                .checked_sub(1)
                .unwrap_or(num_rows - 1);
            self.task_list_scroll.select(Some(self.selected_task_index));
            self.is_task_selection_pinned = true;
            self.persist_active_task().ok();
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn scroll_terminal_output(&mut self, direction: Direction) -> Result<(), Error> {
        self.get_full_task_mut()?.scroll(direction)?;
        Ok(())
    }

    pub fn enter_search(&mut self) -> Result<(), Error> {
        self.section_focus = LayoutSections::Search {
            previous_selection: self.active_task()?.to_string(),
            results: SearchResults::new(&self.tasks_by_status),
        };
        // We set scroll as we want to keep the current selection
        self.is_task_selection_pinned = true;
        Ok(())
    }

    pub fn exit_search(&mut self, restore_scroll: bool) {
        let mut prev_focus = LayoutSections::TaskList;
        mem::swap(&mut self.section_focus, &mut prev_focus);
        if let LayoutSections::Search {
            previous_selection, ..
        } = prev_focus
        {
            if restore_scroll && self.select_task(&previous_selection).is_err() {
                // If the task that was selected is no longer in the task list we reset
                // scrolling.
                self.reset_scroll();
            }
        }
    }

    pub fn search_scroll(&mut self, direction: Direction) -> Result<(), Error> {
        let LayoutSections::Search { results, .. } = &self.section_focus else {
            debug!("scrolling search while not searching");
            return Ok(());
        };
        let new_selection = match direction {
            Direction::Up => results.first_match(
                self.tasks_by_status
                    .task_names_in_displayed_order()
                    .rev()
                    // We skip all of the tasks that are at or after the current selection
                    .skip(self.tasks_by_status.count_all() - self.selected_task_index),
            ),
            Direction::Down => results.first_match(
                self.tasks_by_status
                    .task_names_in_displayed_order()
                    .skip(self.selected_task_index + 1),
            ),
        };
        if let Some(new_selection) = new_selection {
            let new_selection = new_selection.to_owned();
            self.select_task(&new_selection)?;
        }
        Ok(())
    }

    pub fn search_enter_char(&mut self, c: char) -> Result<(), Error> {
        let LayoutSections::Search { results, .. } = &mut self.section_focus else {
            debug!("modifying search query while not searching");
            return Ok(());
        };
        results.modify_query(|s| s.push(c));
        self.update_search_results();
        Ok(())
    }

    pub fn search_remove_char(&mut self) -> Result<(), Error> {
        let LayoutSections::Search { results, .. } = &mut self.section_focus else {
            debug!("modified search query while not searching");
            return Ok(());
        };
        let mut query_was_empty = false;
        results.modify_query(|s| {
            query_was_empty = s.pop().is_none();
        });
        if query_was_empty {
            self.exit_search(true);
        } else {
            self.update_search_results();
        }
        Ok(())
    }

    fn update_search_results(&mut self) {
        let LayoutSections::Search { results, .. } = &self.section_focus else {
            return;
        };

        // if currently selected task is in results stay on it
        // if not we go forward looking for a task in results
        if let Some(result) = results
            .first_match(
                self.tasks_by_status
                    .task_names_in_displayed_order()
                    .skip(self.selected_task_index),
            )
            .or_else(|| results.first_match(self.tasks_by_status.task_names_in_displayed_order()))
        {
            let new_selection = result.to_owned();
            self.is_task_selection_pinned = true;
            self.select_task(&new_selection).expect("todo");
        }
    }

    /// Mark the given task as started.
    /// If planned, pulls it from planned tasks and starts it.
    /// If finished, removes from finished and starts again as new task.
    #[tracing::instrument(skip(self, output_logs))]
    pub fn start_task(&mut self, task: &str, output_logs: OutputLogs) -> Result<(), Error> {
        debug!("starting {task}");
        // Name of currently highlighted task.
        // We will use this after the order switches.
        let highlighted_task = self
            .tasks_by_status
            .task_name(self.selected_task_index)?
            .to_string();

        let mut found_task = false;

        if let Some(planned_idx) = self
            .tasks_by_status
            .planned
            .iter()
            .position(|planned| planned.name() == task)
        {
            let planned = self.tasks_by_status.planned.remove(planned_idx);
            let running = planned.start();
            self.tasks_by_status.running.push(running);

            found_task = true;
        }

        if !found_task {
            return Err(Error::TaskNotFound { name: task.into() });
        }
        self.tasks
            .get_mut(task)
            .ok_or_else(|| Error::TaskNotFound { name: task.into() })?
            .output_logs = Some(output_logs);

        // If user hasn't interacted, keep highlighting top-most task in list.
        self.select_task(&highlighted_task)?;

        Ok(())
    }

    /// Mark the given running task as finished
    /// Errors if given task wasn't a running task
    #[tracing::instrument(skip(self, result))]
    pub fn finish_task(&mut self, task: &str, result: TaskResult) -> Result<(), Error> {
        debug!("finishing task {task}");
        // Name of currently highlighted task.
        // We will use this after the order switches.
        let highlighted_task = self
            .tasks_by_status
            .task_name(self.selected_task_index)?
            .to_string();

        let running_idx = self
            .tasks_by_status
            .running
            .iter()
            .position(|running| running.name() == task)
            .ok_or_else(|| Error::TaskNotFound { name: task.into() })?;

        let running = self.tasks_by_status.running.remove(running_idx);
        self.tasks_by_status
            .insert_finished_task(running.finish(result));

        self.tasks
            .get_mut(task)
            .ok_or_else(|| Error::TaskNotFound { name: task.into() })?
            .task_result = Some(result);

        // Find the highlighted task from before the list movement in the new list.
        self.select_task(&highlighted_task)?;

        Ok(())
    }

    pub fn has_stdin(&self) -> Result<bool, Error> {
        if let Some(term) = self.tasks.get(self.active_task()?) {
            Ok(term.stdin.is_some())
        } else {
            Ok(false)
        }
    }

    pub fn interact(&mut self) -> Result<(), Error> {
        if matches!(self.section_focus, LayoutSections::Pane) {
            self.section_focus = LayoutSections::TaskList
        } else if self.has_stdin()? {
            self.section_focus = LayoutSections::Pane;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn update_tasks(&mut self, tasks: Vec<String>) -> Result<(), Error> {
        if tasks.is_empty() {
            debug!("got request to update task list to empty list, ignoring request");
            return Ok(());
        }
        debug!("updating task list: {tasks:?}");
        let highlighted_task = self.active_task()?.to_owned();
        // Make sure all tasks have a terminal output
        for task in &tasks {
            self.tasks.entry(task.clone()).or_insert_with(|| {
                TerminalOutput::new(self.size.pane_rows(), self.size.pane_cols(), None)
            });
        }
        // Trim the terminal output to only tasks that exist in new list
        self.tasks.retain(|name, _| tasks.contains(name));
        // Update task list
        let mut task_list = tasks.into_iter().map(Task::new).collect::<Vec<_>>();
        task_list.sort_unstable();
        task_list.dedup();
        self.tasks_by_status = TasksByStatus {
            planned: task_list,
            running: Default::default(),
            finished: Default::default(),
        };

        // Task that was selected may have been removed, go back to top if this happens
        if self.select_task(&highlighted_task).is_err() {
            trace!("{highlighted_task} was removed from list");
            self.reset_scroll();
        }

        if let LayoutSections::Search { results, .. } = &mut self.section_focus {
            results.update_tasks(&self.tasks_by_status);
        }
        self.update_search_results();

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn restart_tasks(&mut self, tasks: Vec<String>) -> Result<(), Error> {
        debug!("tasks to reset: {tasks:?}");
        let highlighted_task = self.active_task()?.to_owned();
        // Make sure all tasks have a terminal output
        for task in &tasks {
            self.tasks.entry(task.clone()).or_insert_with(|| {
                TerminalOutput::new(self.size.pane_rows(), self.size.pane_cols(), None)
            });
        }

        self.tasks_by_status
            .restart_tasks(tasks.iter().map(|s| s.as_str()));

        if let LayoutSections::Search { results, .. } = &mut self.section_focus {
            results.update_tasks(&self.tasks_by_status);
        }

        if self.select_task(&highlighted_task).is_err() {
            debug!("was unable to find {highlighted_task} after restart");
            self.reset_scroll();
        }

        Ok(())
    }

    /// Persist all task output to the after closing the TUI
    pub fn persist_tasks(&mut self, started_tasks: Vec<String>) -> std::io::Result<()> {
        for (task_name, task) in started_tasks.into_iter().filter_map(|started_task| {
            (Some(started_task.clone())).zip(self.tasks.get(&started_task))
        }) {
            task.persist_screen(&task_name)?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_status(
        &mut self,
        task: String,
        status: String,
        result: CacheResult,
    ) -> Result<(), Error> {
        let task = self
            .tasks
            .get_mut(&task)
            .ok_or_else(|| Error::TaskNotFound {
                name: task.to_owned(),
            })?;
        task.status = Some(status);
        task.cache_result = Some(result);
        Ok(())
    }

    pub fn handle_mouse(&mut self, mut event: crossterm::event::MouseEvent) -> Result<(), Error> {
        let table_width = self.size.task_list_width();
        debug!("original mouse event: {event:?}, table_width: {table_width}");
        // Only handle mouse event if it happens inside of pane
        // We give a 1 cell buffer to make it easier to select the first column of a row
        if event.row > 0 && event.column >= table_width {
            // Subtract 1 from the y axis due to the title of the pane
            event.row -= 1;
            // Subtract the width of the table
            event.column -= table_width;
            debug!("translated mouse event: {event:?}");

            let task = self.get_full_task_mut()?;
            task.handle_mouse(event)?;
        }

        Ok(())
    }

    pub fn copy_selection(&self) -> Result<(), Error> {
        let task = self.get_full_task()?;
        let Some(text) = task.copy_selection() else {
            return Ok(());
        };
        super::copy_to_clipboard(&text);
        Ok(())
    }

    fn select_task(&mut self, task_name: &str) -> Result<(), Error> {
        if !self.is_task_selection_pinned {
            return Ok(());
        }

        let Some(new_index_to_highlight) = self
            .tasks_by_status
            .task_names_in_displayed_order()
            .position(|task| task == task_name)
        else {
            return Err(Error::TaskNotFound {
                name: task_name.to_owned(),
            });
        };
        self.selected_task_index = new_index_to_highlight;
        self.task_list_scroll.select(Some(new_index_to_highlight));

        Ok(())
    }

    /// Resets scroll state
    pub fn reset_scroll(&mut self) {
        self.is_task_selection_pinned = false;
        self.task_list_scroll.select(Some(0));
        self.selected_task_index = 0;
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.size.resize(rows, cols);
        let pane_rows = self.size.pane_rows();
        let pane_cols = self.size.pane_cols();
        self.tasks.values_mut().for_each(|term| {
            term.resize(pane_rows, pane_cols);
        })
    }
}

impl<W: Write> App<W> {
    /// Insert a stdin to be associated with a task
    pub fn insert_stdin(&mut self, task: &str, stdin: Option<W>) -> Result<(), Error> {
        let task = self
            .tasks
            .get_mut(task)
            .ok_or_else(|| Error::TaskNotFound {
                name: task.to_owned(),
            })?;
        task.stdin = stdin;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn forward_input(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if matches!(self.section_focus, LayoutSections::Pane) {
            let task_output = self.get_full_task_mut()?;
            if let Some(stdin) = &mut task_output.stdin {
                stdin.write_all(bytes).map_err(|e| Error::Stdin {
                    name: self
                        .active_task()
                        .unwrap_or("<unable to retrieve task name>")
                        .to_owned(),
                    e,
                })?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip(self, output))]
    pub fn process_output(&mut self, task: &str, output: &[u8]) -> Result<(), Error> {
        let task_output = self
            .tasks
            .get_mut(task)
            .ok_or_else(|| Error::TaskNotFound {
                name: task.to_owned(),
            })?;
        task_output.process(output);
        Ok(())
    }
}

/// Handle the rendering of the `App` widget based on events received by
/// `receiver`
pub async fn run_app(
    tasks: Vec<String>,
    receiver: AppReceiver,
    color_config: ColorConfig,
    repo_root: &AbsoluteSystemPathBuf,
) -> Result<(), Error> {
    let mut terminal = startup(color_config)?;
    let size = terminal.size()?;
    let preferences = PreferenceLoader::new(repo_root)?;

    let mut app: App<Box<dyn io::Write + Send>> =
        App::new(size.height, size.width, tasks, preferences);
    let (crossterm_tx, crossterm_rx) = mpsc::channel(1024);
    input::start_crossterm_stream(crossterm_tx);

    let (result, callback) =
        match run_app_inner(&mut terminal, &mut app, receiver, crossterm_rx).await {
            Ok(callback) => (Ok(()), callback),
            Err(err) => {
                debug!("tui shutting down: {err}");
                (Err(err), None)
            }
        };

    cleanup(terminal, app, callback)?;

    result
}

// Break out inner loop so we can use `?` without worrying about cleaning up the
// terminal.
async fn run_app_inner<B: Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App<Box<dyn io::Write + Send>>,
    mut receiver: AppReceiver,
    mut crossterm_rx: mpsc::Receiver<crossterm::event::Event>,
) -> Result<Option<oneshot::Sender<()>>, Error> {
    // Render initial state to paint the screen
    terminal.draw(|f| view(app, f))?;
    let mut last_render = Instant::now();
    let mut resize_debouncer = Debouncer::new(RESIZE_DEBOUNCE_DELAY);
    let mut callback = None;
    let mut needs_rerender = true;
    while let Some(event) = poll(app.input_options()?, &mut receiver, &mut crossterm_rx).await {
        // If we only receive ticks, then there's been no state change so no update
        // needed
        if !matches!(event, Event::Tick) {
            needs_rerender = true;
        }

        let mut event = Some(event);
        let mut resize_event = None;
        if matches!(event, Some(Event::Resize { .. })) {
            resize_event = resize_debouncer.update(
                event
                    .take()
                    .expect("we just matched against a present value"),
            );
        }
        if let Some(resize) = resize_event.take().or_else(|| resize_debouncer.query()) {
            // If we got a resize event, make sure to update ratatui backend.
            terminal.autoresize()?;
            update(app, resize)?;
        }
        if let Some(event) = event {
            callback = update(app, event)?;
            if app.done {
                break;
            }
            if FRAMERATE <= last_render.elapsed() && needs_rerender {
                terminal.draw(|f| view(app, f))?;
                last_render = Instant::now();
                needs_rerender = false;
            }
        }
    }

    Ok(callback)
}

/// Blocking poll for events, will only return None if app handle has been
/// dropped
async fn poll(
    input_options: InputOptions<'_>,
    receiver: &mut AppReceiver,
    crossterm_rx: &mut mpsc::Receiver<crossterm::event::Event>,
) -> Option<Event> {
    let input_closed = crossterm_rx.is_closed();

    if input_closed {
        receiver.recv().await
    } else {
        // tokio::select is messing with variable read detection
        #[allow(unused_assignments)]
        let mut event = None;
        loop {
            tokio::select! {
                e = crossterm_rx.recv() => {
                    event = e.and_then(|e| input_options.handle_crossterm_event(e));
                }
                e = receiver.recv() => {
                    event = e;
                }
            }
            if event.is_some() {
                break;
            }
        }
        event
    }
}

const MIN_HEIGHT: u16 = 10;
const MIN_WIDTH: u16 = 20;

pub fn terminal_big_enough() -> Result<bool, Error> {
    let (width, height) = crossterm::terminal::size()?;
    Ok(width >= MIN_WIDTH && height >= MIN_HEIGHT)
}

/// Configures terminal for rendering App
#[tracing::instrument]
fn startup(color_config: ColorConfig) -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    if color_config.should_strip_ansi {
        crossterm::style::force_color_output(false);
    }
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Ensure all pending writes are flushed before we switch to alternative screen
    stdout.flush()?;
    crossterm::execute!(
        stdout,
        crossterm::event::EnableMouseCapture,
        crossterm::terminal::EnterAlternateScreen
    )?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::with_options(
        backend,
        ratatui::TerminalOptions {
            viewport: ratatui::Viewport::Fullscreen,
        },
    )?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

/// Restores terminal to expected state
#[tracing::instrument(skip_all)]
fn cleanup<B: Backend + io::Write>(
    mut terminal: Terminal<B>,
    mut app: App<Box<dyn io::Write + Send>>,
    callback: Option<oneshot::Sender<()>>,
) -> io::Result<()> {
    terminal.clear()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen,
    )?;
    let tasks_started = app.tasks_by_status.tasks_started();
    app.persist_tasks(tasks_started)?;
    app.preferences.flush_to_disk().ok();
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    // We can close the channel now that terminal is back restored to a normal state
    drop(callback);
    Ok(())
}

fn update(
    app: &mut App<Box<dyn io::Write + Send>>,
    event: Event,
) -> Result<Option<oneshot::Sender<()>>, Error> {
    match event {
        Event::StartTask { task, output_logs } => {
            app.start_task(&task, output_logs)?;
        }
        Event::TaskOutput { task, output } => {
            app.process_output(&task, &output)?;
        }
        Event::Status {
            task,
            status,
            result,
        } => {
            app.set_status(task, status, result)?;
        }
        Event::InternalStop => {
            debug!("shutting down due to internal failure");
            app.done = true;
        }
        Event::Stop(callback) => {
            debug!("shutting down due to message");
            app.done = true;
            return Ok(Some(callback));
        }
        Event::Tick => {
            // app.table.tick();
        }
        Event::EndTask { task, result } => {
            app.finish_task(&task, result)?;
        }
        Event::Up => {
            app.previous();
        }
        Event::Down => {
            app.next();
        }
        Event::ScrollUp => {
            app.is_task_selection_pinned = true;
            app.scroll_terminal_output(Direction::Up)?;
        }
        Event::ScrollDown => {
            app.is_task_selection_pinned = true;
            app.scroll_terminal_output(Direction::Down)?;
        }
        Event::EnterInteractive => {
            app.is_task_selection_pinned = true;
            app.interact()?;
        }
        Event::ExitInteractive => {
            app.is_task_selection_pinned = true;
            app.interact()?;
        }
        Event::TogglePinnedTask => {
            app.update_task_selection_pinned_state()?;
        }
        Event::ToggleSidebar => {
            app.update_sidebar_toggle();
        }
        Event::ToggleHelpPopup => {
            app.showing_help_popup = !app.showing_help_popup;
        }
        Event::Input { bytes } => {
            app.forward_input(&bytes)?;
        }
        Event::SetStdin { task, stdin } => {
            app.insert_stdin(&task, Some(stdin))?;
        }
        Event::UpdateTasks { tasks } => {
            app.update_tasks(tasks)?;
        }
        Event::Mouse(m) => {
            app.handle_mouse(m)?;
        }
        Event::CopySelection => {
            app.copy_selection()?;
        }
        Event::RestartTasks { tasks } => {
            app.restart_tasks(tasks)?;
        }
        Event::Resize { rows, cols } => {
            app.resize(rows, cols);
        }
        Event::SearchEnter => {
            app.enter_search()?;
        }
        Event::SearchExit { restore_scroll } => {
            app.exit_search(restore_scroll);
        }
        Event::SearchScroll { direction } => {
            app.search_scroll(direction)?;
        }
        Event::SearchEnterChar(c) => {
            app.search_enter_char(c)?;
        }
        Event::SearchBackspace => {
            app.search_remove_char()?;
        }
        Event::PaneSizeQuery(callback) => {
            // If caller has already hung up do nothing
            callback
                .send(PaneSize {
                    rows: app.size.pane_rows(),
                    cols: app.size.pane_cols(),
                })
                .ok();
        }
    }
    Ok(None)
}

fn view<W>(app: &mut App<W>, f: &mut Frame) {
    let cols = app.size.pane_cols();
    let horizontal = if app.preferences.is_task_list_visible() {
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(cols)])
    } else {
        Layout::horizontal([Constraint::Max(0), Constraint::Length(cols)])
    };
    let [table, pane] = horizontal.areas(f.size());

    let active_task = app.active_task().unwrap().to_string();

    let output_logs = app.tasks.get(&active_task).unwrap();
    let pane_to_render: TerminalPane<W> = TerminalPane::new(
        output_logs,
        &active_task,
        &app.section_focus,
        app.preferences.is_task_list_visible(),
    );

    let table_to_render = TaskTable::new(&app.tasks_by_status);

    f.render_stateful_widget(&table_to_render, table, &mut app.task_list_scroll);
    f.render_widget(&pane_to_render, pane);

    if app.showing_help_popup {
        let area = popup_area(*f.buffer_mut().area());
        let area = area.intersection(*f.buffer_mut().area());
        f.render_widget(Clear, area); // Clears background underneath popup
        f.render_widget(popup(area), area);
    }
}



---
File: /crates/turborepo-ui/src/tui/clipboard.rs
---

// Inspired by https://github.com/pvolok/mprocs/blob/master/src/clipboard.rs
use std::process::Stdio;

use base64::Engine;
use which::which;

pub fn copy_to_clipboard(s: &str) {
    match copy_impl(s, &PROVIDER) {
        Ok(()) => (),
        Err(err) => tracing::debug!("Unable to copy: {}", err.to_string()),
    }
}

#[allow(dead_code)]
enum Provider {
    OSC52,
    Exec(&'static str, Vec<&'static str>),
    #[cfg(windows)]
    Win,
    NoOp,
}

#[cfg(windows)]
fn detect_copy_provider() -> Provider {
    Provider::Win
}

#[cfg(target_os = "macos")]
fn detect_copy_provider() -> Provider {
    if let Some(provider) = check_prog("pbcopy", &[]) {
        return provider;
    }
    Provider::OSC52
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn detect_copy_provider() -> Provider {
    // Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Some(provider) = check_prog("wl-copy", &["--type", "text/plain"]) {
            return provider;
        }
    }
    // X11
    if std::env::var("DISPLAY").is_ok() {
        if let Some(provider) = check_prog("xclip", &["-i", "-selection", "clipboard"]) {
            return provider;
        }
        if let Some(provider) = check_prog("xsel", &["-i", "-b"]) {
            return provider;
        }
    }
    // Termux
    if let Some(provider) = check_prog("termux-clipboard-set", &[]) {
        return provider;
    }
    // Tmux
    if std::env::var("TMUX").is_ok() {
        if let Some(provider) = check_prog("tmux", &["load-buffer", "-"]) {
            return provider;
        }
    }

    Provider::OSC52
}

#[allow(dead_code)]
fn check_prog(cmd: &'static str, args: &[&'static str]) -> Option<Provider> {
    if which(cmd).is_ok() {
        Some(Provider::Exec(cmd, args.to_vec()))
    } else {
        None
    }
}

fn copy_impl(s: &str, provider: &Provider) -> std::io::Result<()> {
    match provider {
        Provider::OSC52 => {
            let mut stdout = std::io::stdout().lock();
            use std::io::Write;
            write!(
                &mut stdout,
                "\x1b]52;;{}\x07",
                base64::engine::general_purpose::STANDARD.encode(s)
            )?;
        }

        Provider::Exec(prog, args) => {
            let mut child = std::process::Command::new(prog)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            // Do not exit early if we fail to write to the clipboard, make sure we attempt
            // to wait on the clipboard to exit to avoid a zombie process.
            let write_result =
                std::io::Write::write_all(&mut child.stdin.as_ref().unwrap(), s.as_bytes());
            let wait_result = child.wait();
            write_result?;
            wait_result?;
        }

        #[cfg(windows)]
        Provider::Win => clipboard_win::set_clipboard_string(s)
            .map_err(|e| std::io::Error::other(e.to_string()))?,

        Provider::NoOp => (),
    };

    Ok(())
}

lazy_static::lazy_static! {
  static ref PROVIDER: Provider = detect_copy_provider();
}



---
File: /crates/turborepo-ui/src/tui/debouncer.rs
---

use std::time::{Duration, Instant};

pub struct Debouncer<T> {
    value: Option<T>,
    duration: Duration,
    start: Option<Instant>,
}

impl<T> Debouncer<T> {
    /// Creates a new debouncer that will yield the latest value after the
    /// provided duration Duration is reset after the debouncer yields a
    /// value.
    pub fn new(duration: Duration) -> Self {
        Self {
            value: None,
            duration,
            start: None,
        }
    }

    /// Returns a value if debouncer duration has elapsed.
    #[must_use]
    pub fn query(&mut self) -> Option<T> {
        if self
            .start
            .is_some_and(|start| start.elapsed() >= self.duration)
        {
            self.start = None;
            self.value.take()
        } else {
            None
        }
    }

    /// Updates debouncer with given value. Returns a value if debouncer
    /// duration has elapsed.
    #[must_use]
    pub fn update(&mut self, value: T) -> Option<T> {
        self.insert_value(Some(value));
        self.query()
    }

    fn insert_value(&mut self, value: Option<T>) {
        // If there isn't a start set, bump it
        self.start.get_or_insert_with(Instant::now);
        if let Some(value) = value {
            self.value = Some(value);
        }
    }
}


---
File: /crates/turborepo-ui/src/tui/event.rs
---

use async_graphql::Enum;
use serde::Serialize;
use tokio::sync::oneshot;

pub enum Event {
    StartTask {
        task: String,
        output_logs: OutputLogs,
    },
    TaskOutput {
        task: String,
        output: Vec<u8>,
    },
    EndTask {
        task: String,
        result: TaskResult,
    },
    Status {
        task: String,
        status: String,
        result: CacheResult,
    },
    PaneSizeQuery(oneshot::Sender<PaneSize>),
    Stop(oneshot::Sender<()>),
    // Stop initiated by the TUI itself
    InternalStop,
    Tick,
    Up,
    Down,
    ScrollUp,
    ScrollDown,
    SetStdin {
        task: String,
        stdin: Box<dyn std::io::Write + Send>,
    },
    EnterInteractive,
    ExitInteractive,
    Input {
        bytes: Vec<u8>,
    },
    UpdateTasks {
        tasks: Vec<String>,
    },
    Mouse(crossterm::event::MouseEvent),
    CopySelection,
    RestartTasks {
        tasks: Vec<String>,
    },
    Resize {
        rows: u16,
        cols: u16,
    },
    ToggleSidebar,
    ToggleHelpPopup,
    TogglePinnedTask,
    SearchEnter,
    SearchExit {
        restore_scroll: bool,
    },
    SearchScroll {
        direction: Direction,
    },
    SearchEnterChar(char),
    SearchBackspace,
}

pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Enum)]
pub enum TaskResult {
    Success,
    Failure,
    CacheHit,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Enum)]
pub enum CacheResult {
    Hit,
    Miss,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Enum)]
pub enum OutputLogs {
    // Entire task output is persisted after run
    Full,
    // None of a task output is persisted after run
    None,
    // Only the status line of a task is persisted
    HashOnly,
    // Output is only persisted if it is a cache miss
    NewOnly,
    // Output is only persisted if the task failed
    ErrorsOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneSize {
    pub rows: u16,
    pub cols: u16,
}


---
File: /crates/turborepo-ui/src/tui/handle.rs
---

use tokio::sync::{mpsc, oneshot};

use super::{
    app::FRAMERATE,
    event::{CacheResult, OutputLogs, PaneSize},
    Error, Event, TaskResult,
};
use crate::sender::{TaskSender, UISender};

/// Struct for sending app events to TUI rendering
#[derive(Debug, Clone)]
pub struct TuiSender {
    primary: mpsc::UnboundedSender<Event>,
}

/// Struct for receiving app events
pub struct AppReceiver {
    primary: mpsc::UnboundedReceiver<Event>,
}

impl TuiSender {
    /// Create a new channel for sending app events.
    ///
    /// AppSender is meant to be held by the actual task runner
    /// AppReceiver should be passed to `crate::tui::run_app`
    pub fn new() -> (Self, AppReceiver) {
        let (primary_tx, primary_rx) = mpsc::unbounded_channel();
        let tick_sender = primary_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(FRAMERATE);
            loop {
                interval.tick().await;
                if tick_sender.send(Event::Tick).is_err() {
                    break;
                }
            }
        });
        (
            Self {
                primary: primary_tx,
            },
            AppReceiver {
                primary: primary_rx,
            },
        )
    }
}

impl TuiSender {
    pub fn start_task(&self, task: String, output_logs: OutputLogs) {
        self.primary
            .send(Event::StartTask { task, output_logs })
            .ok();
    }

    pub fn end_task(&self, task: String, result: TaskResult) {
        self.primary.send(Event::EndTask { task, result }).ok();
    }

    pub fn status(&self, task: String, status: String, result: CacheResult) {
        self.primary
            .send(Event::Status {
                task,
                status,
                result,
            })
            .ok();
    }

    pub fn set_stdin(&self, task: String, stdin: Box<dyn std::io::Write + Send>) {
        self.primary.send(Event::SetStdin { task, stdin }).ok();
    }

    /// Construct a sender configured for a specific task
    pub fn task(&self, task: String) -> TaskSender {
        TaskSender {
            name: task,
            handle: UISender::Tui(self.clone()),
            logs: Default::default(),
        }
    }

    /// Stop rendering TUI and restore terminal to default configuration
    pub async fn stop(&self) {
        let (callback_tx, callback_rx) = oneshot::channel();
        // Send stop event, if receiver has dropped ignore error as
        // it'll be a no-op.
        self.primary.send(Event::Stop(callback_tx)).ok();
        // Wait for callback to be sent or the channel closed.
        callback_rx.await.ok();
    }

    /// Update the list of tasks displayed in the TUI
    pub fn update_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        Ok(self
            .primary
            .send(Event::UpdateTasks { tasks })
            .map_err(|err| Error::Mpsc(err.to_string()))?)
    }

    pub fn output(&self, task: String, output: Vec<u8>) -> Result<(), crate::Error> {
        Ok(self
            .primary
            .send(Event::TaskOutput { task, output })
            .map_err(|err| Error::Mpsc(err.to_string()))?)
    }

    /// Restart the list of tasks displayed in the TUI
    pub fn restart_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        Ok(self
            .primary
            .send(Event::RestartTasks { tasks })
            .map_err(|err| Error::Mpsc(err.to_string()))?)
    }

    /// Fetches the size of the terminal pane
    pub async fn pane_size(&self) -> Option<PaneSize> {
        let (callback_tx, callback_rx) = oneshot::channel();
        // Send query, if no receiver to handle the request return None
        self.primary.send(Event::PaneSizeQuery(callback_tx)).ok()?;
        // Wait for callback to be sent
        callback_rx.await.ok()
    }
}

impl AppReceiver {
    /// Receive an event, producing a tick event if no events are rec eived by
    /// the deadline.
    pub async fn recv(&mut self) -> Option<Event> {
        self.primary.recv().await
    }
}



---
File: /crates/turborepo-ui/src/tui/input.rs
---

use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::debug;

use super::{
    app::LayoutSections,
    event::{Direction, Event},
};

#[derive(Debug, Clone, Copy)]
pub struct InputOptions<'a> {
    pub focus: &'a LayoutSections,
    pub has_selection: bool,
    pub is_help_popup_open: bool,
}

pub fn start_crossterm_stream(tx: mpsc::Sender<crossterm::event::Event>) -> Option<JoinHandle<()>> {
    // quick check if stdin is tty
    if !atty::is(atty::Stream::Stdin) {
        return None;
    }

    let mut events = EventStream::new();
    Some(tokio::spawn(async move {
        while let Some(Ok(event)) = events.next().await {
            if tx.send(event).await.is_err() {
                break;
            }
        }
    }))
}

impl InputOptions<'_> {
    /// Maps a crossterm::event::Event to a tui::Event
    pub fn handle_crossterm_event(self, event: crossterm::event::Event) -> Option<Event> {
        match event {
            crossterm::event::Event::Key(k) => translate_key_event(self, k),
            crossterm::event::Event::Mouse(m) => match m.kind {
                crossterm::event::MouseEventKind::ScrollDown => Some(Event::ScrollDown),
                crossterm::event::MouseEventKind::ScrollUp => Some(Event::ScrollUp),
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
                | crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                    Some(Event::Mouse(m))
                }
                _ => None,
            },
            crossterm::event::Event::Resize(cols, rows) => Some(Event::Resize { rows, cols }),
            _ => None,
        }
    }
}

/// Converts a crossterm key event into a TUI interaction event
fn translate_key_event(options: InputOptions, key_event: KeyEvent) -> Option<Event> {
    // On Windows events for releasing a key are produced
    // We skip these to avoid emitting 2 events per key press.
    // There is still a `Repeat` event for when a key is held that will pass through
    // this guard.
    if key_event.kind == KeyEventKind::Release {
        return None;
    }
    match key_event.code {
        KeyCode::Char('c') if key_event.modifiers == crossterm::event::KeyModifiers::CONTROL => {
            ctrl_c();
            Some(Event::InternalStop)
        }
        KeyCode::Char('c') if options.has_selection => Some(Event::CopySelection),
        // Interactive branches
        KeyCode::Char('z')
            if matches!(options.focus, LayoutSections::Pane)
                && key_event.modifiers == crossterm::event::KeyModifiers::CONTROL =>
        {
            Some(Event::ExitInteractive)
        }
        // If we're in interactive mode, convert the key event to bytes to send to stdin
        _ if matches!(options.focus, LayoutSections::Pane) => Some(Event::Input {
            bytes: encode_key(key_event),
        }),
        // If we're on the list and user presses `/` enter search mode
        KeyCode::Char('/') if matches!(options.focus, LayoutSections::TaskList) => {
            Some(Event::SearchEnter)
        }
        KeyCode::Esc if options.is_help_popup_open => Some(Event::ToggleHelpPopup),
        KeyCode::Esc if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchExit {
                restore_scroll: true,
            })
        }
        KeyCode::Enter if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchExit {
                restore_scroll: false,
            })
        }
        KeyCode::Up if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchScroll {
                direction: Direction::Up,
            })
        }
        KeyCode::Down if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchScroll {
                direction: Direction::Down,
            })
        }
        KeyCode::Backspace if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchBackspace)
        }
        KeyCode::Char(c) if matches!(options.focus, LayoutSections::Search { .. }) => {
            Some(Event::SearchEnterChar(c))
        }
        // Fall through if we aren't in interactive mode
        KeyCode::Char('h') => Some(Event::ToggleSidebar),
        KeyCode::Char('u') => Some(Event::ScrollUp),
        KeyCode::Char('d') => Some(Event::ScrollDown),
        KeyCode::Char('m') => Some(Event::ToggleHelpPopup),
        KeyCode::Char('p') => Some(Event::TogglePinnedTask),
        KeyCode::Up | KeyCode::Char('k') => Some(Event::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Event::Down),
        KeyCode::Enter | KeyCode::Char('i') => Some(Event::EnterInteractive),
        _ => None,
    }
}

#[cfg(unix)]
fn ctrl_c() -> Option<Event> {
    use nix::sys::signal;
    match signal::raise(signal::SIGINT) {
        Ok(_) => None,
        // We're unable to send the signal, stop rendering to force shutdown
        Err(_) => {
            debug!("unable to send sigint, shutting down");
            Some(Event::InternalStop)
        }
    }
}

#[cfg(windows)]
fn ctrl_c() -> Option<Event> {
    use windows_sys::Win32::{
        Foundation::{BOOL, TRUE},
        System::Console::GenerateConsoleCtrlEvent,
    };
    // First parameter corresponds to what event to generate, 0 is a Ctrl-C
    let ctrl_c_event = 0x0;
    // Second parameter corresponds to which process group to send the event to.
    // If 0 is passed the event gets sent to every process connected to the current
    // Console.
    let process_group_id = 0x0;
    let success: BOOL = unsafe {
        // See docs https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent
        GenerateConsoleCtrlEvent(ctrl_c_event, process_group_id)
    };
    if success == TRUE {
        None
    } else {
        // We're unable to send the Ctrl-C event, stop rendering to force shutdown
        debug!("unable to send sigint, shutting down");
        Some(Event::InternalStop)
    }
}

// Inspired by mprocs encode_term module
// https://github.com/pvolok/mprocs/blob/08d17adebd110501106f86124ef1955fb2beb881/src/encode_term.rs
fn encode_key(key: KeyEvent) -> Vec<u8> {
    use crossterm::event::KeyCode::*;

    if key.kind == KeyEventKind::Release {
        return Vec::new();
    }

    let code = key.code;
    let mods = key.modifiers;

    let mut buf = String::new();

    let code = normalize_shift_to_upper_case(code, &mods);

    // Normalize Backspace and Delete
    let code = match code {
        Char('\x7f') => KeyCode::Backspace,
        Char('\x08') => KeyCode::Delete,
        c => c,
    };

    match code {
        Char(c) if mods.contains(KeyModifiers::CONTROL) && ctrl_mapping(c).is_some() => {
            let c = ctrl_mapping(c).unwrap();
            if mods.contains(KeyModifiers::ALT) {
                buf.push(0x1b as char);
            }
            buf.push(c);
        }

        // When alt is pressed, send escape first to indicate to the peer that
        // ALT is pressed.  We do this only for ascii alnum characters because
        // eg: on macOS generates altgr style glyphs and keeps the ALT key
        // in the modifier set.  This confuses eg: zsh which then just displays
        // <fffffffff> as the input, so we want to avoid that.
        Char(c)
            if (c.is_ascii_alphanumeric() || c.is_ascii_punctuation())
                && mods.contains(KeyModifiers::ALT) =>
        {
            buf.push(0x1b as char);
            buf.push(c);
        }

        Enter | Esc | Backspace => {
            let c = match code {
                Enter => '\r',
                Esc => '\x1b',
                // Backspace sends the default VERASE which is confusingly
                // the DEL ascii codepoint
                Backspace => '\x7f',
                _ => unreachable!(),
            };
            if mods.contains(KeyModifiers::ALT) {
                buf.push(0x1b as char);
            }
            buf.push(c);
        }

        Tab => {
            if mods.contains(KeyModifiers::ALT) {
                buf.push(0x1b as char);
            }
            let mods = mods & !KeyModifiers::ALT;
            if mods == KeyModifiers::CONTROL {
                buf.push_str("\x1b[9;5u");
            } else if mods == KeyModifiers::CONTROL | KeyModifiers::SHIFT {
                buf.push_str("\x1b[1;5Z");
            } else if mods == KeyModifiers::SHIFT {
                buf.push_str("\x1b[Z");
            } else {
                buf.push('\t');
            }
        }

        BackTab => {
            buf.push_str("\x1b[Z");
        }

        Char(c) => {
            buf.push(c);
        }

        Home | End | Up | Down | Right | Left => {
            let c = match code {
                Up => 'A',
                Down => 'B',
                Right => 'C',
                Left => 'D',
                Home => 'H',
                End => 'F',
                _ => unreachable!(),
            };

            if mods.contains(KeyModifiers::ALT)
                || mods.contains(KeyModifiers::SHIFT)
                || mods.contains(KeyModifiers::CONTROL)
            {
                buf.push_str("\x1b[1;");
                buf.push_str(&(1 + encode_modifiers(mods)).to_string());
                buf.push(c);
            } else {
                buf.push_str("\x1b[");
                buf.push(c);
            }
        }

        PageUp | PageDown | Insert | Delete => {
            let c = match code {
                Insert => '2',
                Delete => '3',
                PageUp => '5',
                PageDown => '6',
                _ => unreachable!(),
            };

            if mods.contains(KeyModifiers::ALT)
                || mods.contains(KeyModifiers::SHIFT)
                || mods.contains(KeyModifiers::CONTROL)
            {
                buf.push_str("\x1b[");
                buf.push(c);
                buf.push_str(&(1 + encode_modifiers(mods)).to_string());
            } else {
                buf.push_str("\x1b[");
                buf.push(c);
                buf.push('~');
            }
        }

        F(n) => {
            if mods.is_empty() && n < 5 {
                // F1-F4 are encoded using SS3 if there are no modifiers
                let s = match n {
                    1 => "\x1bOP",
                    2 => "\x1bOQ",
                    3 => "\x1bOR",
                    4 => "\x1bOS",
                    _ => unreachable!("wat?"),
                };
                buf.push_str(s);
            } else {
                // Higher numbered F-keys plus modified F-keys are encoded
                // using CSI instead of SS3.
                let intro = match n {
                    1 => "\x1b[11",
                    2 => "\x1b[12",
                    3 => "\x1b[13",
                    4 => "\x1b[14",
                    5 => "\x1b[15",
                    6 => "\x1b[17",
                    7 => "\x1b[18",
                    8 => "\x1b[19",
                    9 => "\x1b[20",
                    10 => "\x1b[21",
                    11 => "\x1b[23",
                    12 => "\x1b[24",
                    _ => panic!("unhandled fkey number {}", n),
                };
                let encoded_mods = encode_modifiers(mods);
                if encoded_mods == 0 {
                    // If no modifiers are held, don't send the modifier
                    // sequence, as the modifier encoding is a CSI-u extension.
                    buf.push_str(intro);
                    buf.push('~');
                } else {
                    buf.push_str(intro);
                    buf.push(';');
                    buf.push_str(&(1 + encoded_mods).to_string());
                    buf.push('~');
                }
            }
        }

        Null => (),
        CapsLock => (),
        ScrollLock => (),
        NumLock => (),
        PrintScreen => (),
        Pause => (),
        Menu => (),
        KeypadBegin => (),
        Media(_) => (),
        Modifier(_) => (),
    };

    buf.into_bytes()
}

/// Map c to its Ctrl equivalent.
/// In theory, this mapping is simply translating alpha characters
/// to upper case and then masking them by 0x1f, but xterm inherits
/// some built-in translation from legacy X11 so that are some
/// aliased mappings and a couple that might be technically tied
/// to US keyboard layout (particularly the punctuation characters
/// produced in combination with SHIFT) that may not be 100%
/// the right thing to do here for users with non-US layouts.
fn ctrl_mapping(c: char) -> Option<char> {
    Some(match c {
        '@' | '`' | ' ' | '2' => '\x00',
        'A' | 'a' => '\x01',
        'B' | 'b' => '\x02',
        'C' | 'c' => '\x03',
        'D' | 'd' => '\x04',
        'E' | 'e' => '\x05',
        'F' | 'f' => '\x06',
        'G' | 'g' => '\x07',
        'H' | 'h' => '\x08',
        'I' | 'i' => '\x09',
        'J' | 'j' => '\x0a',
        'K' | 'k' => '\x0b',
        'L' | 'l' => '\x0c',
        'M' | 'm' => '\x0d',
        'N' | 'n' => '\x0e',
        'O' | 'o' => '\x0f',
        'P' | 'p' => '\x10',
        'Q' | 'q' => '\x11',
        'R' | 'r' => '\x12',
        'S' | 's' => '\x13',
        'T' | 't' => '\x14',
        'U' | 'u' => '\x15',
        'V' | 'v' => '\x16',
        'W' | 'w' => '\x17',
        'X' | 'x' => '\x18',
        'Y' | 'y' => '\x19',
        'Z' | 'z' => '\x1a',
        '[' | '3' | '{' => '\x1b',
        '\\' | '4' | '|' => '\x1c',
        ']' | '5' | '}' => '\x1d',
        '^' | '6' | '~' => '\x1e',
        '_' | '7' | '/' => '\x1f',
        '8' | '?' => '\x7f', // `Delete`
        _ => return None,
    })
}

/// if SHIFT is held and we have KeyCode::Char('c') we want to normalize
/// that keycode to KeyCode::Char('C'); that is what this function does.
pub fn normalize_shift_to_upper_case(code: KeyCode, modifiers: &KeyModifiers) -> KeyCode {
    if modifiers.contains(KeyModifiers::SHIFT) {
        match code {
            KeyCode::Char(c) if c.is_ascii_lowercase() => KeyCode::Char(c.to_ascii_uppercase()),
            _ => code,
        }
    } else {
        code
    }
}

fn encode_modifiers(mods: KeyModifiers) -> u8 {
    let mut number = 0;
    if mods.contains(KeyModifiers::SHIFT) {
        number |= 1;
    }
    if mods.contains(KeyModifiers::ALT) {
        number |= 2;
    }
    if mods.contains(KeyModifiers::CONTROL) {
        number |= 4;
    }
    number
}

---
File: /crates/turborepo-ui/src/tui/mod.rs
---

mod app;
mod clipboard;
mod debouncer;
pub mod event;
mod handle;
mod input;
mod pane;
mod popup;
mod preferences;
mod search;
mod size;
mod spinner;
mod table;
mod task;
mod term_output;

pub use app::{run_app, terminal_big_enough};
use clipboard::copy_to_clipboard;
use debouncer::Debouncer;
use event::{Event, TaskResult};
pub use handle::{AppReceiver, TuiSender};
use input::InputOptions;
pub use pane::TerminalPane;
use size::SizeInfo;
pub use table::TaskTable;
pub use term_output::TerminalOutput;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send event to TUI: {0}")]
    Mpsc(String),
    #[error("No task found with name '{name}'.")]
    TaskNotFound { name: String },
    #[error("No task at index {index} (only {len} tasks)")]
    TaskNotFoundIndex { index: usize, len: usize },
    #[error("Unable to write to stdin for '{name}': {e}")]
    Stdin { name: String, e: std::io::Error },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Unable to persist preferences.")]
    Preferences(#[from] preferences::Error),
}



---
File: /crates/turborepo-ui/src/tui/pane.rs
---

use ratatui::{
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Widget},
};
use tui_term::widget::PseudoTerminal;

use super::{app::LayoutSections, TerminalOutput};

const EXIT_INTERACTIVE_HINT: &str = "Ctrl-z - Stop interacting";
const ENTER_INTERACTIVE_HINT: &str = "i - Interact";
const HAS_SELECTION: &str = "c - Copy selection";
const SCROLL_LOGS: &str = "u/d - Scroll logs";
const TASK_LIST_HIDDEN: &str = "h - Show task list";

pub struct TerminalPane<'a, W> {
    terminal_output: &'a TerminalOutput<W>,
    task_name: &'a str,
    section: &'a LayoutSections,
    has_sidebar: bool,
}

impl<'a, W> TerminalPane<'a, W> {
    pub fn new(
        terminal_output: &'a TerminalOutput<W>,
        task_name: &'a str,
        section: &'a LayoutSections,
        has_sidebar: bool,
    ) -> Self {
        Self {
            terminal_output,
            section,
            task_name,
            has_sidebar,
        }
    }

    fn has_stdin(&self) -> bool {
        self.terminal_output.stdin.is_some()
    }

    fn footer(&self) -> Line {
        let build_message_vec = |footer_text: &[&str]| -> Line {
            let mut messages = Vec::new();
            messages.extend_from_slice(footer_text);

            if !self.has_sidebar {
                messages.push(TASK_LIST_HIDDEN);
            }

            if self.terminal_output.has_selection() {
                messages.push(HAS_SELECTION);
            }

            // Spaces are used to pad the footer text for aesthetics
            let formatted_messages = format!("   {}", messages.join("   "));

            Line::styled(
                formatted_messages.to_string(),
                Style::default().add_modifier(Modifier::DIM),
            )
            .left_aligned()
        };

        match self.section {
            LayoutSections::Pane => build_message_vec(&[EXIT_INTERACTIVE_HINT]),
            LayoutSections::TaskList if self.has_stdin() => {
                build_message_vec(&[ENTER_INTERACTIVE_HINT, SCROLL_LOGS])
            }
            LayoutSections::TaskList => build_message_vec(&[SCROLL_LOGS]),
            LayoutSections::Search { results, .. } => {
                Line::from(format!("/ {}", results.query())).left_aligned()
            }
        }
    }
}

impl<W> Widget for &TerminalPane<'_, W> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let screen = self.terminal_output.parser.screen();
        let block = Block::default()
            .title(
                self.terminal_output
                    .title(self.task_name)
                    .add_modifier(Modifier::DIM),
            )
            .title_bottom(self.footer());

        let term = PseudoTerminal::new(screen).block(block);
        term.render(area, buf)
    }
}


---
File: /crates/turborepo-ui/src/tui/popup.rs
---

use std::cmp::min;

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, List, ListItem, Padding},
};

const BIND_LIST: [&str; 12] = [
    "m      - Toggle this help popup",
    "↑ or j - Select previous task",
    "↓ or k - Select next task",
    "h      - Toggle task list",
    "p      - Toggle pinned task selection",
    "/      - Filter tasks to search term",
    "ESC    - Clear filter",
    "i      - Interact with task",
    "Ctrl+z - Stop interacting with task",
    "c      - Copy logs selection (Only when logs are selected)",
    "u      - Scroll logs up",
    "d      - Scroll logs down",
];

pub fn popup_area(area: Rect) -> Rect {
    let screen_width = area.width;
    let screen_height = area.height;

    let popup_width = BIND_LIST
        .iter()
        .map(|s| s.len().saturating_add(4))
        .max()
        .unwrap_or(0) as u16;
    let popup_height = min((BIND_LIST.len().saturating_add(4)) as u16, screen_height);

    let x = screen_width.saturating_sub(popup_width) / 2;
    let y = screen_height.saturating_sub(popup_height) / 2;

    let vertical = Layout::vertical([Constraint::Percentage(100)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(100)]).flex(Flex::Center);

    let [vertical_area] = vertical.areas(Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    });

    let [area] = horizontal.areas(vertical_area);

    area
}

pub fn popup(area: Rect) -> List<'static> {
    let available_height = area.height.saturating_sub(4) as usize;

    let items: Vec<ListItem> = BIND_LIST
        .iter()
        .take(available_height)
        .map(|item| ListItem::new(Line::from(*item)))
        .collect();

    let title_bottom = if available_height < BIND_LIST.len() {
        let binds_not_visible = BIND_LIST.len().saturating_sub(available_height);

        let pluralize = if binds_not_visible > 1 { "s" } else { "" };
        let message = format!(
            " {} more bind{}. Make your terminal taller. ",
            binds_not_visible, pluralize
        );
        Line::from(message)
    } else {
        Line::from("")
    };

    let outer = Block::bordered()
        .title(" Keybinds ")
        .title_bottom(title_bottom.to_string())
        .padding(Padding::uniform(1));

    List::new(items).block(outer)
}



---
File: /crates/turborepo-ui/src/tui/preferences.rs
---

use serde::{Deserialize, Serialize};
use turbopath::AbsoluteSystemPathBuf;

const TUI_PREFERENCES_PATH_COMPONENTS: &[&str] = &[".turbo", "preferences", "tui.json"];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

pub struct PreferenceLoader {
    file_path: AbsoluteSystemPathBuf,
    config: Preferences,
}

impl PreferenceLoader {
    pub fn new(repo_root: &AbsoluteSystemPathBuf) -> Result<Self, Error> {
        let file_path = repo_root.join_components(TUI_PREFERENCES_PATH_COMPONENTS);
        let contents = file_path.read_existing_to_string()?;
        let config = contents
            .map(|string| serde_json::from_str(&string))
            .transpose()?
            .unwrap_or_default();

        Ok(Self { file_path, config })
    }

    pub fn is_task_list_visible(&self) -> bool {
        self.config.is_task_list_visible.unwrap_or(true)
    }

    pub fn set_is_task_list_visible(&mut self, value: Option<bool>) {
        self.config.is_task_list_visible = value;
    }

    pub fn active_task(&self) -> Option<&str> {
        let active_task = self.config.active_task.as_deref()?;
        Some(active_task)
    }

    pub fn set_active_task(&mut self, value: Option<String>) -> Result<(), Error> {
        self.config.active_task = value;
        Ok(())
    }

    pub fn flush_to_disk(&self) -> Result<(), Error> {
        self.file_path.ensure_dir()?;
        self.file_path
            .create_with_contents(serde_json::to_string_pretty(&self.config)?)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Preferences {
    pub is_task_list_visible: Option<bool>,
    pub active_task: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            active_task: None,
            is_task_list_visible: Some(true),
        }
    }
}

---
File: /crates/turborepo-ui/src/tui/search.rs
---

use std::{collections::HashSet, sync::Arc};

use super::task::TasksByStatus;

#[derive(Debug, Clone)]
pub struct SearchResults {
    query: String,
    // We use Rc<str> instead of String here for two reasons:
    // - Rc for cheap clones since elements in `matches` will always be in `tasks` as well
    // - Rc<str> implements Borrow<str> meaning we can query a `HashSet<Rc<str>>` using a `&str`
    // We do not modify the provided task names so we do not need the capabilities of String.
    tasks: Vec<Arc<str>>,
    matches: HashSet<Arc<str>>,
}

impl SearchResults {
    pub fn new(tasks: &TasksByStatus) -> Self {
        Self {
            tasks: tasks
                .task_names_in_displayed_order()
                .map(Arc::from)
                .collect(),
            query: String::new(),
            matches: HashSet::new(),
        }
    }

    /// Updates search results with new search body
    pub fn update_tasks(&mut self, tasks: &TasksByStatus) {
        self.tasks.clear();
        self.tasks
            .extend(tasks.task_names_in_displayed_order().map(Arc::from));
        self.update_matches();
    }

    /// Updates the query and the matches
    pub fn modify_query(&mut self, modification: impl FnOnce(&mut String)) {
        modification(&mut self.query);
        self.update_matches();
    }

    fn update_matches(&mut self) {
        self.matches.clear();
        if self.query.is_empty() {
            return;
        }
        for task in self.tasks.iter().filter(|task| task.contains(&self.query)) {
            self.matches.insert(task.clone());
        }
    }

    /// Given an iterator it returns the first task that is in the search
    /// results
    pub fn first_match<'a>(&self, mut tasks: impl Iterator<Item = &'a str>) -> Option<&'a str> {
        tasks.find(|task| self.matches.contains(*task))
    }

    /// Returns if there are any matches for the query
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// Returns query
    pub fn query(&self) -> &str {
        &self.query
    }
}

---
File: /crates/turborepo-ui/src/tui/size.rs
---

use crate::TaskTable;

const PANE_SIZE_RATIO: f32 = 3.0 / 4.0;

#[derive(Debug, Clone, Copy)]
pub struct SizeInfo {
    task_width_hint: u16,
    rows: u16,
    cols: u16,
}

impl SizeInfo {
    pub fn new<'a>(rows: u16, cols: u16, tasks: impl Iterator<Item = &'a str>) -> Self {
        let task_width_hint = TaskTable::width_hint(tasks);
        Self {
            rows,
            cols,
            task_width_hint,
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.rows = rows;
        self.cols = cols;
    }

    pub fn pane_rows(&self) -> u16 {
        self.rows
            // Account for header and footer in layout
            .saturating_sub(2)
            // Always allocate at least one row as vt100 crashes if emulating a zero area terminal
            .max(1)
    }

    pub fn task_list_width(&self) -> u16 {
        self.cols - self.pane_cols()
    }

    pub fn pane_cols(&self) -> u16 {
        // Want to maximize pane width
        let ratio_pane_width = (f32::from(self.cols) * PANE_SIZE_RATIO) as u16;
        let full_task_width = self.cols.saturating_sub(self.task_width_hint);
        full_task_width
            .max(ratio_pane_width)
            // We need to account for the left border of the pane
            .saturating_sub(1)
    }
}



---
File: /crates/turborepo-ui/src/tui/spinner.rs
---

use std::time::{Duration, Instant};

const SPINNER_FRAMES: &[&str] = ["»"].as_slice();
// const SPINNER_FRAMES: &[&str] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇",
// "⠏"].as_slice();
const FRAMERATE: Duration = Duration::from_millis(80);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpinnerState {
    frame: usize,
    last_render: Option<Instant>,
}

impl SpinnerState {
    pub fn new() -> Self {
        Self {
            frame: 0,
            last_render: None,
        }
    }

    pub fn update(&mut self) {
        if let Some(last_render) = self.last_render {
            if last_render.elapsed() > FRAMERATE {
                self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
                self.last_render = Some(Instant::now());
            }
        } else {
            self.last_render = Some(Instant::now());
        }
    }

    pub fn current(&self) -> &'static str {
        SPINNER_FRAMES[self.frame]
    }
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self::new()
    }
}

---
File: /crates/turborepo-ui/src/tui/state.rs
---

use std::time::Instant;

enum Event {
    Tick,
}

struct State {
    current_time: Instant,
}

struct Focus {
    task_id: String,
    focus_type: FocusType,
}

enum FocusType {
    View,
    Interact,
}

struct DoneTask {
    id: String,
    start: Instant,
    end: Instant,
}



---
File: /crates/turborepo-ui/src/tui/table.rs
---

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState},
};

use super::{event::TaskResult, spinner::SpinnerState, task::TasksByStatus};

/// A widget that renders a table of their tasks and their current status
///
/// The tasks are ordered as follows:
/// - running tasks
/// - planned tasks
/// - finished tasks
///   - failed tasks
///   - successful tasks
///   - cached tasks
pub struct TaskTable<'b> {
    tasks_by_type: &'b TasksByStatus,
    spinner: SpinnerState,
}

const TASK_NAVIGATE_INSTRUCTIONS: &str = "↑ ↓ - Select";
const MORE_BINDS_INSTRUCTIONS: &str = "m - More binds";

impl<'b> TaskTable<'b> {
    /// Construct a new table with all of the planned tasks
    pub fn new(tasks_by_type: &'b TasksByStatus) -> Self {
        Self {
            tasks_by_type,
            spinner: SpinnerState::default(),
        }
    }

    /// Provides a suggested width for the task table
    pub fn width_hint<'a>(tasks: impl Iterator<Item = &'a str>) -> u16 {
        let task_name_width = tasks
            .map(|task| task.len())
            .max()
            .unwrap_or_default()
            // Task column width should be large enough to fit "↑ ↓ to navigate instructions
            // and truncate tasks with more than 40 chars.
            .clamp(TASK_NAVIGATE_INSTRUCTIONS.len(), 40) as u16;
        // Add space for column divider and status emoji
        task_name_width + 1
    }

    /// Update the current time of the table
    pub fn tick(&mut self) {
        self.spinner.update();
    }

    fn finished_rows(&self) -> impl Iterator<Item = Row> + '_ {
        self.tasks_by_type.finished.iter().map(move |task| {
            let name = if matches!(task.result(), TaskResult::CacheHit) {
                Cell::new(Text::styled(task.name(), Style::default().italic()))
            } else {
                Cell::new(task.name())
            };

            Row::new(vec![
                name,
                match task.result() {
                    // matches Next.js (and many other CLI tools) https://github.com/vercel/next.js/blob/1a04d94aaec943d3cce93487fea3b8c8f8898f31/packages/next/src/build/output/log.ts
                    TaskResult::Success => {
                        Cell::new(Text::styled("✓", Style::default().green().bold()))
                    }
                    TaskResult::CacheHit => {
                        Cell::new(Text::styled("⊙", Style::default().magenta()))
                    }
                    TaskResult::Failure => {
                        Cell::new(Text::styled("⨯", Style::default().red().bold()))
                    }
                },
            ])
        })
    }

    fn running_rows(&self) -> impl Iterator<Item = Row> + '_ {
        let spinner = self.spinner.current();
        self.tasks_by_type
            .running
            .iter()
            .map(move |task| Row::new(vec![Cell::new(task.name()), Cell::new(Text::raw(spinner))]))
    }

    fn planned_rows(&self) -> impl Iterator<Item = Row> + '_ {
        self.tasks_by_type
            .planned
            .iter()
            .map(move |task| Row::new(vec![Cell::new(task.name()), Cell::new(" ")]))
    }
}

impl<'a> StatefulWidget for &'a TaskTable<'a> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let table = Table::new(
            self.running_rows()
                .chain(self.planned_rows())
                .chain(self.finished_rows()),
            [
                Constraint::Min(15),
                // Status takes one cell to render
                Constraint::Length(1),
            ],
        )
        .highlight_style(Style::default().fg(Color::Yellow))
        .column_spacing(0)
        .block(Block::new().borders(Borders::RIGHT))
        .header(
            vec![Text::styled(
                "Tasks",
                Style::default().add_modifier(Modifier::DIM),
            )]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1),
        )
        .footer(
            vec![Text::styled(
                format!("{TASK_NAVIGATE_INSTRUCTIONS}\n{MORE_BINDS_INSTRUCTIONS}"),
                Style::default().add_modifier(Modifier::DIM),
            )]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(2),
        );
        StatefulWidget::render(table, area, buf, state);
    }
}



---
File: /crates/turborepo-ui/src/tui/task.rs
---

#![allow(dead_code)]
use std::{collections::HashSet, mem, time::Instant};

use super::{event::TaskResult, Error};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Planned;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Running {
    start: Instant,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Finished {
    start: Instant,
    end: Instant,
    result: TaskResult,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Task<S> {
    name: String,
    state: S,
}

pub enum TaskType {
    Planned,
    Running,
    Finished,
}

impl<S> Task<S> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Task<Planned> {
    pub fn new(name: String) -> Task<Planned> {
        Task {
            name,
            state: Planned,
        }
    }

    pub fn start(self) -> Task<Running> {
        Task {
            name: self.name,
            state: Running {
                start: Instant::now(),
            },
        }
    }
}

impl Task<Running> {
    pub fn finish(self, result: TaskResult) -> Task<Finished> {
        let Task {
            name,
            state: Running { start },
        } = self;
        Task {
            name,
            state: Finished {
                start,
                result,
                end: Instant::now(),
            },
        }
    }

    pub fn start(&self) -> Instant {
        self.state.start
    }

    pub fn restart(self) -> Task<Planned> {
        Task {
            name: self.name,
            state: Planned,
        }
    }
}

impl Task<Finished> {
    pub fn start(&self) -> Instant {
        self.state.start
    }

    pub fn end(&self) -> Instant {
        self.state.end
    }

    pub fn result(&self) -> TaskResult {
        self.state.result
    }

    pub fn restart(self) -> Task<Planned> {
        Task {
            name: self.name,
            state: Planned,
        }
    }
}

#[derive(Default)]
pub struct TaskNamesByStatus {
    pub running: Vec<String>,
    pub planned: Vec<String>,
    pub finished: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TasksByStatus {
    pub running: Vec<Task<Running>>,
    pub planned: Vec<Task<Planned>>,
    pub finished: Vec<Task<Finished>>,
}

impl TasksByStatus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all_empty(&self) -> bool {
        self.planned.is_empty() && self.finished.is_empty() && self.running.is_empty()
    }

    pub fn count_all(&self) -> usize {
        self.task_names_in_displayed_order().count()
    }

    pub fn task_names_in_displayed_order(&self) -> impl DoubleEndedIterator<Item = &str> + '_ {
        let running_names = self.running.iter().map(|task| task.name());
        let planned_names = self.planned.iter().map(|task| task.name());
        let finished_names = self.finished.iter().map(|task| task.name());

        running_names.chain(planned_names).chain(finished_names)
    }

    pub fn active_index(&self, task_name: &str) -> Option<usize> {
        self.task_names_in_displayed_order()
            .position(|task| task == task_name)
    }

    pub fn task_name(&self, index: usize) -> Result<&str, Error> {
        self.task_names_in_displayed_order()
            .nth(index)
            .ok_or_else(|| Error::TaskNotFoundIndex {
                index,
                len: self.count_all(),
            })
    }

    pub fn tasks_started(&self) -> Vec<String> {
        let (errors, success): (Vec<_>, Vec<_>) = self
            .finished
            .iter()
            .partition(|task| matches!(task.result(), TaskResult::Failure));

        // We return errors last as they most likely have information users want to see
        success
            .into_iter()
            .map(|task| task.name())
            .chain(self.running.iter().map(|task| task.name()))
            .chain(errors.into_iter().map(|task| task.name()))
            .map(|task| task.to_string())
            .collect()
    }

    pub fn restart_tasks<'a>(&mut self, tasks: impl Iterator<Item = &'a str>) {
        let mut tasks_to_restart = tasks.collect::<HashSet<_>>();

        let (restarted_running, keep_running): (Vec<_>, Vec<_>) = mem::take(&mut self.running)
            .into_iter()
            .partition(|task| tasks_to_restart.contains(task.name()));
        self.running = keep_running;

        let (restarted_finished, keep_finished): (Vec<_>, Vec<_>) = mem::take(&mut self.finished)
            .into_iter()
            .partition(|task| tasks_to_restart.contains(task.name()));
        self.finished = keep_finished;
        self.planned.extend(
            restarted_running
                .into_iter()
                .map(|task| task.restart())
                .chain(restarted_finished.into_iter().map(|task| task.restart())),
        );
        // There is a chance that watch might attempt to restart a task that did not
        // exist before.
        for task in &self.planned {
            tasks_to_restart.remove(task.name());
        }
        self.planned.extend(
            tasks_to_restart
                .into_iter()
                .map(ToOwned::to_owned)
                .map(Task::new),
        );
        self.planned.sort_unstable();
    }

    /// Insert a finished task into the correct place in the finished section.
    /// The order of `finished` is expected to be: failure, success, cached
    /// with each subsection being sorted by finish time.
    /// Returns the index task was inserted at
    pub fn insert_finished_task(&mut self, task: Task<Finished>) -> usize {
        let index = match task.result() {
            TaskResult::Failure => self
                .finished
                .iter()
                .enumerate()
                .skip_while(|(_, task)| task.result() == TaskResult::Failure)
                .map(|(idx, _)| idx)
                .next(),
            TaskResult::Success => self
                .finished
                .iter()
                .enumerate()
                .skip_while(|(_, task)| {
                    task.result() == TaskResult::Failure || task.result() == TaskResult::Success
                })
                .map(|(idx, _)| idx)
                .next(),
            TaskResult::CacheHit => None,
        }
        .unwrap_or(self.finished.len());
        self.finished.insert(index, task);
        index
    }
}


---
File: /crates/turborepo-ui/src/tui/term_output.rs
---

use std::{io::Write, mem};

use turborepo_vt100 as vt100;

use super::{
    event::{CacheResult, Direction, OutputLogs, TaskResult},
    Error,
};

const SCROLLBACK_LEN: usize = 1024;

pub struct TerminalOutput<W> {
    output: Vec<u8>,
    pub parser: vt100::Parser,
    pub stdin: Option<W>,
    pub status: Option<String>,
    pub output_logs: Option<OutputLogs>,
    pub task_result: Option<TaskResult>,
    pub cache_result: Option<CacheResult>,
}

#[derive(Debug, Clone, Copy)]
enum LogBehavior {
    Full,
    Status,
    Nothing,
}

impl<W> TerminalOutput<W> {
    pub fn new(rows: u16, cols: u16, stdin: Option<W>) -> Self {
        Self {
            output: Vec::new(),
            parser: vt100::Parser::new(rows, cols, SCROLLBACK_LEN),
            stdin,
            status: None,
            output_logs: None,
            task_result: None,
            cache_result: None,
        }
    }

    pub fn title(&self, task_name: &str) -> String {
        match self.status.as_deref() {
            Some(status) => format!(" {task_name} > {status} "),
            None => format!(" {task_name} > "),
        }
    }

    pub fn size(&self) -> (u16, u16) {
        self.parser.screen().size()
    }

    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
        self.output.extend_from_slice(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        if self.parser.screen().size() != (rows, cols) {
            let scrollback = self.parser.screen().scrollback();
            let mut new_parser = vt100::Parser::new(rows, cols, SCROLLBACK_LEN);
            new_parser.process(&self.output);
            new_parser.screen_mut().set_scrollback(scrollback);
            // Completely swap out the old vterm with a new correctly sized one
            mem::swap(&mut self.parser, &mut new_parser);
        }
    }

    pub fn scroll(&mut self, direction: Direction) -> Result<(), Error> {
        let scrollback = self.parser.screen().scrollback();
        let new_scrollback = match direction {
            Direction::Up => scrollback + 1,
            Direction::Down => scrollback.saturating_sub(1),
        };
        self.parser.screen_mut().set_scrollback(new_scrollback);
        Ok(())
    }

    fn persist_behavior(&self) -> LogBehavior {
        match self.output_logs.unwrap_or(OutputLogs::Full) {
            OutputLogs::Full => LogBehavior::Full,
            OutputLogs::None => LogBehavior::Nothing,
            OutputLogs::HashOnly => LogBehavior::Status,
            OutputLogs::NewOnly => {
                if matches!(self.cache_result, Some(super::event::CacheResult::Miss),) {
                    LogBehavior::Full
                } else {
                    LogBehavior::Status
                }
            }
            OutputLogs::ErrorsOnly => {
                if matches!(self.task_result, Some(TaskResult::Failure)) {
                    LogBehavior::Full
                } else {
                    LogBehavior::Nothing
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn persist_screen(&self, task_name: &str) -> std::io::Result<()> {
        let mut stdout = std::io::stdout().lock();
        let title = self.title(task_name);
        match self.persist_behavior() {
            LogBehavior::Full => {
                let screen = self.parser.entire_screen();
                let (_, cols) = screen.size();
                stdout.write_all("┌".as_bytes())?;
                stdout.write_all(title.as_bytes())?;
                stdout.write_all(b"\r\n")?;
                for row in screen.rows_formatted(0, cols) {
                    stdout.write_all("│ ".as_bytes())?;
                    stdout.write_all(&row)?;
                    stdout.write_all(b"\r\n")?;
                }
                stdout.write_all("└────>\r\n".as_bytes())?;
            }
            LogBehavior::Status => {
                stdout.write_all(title.as_bytes())?;
                stdout.write_all(b"\r\n")?;
            }
            LogBehavior::Nothing => (),
        }
        Ok(())
    }

    pub fn has_selection(&self) -> bool {
        self.parser
            .screen()
            .selected_text()
            .is_some_and(|s| !s.is_empty())
    }

    pub fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) -> Result<(), Error> {
        match event.kind {
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // We need to update the vterm so we don't continue to render the selection
                self.parser.screen_mut().clear_selection();
            }
            crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                // Update selection of underlying parser
                self.parser
                    .screen_mut()
                    .update_selection(event.row, event.column);
            }
            // Scrolling is handled elsewhere
            crossterm::event::MouseEventKind::ScrollDown => (),
            crossterm::event::MouseEventKind::ScrollUp => (),
            // I think we can ignore this?
            crossterm::event::MouseEventKind::Moved => (),
            // Don't care about other mouse buttons
            crossterm::event::MouseEventKind::Down(_) => (),
            crossterm::event::MouseEventKind::Drag(_) => (),
            // We don't support horizontal scroll
            crossterm::event::MouseEventKind::ScrollLeft
            | crossterm::event::MouseEventKind::ScrollRight => (),
            // Cool, person stopped holding down mouse
            crossterm::event::MouseEventKind::Up(_) => (),
        }
        Ok(())
    }

    pub fn copy_selection(&self) -> Option<String> {
        self.parser.screen().selected_text()
    }
}



---
File: /crates/turborepo-ui/src/wui/event.rs
---

use serde::Serialize;

use crate::tui::event::{CacheResult, OutputLogs, TaskResult};

/// Specific events that the GraphQL server can send to the client,
/// not all the `Event` types from the TUI.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebUIEvent {
    StartTask {
        task: String,
        output_logs: OutputLogs,
    },
    TaskOutput {
        task: String,
        output: Vec<u8>,
    },
    EndTask {
        task: String,
        result: TaskResult,
    },
    CacheStatus {
        task: String,
        message: String,
        result: CacheResult,
    },
    UpdateTasks {
        tasks: Vec<String>,
    },
    RestartTasks {
        tasks: Vec<String>,
    },
    Stop,
}



---
File: /crates/turborepo-ui/src/wui/mod.rs
---

//! Web UI for Turborepo. Creates a WebSocket server that can be subscribed to
//! by a web client to display the status of tasks.

pub mod event;
pub mod query;
pub mod sender;
pub mod subscriber;

use event::WebUIEvent;
pub use query::RunQuery;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to start server.")]
    Server(#[from] std::io::Error),
    #[error("Failed to start websocket server: {0}")]
    WebSocket(#[source] axum::Error),
    #[error("Failed to serialize message: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Failed to send message.")]
    Send(#[from] axum::Error),
    #[error("Failed to send message through channel.")]
    Broadcast(#[from] tokio::sync::mpsc::error::SendError<WebUIEvent>),
}



---
File: /crates/turborepo-ui/src/wui/query.rs
---

use std::sync::Arc;

use async_graphql::{Object, SimpleObject};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::wui::subscriber::{TaskState, WebUIState};

#[derive(Debug, Clone, Serialize, SimpleObject)]
struct RunTask {
    name: String,
    state: TaskState,
}

struct CurrentRun<'a> {
    state: &'a SharedState,
}

#[Object]
impl CurrentRun<'_> {
    async fn tasks(&self) -> Vec<RunTask> {
        self.state
            .lock()
            .await
            .tasks()
            .iter()
            .map(|(task, state)| RunTask {
                name: task.clone(),
                state: state.clone(),
            })
            .collect()
    }
}

/// We keep the state in a `Arc<Mutex<RefCell<T>>>` so both `Subscriber` and
/// `Query` can access it, with `Subscriber` mutating it and `Query` only
/// reading it.
pub type SharedState = Arc<Mutex<WebUIState>>;

/// The query for actively running tasks.
///
/// (As opposed to the query for general repository state `RepositoryQuery`
/// in `turborepo_lib::query`)
/// This is `None` when we're not actually running a task (e.g. `turbo query`)
pub struct RunQuery {
    state: Option<SharedState>,
}

impl RunQuery {
    pub fn new(state: Option<SharedState>) -> Self {
        Self { state }
    }
}

#[Object]
impl RunQuery {
    async fn current_run(&self) -> Option<CurrentRun> {
        Some(CurrentRun {
            state: self.state.as_ref()?,
        })
    }
}



---
File: /crates/turborepo-ui/src/wui/sender.rs
---

use std::io::Write;

use tracing::log::warn;

use crate::{
    sender::{TaskSender, UISender},
    tui::event::{CacheResult, OutputLogs, TaskResult},
    wui::{event::WebUIEvent, Error},
};

#[derive(Debug, Clone)]
pub struct WebUISender {
    pub tx: tokio::sync::mpsc::UnboundedSender<WebUIEvent>,
}

impl WebUISender {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<WebUIEvent>) -> Self {
        Self { tx }
    }
    pub fn start_task(&self, task: String, output_logs: OutputLogs) {
        self.tx
            .send(WebUIEvent::StartTask { task, output_logs })
            .ok();
    }

    pub fn restart_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        self.tx
            .send(WebUIEvent::RestartTasks { tasks })
            .map_err(Error::Broadcast)?;
        Ok(())
    }

    pub fn end_task(&self, task: String, result: TaskResult) {
        self.tx.send(WebUIEvent::EndTask { task, result }).ok();
    }

    pub fn status(&self, task: String, message: String, result: CacheResult) {
        self.tx
            .send(WebUIEvent::CacheStatus {
                task,
                message,
                result,
            })
            .ok();
    }

    pub fn set_stdin(&self, _: String, _: Box<dyn Write + Send>) {
        warn!("stdin is not supported (yet) in web ui");
    }

    pub fn task(&self, task: String) -> TaskSender {
        TaskSender {
            name: task,
            handle: UISender::Wui(self.clone()),
            logs: Default::default(),
        }
    }

    pub fn stop(&self) {
        self.tx.send(WebUIEvent::Stop).ok();
    }

    pub fn update_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        self.tx
            .send(WebUIEvent::UpdateTasks { tasks })
            .map_err(Error::Broadcast)?;

        Ok(())
    }

    pub fn output(&self, task: String, output: Vec<u8>) -> Result<(), crate::Error> {
        self.tx
            .send(WebUIEvent::TaskOutput { task, output })
            .map_err(Error::Broadcast)?;

        Ok(())
    }
}



---
File: /crates/turborepo-ui/src/wui/subscriber.rs
---

use std::{collections::BTreeMap, sync::Arc};

use async_graphql::{Enum, SimpleObject};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    tui::event::{CacheResult, TaskResult},
    wui::{event::WebUIEvent, query::SharedState},
};

/// Subscribes to the Web UI events and updates the state
pub struct Subscriber {
    rx: tokio::sync::mpsc::UnboundedReceiver<WebUIEvent>,
}

impl Subscriber {
    pub fn new(rx: tokio::sync::mpsc::UnboundedReceiver<WebUIEvent>) -> Self {
        Self { rx }
    }

    pub async fn watch(
        self,
        // We use a tokio::sync::Mutex here because we want this future to be Send.
        #[allow(clippy::type_complexity)] state: SharedState,
    ) {
        let mut rx = self.rx;
        while let Some(event) = rx.recv().await {
            Self::add_message(&state, event).await;
        }
    }

    async fn add_message(state: &Arc<Mutex<WebUIState>>, event: WebUIEvent) {
        let mut state = state.lock().await;

        match event {
            WebUIEvent::StartTask {
                task,
                output_logs: _,
            } => {
                state.tasks.insert(
                    task,
                    TaskState {
                        output: Vec::new(),
                        status: TaskStatus::Running,
                        cache_result: None,
                        cache_message: None,
                    },
                );
            }
            WebUIEvent::TaskOutput { task, output } => {
                state.tasks.get_mut(&task).unwrap().output.extend(output);
            }
            WebUIEvent::EndTask { task, result } => {
                state.tasks.get_mut(&task).unwrap().status = TaskStatus::from(result);
            }
            WebUIEvent::CacheStatus {
                task,
                result,
                message,
            } => {
                if result == CacheResult::Hit {
                    state.tasks.get_mut(&task).unwrap().status = TaskStatus::Cached;
                }
                state.tasks.get_mut(&task).unwrap().cache_result = Some(result);
                state.tasks.get_mut(&task).unwrap().cache_message = Some(message);
            }
            WebUIEvent::Stop => {
                // TODO: stop watching
            }
            WebUIEvent::UpdateTasks { tasks } => {
                state.tasks = tasks
                    .into_iter()
                    .map(|task| {
                        (
                            task,
                            TaskState {
                                output: Vec::new(),
                                status: TaskStatus::Pending,
                                cache_result: None,
                                cache_message: None,
                            },
                        )
                    })
                    .collect();
            }
            WebUIEvent::RestartTasks { tasks } => {
                state.tasks = tasks
                    .into_iter()
                    .map(|task| {
                        (
                            task,
                            TaskState {
                                output: Vec::new(),
                                status: TaskStatus::Running,
                                cache_result: None,
                                cache_message: None,
                            },
                        )
                    })
                    .collect();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Enum)]
pub enum TaskStatus {
    Pending,
    Running,
    Cached,
    Failed,
    Succeeded,
}

impl From<TaskResult> for TaskStatus {
    fn from(result: TaskResult) -> Self {
        match result {
            TaskResult::Success => Self::Succeeded,
            TaskResult::CacheHit => Self::Cached,
            TaskResult::Failure => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct TaskState {
    output: Vec<u8>,
    status: TaskStatus,
    cache_result: Option<CacheResult>,
    /// The message for the cache status, i.e. `cache hit, replaying logs`
    cache_message: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct WebUIState {
    tasks: BTreeMap<String, TaskState>,
}

impl WebUIState {
    pub fn tasks(&self) -> &BTreeMap<String, TaskState> {
        &self.tasks
    }
}


---
File: /crates/turborepo-ui/src/color_selector.rs
---

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use console::{Style, StyledObject};

static COLORS: OnceLock<[Style; 5]> = OnceLock::new();

pub fn get_terminal_package_colors() -> &'static [Style; 5] {
    COLORS.get_or_init(|| {
        [
            Style::new().cyan(),
            Style::new().magenta(),
            Style::new().green(),
            Style::new().yellow(),
            Style::new().blue(),
        ]
    })
}

/// Selects colors for tasks and caches accordingly.
/// Shared between tasks so allows for concurrent access.
#[derive(Default)]
pub struct ColorSelector {
    inner: Arc<RwLock<ColorSelectorInner>>,
}

#[derive(Default)]
struct ColorSelectorInner {
    idx: usize,
    cache: HashMap<String, &'static Style>,
}

impl ColorSelector {
    pub fn color_for_key(&self, key: &str) -> &'static Style {
        if let Some(style) = self.inner.read().expect("lock poisoned").color(key) {
            return style;
        }

        let color = {
            self.inner
                .write()
                .expect("lock poisoned")
                .insert_color(key.to_string())
        };

        color
    }

    pub fn prefix_with_color(&self, cache_key: &str, prefix: &str) -> StyledObject<String> {
        if prefix.is_empty() {
            return Style::new().apply_to(String::new());
        }

        let style = self.color_for_key(cache_key);
        style.apply_to(format!("{}: ", prefix))
    }
}

impl ColorSelectorInner {
    fn color(&self, key: &str) -> Option<&'static Style> {
        self.cache.get(key).copied()
    }

    fn insert_color(&mut self, key: String) -> &'static Style {
        let colors = get_terminal_package_colors();
        let chosen_color = &colors[self.idx % colors.len()];
        // A color might have been chosen by the time we get to inserting
        self.cache.entry(key).or_insert_with(|| {
            // If a color hasn't been chosen, then we increment the index
            self.idx += 1;
            chosen_color
        })
    }
}

---
File: /crates/turborepo-ui/src/lib.rs
---

//! Turborepo's terminal UI library. Handles elements like spinners, colors,
//! and logging. Includes a `PrefixedUI` struct that can be used to prefix
//! output, and a `ColorSelector` that lets multiple concurrent resources get
//! an assigned color.
#![feature(deadline_api)]

mod color_selector;
mod line;
mod logs;
mod output;
mod prefixed;
pub mod sender;
pub mod tui;
pub mod wui;

use std::{borrow::Cow, env, f64::consts::PI, time::Duration};

use console::{Style, StyledObject};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use thiserror::Error;

pub use crate::{
    color_selector::ColorSelector,
    line::LineWriter,
    logs::{replay_logs, replay_logs_with_crlf, LogWriter},
    output::{OutputClient, OutputClientBehavior, OutputSink, OutputWriter},
    prefixed::{PrefixedUI, PrefixedWriter},
    tui::{TaskTable, TerminalPane},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Tui(#[from] tui::Error),
    #[error(transparent)]
    Wui(#[from] wui::Error),
    #[error("Cannot read logs: {0}")]
    CannotReadLogs(#[source] std::io::Error),
    #[error("Cannot write logs: {0}")]
    CannotWriteLogs(#[source] std::io::Error),
}

pub fn start_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    if env::var("CI").is_ok() {
        pb.enable_steady_tick(Duration::from_secs(30));
    } else {
        pb.enable_steady_tick(Duration::from_millis(125));
    }
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "   ",
                GREY.apply_to(">  ").to_string().as_str(),
                GREY.apply_to(">> ").to_string().as_str(),
                GREY.apply_to(">>>").to_string().as_str(),
                ">>>",
            ]),
    );
    pb.set_message(message.to_string());

    pb
}

#[macro_export]
macro_rules! color {
    ($ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        $ui.apply(colored_str)
    }};
}

#[macro_export]
macro_rules! cprintln {
    ($ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        println!("{}", $ui.apply(colored_str))
    }};
}

#[macro_export]
macro_rules! cprint {
    ($ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        print!("{}", $ui.apply(colored_str))
    }};
}

#[macro_export]
macro_rules! cwrite {
    ($dst:expr, $ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        write!($dst, "{}", $ui.apply(colored_str))
    }};
}

#[macro_export]
macro_rules! cwriteln {
    ($writer:expr, $ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        writeln!($writer, "{}", $ui.apply(colored_str))
    }};
}

#[macro_export]
macro_rules! ceprintln {
    ($ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        eprintln!("{}", $ui.apply(colored_str))
    }};
}

#[macro_export]
macro_rules! ceprint {
    ($ui:expr, $color:expr, $format_string:expr $(, $arg:expr)*) => {{
        let formatted_str = format!($format_string $(, $arg)*);

        let colored_str = $color.apply_to(formatted_str);

        eprint!("{}", $ui.apply(colored_str))
    }};
}

/// Helper struct to apply any necessary formatting to UI output
#[derive(Debug, Clone, Copy)]
pub struct ColorConfig {
    pub should_strip_ansi: bool,
}

impl ColorConfig {
    pub fn new(should_strip_ansi: bool) -> Self {
        Self { should_strip_ansi }
    }

    /// Infer the color choice from environment variables and checking if stdout
    /// is a tty
    pub fn infer() -> Self {
        let env_setting =
            std::env::var("FORCE_COLOR")
                .ok()
                .and_then(|force_color| match force_color.as_str() {
                    "false" | "0" => Some(true),
                    "true" | "1" | "2" | "3" => Some(false),
                    _ => None,
                });
        let should_strip_ansi = env_setting.unwrap_or_else(|| !atty::is(atty::Stream::Stdout));
        Self { should_strip_ansi }
    }

    /// Apply the UI color mode to the given styled object
    ///
    /// This is required to match the Go turborepo coloring logic which differs
    /// from console's coloring detection.
    pub fn apply<D>(&self, obj: StyledObject<D>) -> StyledObject<D> {
        // Setting this to false will skip emitting any ansi codes associated
        // with the style when the object is displayed.
        obj.force_styling(!self.should_strip_ansi)
    }

    // Ported from Go code. Converts an index to a color along the rainbow
    fn rainbow_rgb(i: usize) -> (u8, u8, u8) {
        let f = 0.275;
        let r = (f * i as f64 + 4.0 * PI / 3.0).sin() * 127.0 + 128.0;
        let g = 45.0;
        let b = (f * i as f64).sin() * 127.0 + 128.0;

        (r as u8, g as u8, b as u8)
    }

    pub fn rainbow<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if self.should_strip_ansi {
            return Cow::Borrowed(text);
        }

        // On the macOS Terminal, the rainbow colors don't show up correctly.
        // Instead, we print in bold magenta
        if matches!(env::var("TERM_PROGRAM"), Ok(terminal_program) if terminal_program == "Apple_Terminal")
        {
            return BOLD.apply_to(MAGENTA.apply_to(text)).to_string().into();
        }

        let mut out = Vec::new();
        for (i, c) in text.char_indices() {
            let (r, g, b) = Self::rainbow_rgb(i);
            out.push(format!(
                "\x1b[1m\x1b[38;2;{};{};{}m{}\x1b[0m\x1b[0;1m",
                r, g, b, c
            ));
        }
        out.push(RESET.to_string());

        Cow::Owned(out.join(""))
    }
}

lazy_static! {
    pub static ref GREY: Style = Style::new().dim();
    pub static ref CYAN: Style = Style::new().cyan();
    pub static ref BOLD: Style = Style::new().bold();
    pub static ref MAGENTA: Style = Style::new().magenta();
    pub static ref YELLOW: Style = Style::new().yellow();
    pub static ref BOLD_YELLOW_REVERSE: Style = Style::new().yellow().bold().reverse();
    pub static ref UNDERLINE: Style = Style::new().underlined();
    pub static ref BOLD_CYAN: Style = Style::new().cyan().bold();
    pub static ref BOLD_GREY: Style = Style::new().dim().bold();
    pub static ref BOLD_GREEN: Style = Style::new().green().bold();
    pub static ref BOLD_RED: Style = Style::new().red().bold();
}

pub const RESET: &str = "\x1b[0m";

pub use dialoguer::theme::ColorfulTheme as DialoguerTheme;


---
File: /crates/turborepo-ui/src/line.rs
---

use std::io::Write;

/// Writer that will buffer writes so the underlying writer is only called with
/// writes that end in a newline
pub struct LineWriter<W> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> LineWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(512),
        }
    }
}

impl<W: Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for line in buf.split_inclusive(|c| *c == b'\n') {
            if line.ends_with(b"\n") {
                if self.buffer.is_empty() {
                    self.writer.write_all(line)?;
                } else {
                    self.buffer.extend_from_slice(line);
                    self.writer.write_all(&self.buffer)?;
                    self.buffer.clear();
                }
            } else {
                // This should only happen on the last chunk?
                self.buffer.extend_from_slice(line)
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // We don't flush our buffer as that would lead to a write without a newline
        self.writer.flush()
    }
}



---
File: /crates/turborepo-ui/src/logs.rs
---

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use tracing::{debug, warn};
use turbopath::AbsoluteSystemPath;

use crate::Error;

/// Receives logs and multiplexes them to a log file and/or a prefixed
/// writer
pub struct LogWriter<W> {
    log_file: Option<BufWriter<File>>,
    writer: Option<W>,
}

/// Derive didn't work here.
/// (we don't actually need `W` to implement `Default` here)
impl<W> Default for LogWriter<W> {
    fn default() -> Self {
        Self {
            log_file: None,
            writer: None,
        }
    }
}

impl<W: Write> LogWriter<W> {
    pub fn with_log_file(&mut self, log_file_path: &AbsoluteSystemPath) -> Result<(), Error> {
        log_file_path.ensure_dir().map_err(|err| {
            warn!("error creating log file directory: {:?}", err);
            Error::CannotWriteLogs(err)
        })?;

        let log_file = log_file_path.create().map_err(|err| {
            warn!("error creating log file: {:?}", err);
            Error::CannotWriteLogs(err)
        })?;

        self.log_file = Some(BufWriter::new(log_file));

        Ok(())
    }

    pub fn with_writer(&mut self, writer: W) {
        self.writer = Some(writer);
    }
}

impl<W: Write> Write for LogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match (&mut self.log_file, &mut self.writer) {
            (Some(log_file), Some(prefixed_writer)) => {
                let _ = prefixed_writer.write(buf)?;
                log_file.write(buf)
            }
            (Some(log_file), None) => log_file.write(buf),
            (None, Some(prefixed_writer)) => prefixed_writer.write(buf),
            (None, None) => {
                debug!(
                    "No log file or prefixed writer to write to. This should only happen when \
                     both caching is disabled and output logs are set to none."
                );

                // Returning the buffer's length so callers don't think this is a failure to
                // create the buffer
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(log_file) = &mut self.log_file {
            log_file.flush()?;
        }
        if let Some(prefixed_writer) = &mut self.writer {
            prefixed_writer.flush()?;
        }

        Ok(())
    }
}

pub fn replay_logs<W: Write>(
    mut output: W,
    log_file_name: &AbsoluteSystemPath,
) -> Result<(), Error> {
    debug!("start replaying logs");

    let log_file = File::open(log_file_name).map_err(|err| {
        warn!("error opening log file: {:?}", err);
        Error::CannotReadLogs(err)
    })?;

    let mut log_reader = BufReader::new(log_file);

    let mut buffer = Vec::new();
    loop {
        let num_bytes = log_reader
            .read_until(b'\n', &mut buffer)
            .map_err(Error::CannotReadLogs)?;
        if num_bytes == 0 {
            break;
        }

        // If the log file doesn't end with a newline, then we add one to ensure the
        // underlying writer receives a full line.
        if !buffer.ends_with(b"\n") {
            buffer.push(b'\n');
        }
        output.write_all(&buffer).map_err(Error::CannotReadLogs)?;

        buffer.clear();
    }

    debug!("finish replaying logs");

    Ok(())
}

/// Replay logs, but enforce crlf line endings
// TODO: refactor to share code with `replay_logs`
pub fn replay_logs_with_crlf<W: Write>(
    mut output: W,
    log_file_name: &AbsoluteSystemPath,
) -> Result<(), Error> {
    debug!("start replaying logs");

    let log_file = File::open(log_file_name).map_err(|err| {
        warn!("error opening log file: {:?}", err);
        Error::CannotReadLogs(err)
    })?;

    let mut log_reader = BufReader::new(log_file);

    let mut buffer = Vec::new();
    loop {
        let num_bytes = log_reader
            .read_until(b'\n', &mut buffer)
            .map_err(Error::CannotReadLogs)?;
        if num_bytes == 0 {
            break;
        }

        let line_without_lf = buffer.strip_suffix(b"\n").unwrap_or(&buffer);
        let line_without_crlf = line_without_lf
            .strip_suffix(b"\r")
            .unwrap_or(line_without_lf);

        output
            .write_all(line_without_crlf)
            .map_err(Error::CannotReadLogs)?;
        output.write_all(b"\r\n").map_err(Error::CannotReadLogs)?;

        buffer.clear();
    }

    debug!("finish replaying logs");

    Ok(())
}

---
File: /crates/turborepo-ui/src/output.rs
---

use std::{
    borrow::Cow,
    io::{self, Write},
    sync::{Arc, Mutex, RwLock},
};

use turborepo_ci::GroupPrefixFn;

/// OutputSink represent a sink for outputs that can be written to from multiple
/// threads through the use of Loggers.
pub struct OutputSink<W> {
    writers: Arc<Mutex<SinkWriters<W>>>,
}

struct SinkWriters<W> {
    out: W,
    err: W,
}

/// OutputClient allows for multiple threads to write to the same OutputSink
pub struct OutputClient<W> {
    behavior: OutputClientBehavior,
    // We could use a RefCell if we didn't use this with async code.
    // Any locals held across an await must implement Sync and RwLock lets us achieve this
    buffer: Option<RwLock<Vec<SinkBytes<'static>>>>,
    writers: Arc<Mutex<SinkWriters<W>>>,
    primary: Marginals,
    error: Marginals,
}

#[derive(Default)]
struct Marginals {
    header: Option<GroupPrefixFn>,
    footer: Option<GroupPrefixFn>,
}

pub struct OutputWriter<'a, W> {
    logger: &'a OutputClient<W>,
    destination: Destination,
    buffer: Vec<u8>,
}

/// Enum for controlling the behavior of the client
#[derive(Debug, Clone, Copy)]
pub enum OutputClientBehavior {
    /// Every line sent to the client will get immediately sent to the sink
    Passthrough,
    /// Every line sent to the client will get immediately sent to the sink,
    /// but a buffer will be built up as well and returned when finish is called
    InMemoryBuffer,
    // Every line sent to the client will get tracked in the buffer only being
    // sent to the sink once finish is called.
    Grouped,
}

#[derive(Debug, Clone, Copy)]
enum Destination {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
struct SinkBytes<'a> {
    buffer: Cow<'a, [u8]>,
    destination: Destination,
}

impl<W: Write> OutputSink<W> {
    /// Produces a new sink with the corresponding out and err writers
    pub fn new(out: W, err: W) -> Self {
        Self {
            writers: Arc::new(Mutex::new(SinkWriters { out, err })),
        }
    }

    /// Produces a new client that will send all bytes that it receives to the
    /// underlying sink. Behavior of how these bytes are sent is controlled
    /// by the behavior parameter. Note that OutputClient intentionally doesn't
    /// implement Sync as if you want to write to the same sink
    /// from multiple threads, then you should create a logger for each thread.
    pub fn logger(&self, behavior: OutputClientBehavior) -> OutputClient<W> {
        let buffer = match behavior {
            OutputClientBehavior::Passthrough => None,
            OutputClientBehavior::InMemoryBuffer | OutputClientBehavior::Grouped => {
                Some(Default::default())
            }
        };
        let writers = self.writers.clone();
        OutputClient {
            behavior,
            buffer,
            writers,
            primary: Default::default(),
            error: Default::default(),
        }
    }
}

impl<W: Write> OutputClient<W> {
    pub fn with_header_footer(
        &mut self,
        header: Option<GroupPrefixFn>,
        footer: Option<GroupPrefixFn>,
    ) {
        self.primary = Marginals { header, footer };
    }

    pub fn with_error_header_footer(
        &mut self,
        header: Option<GroupPrefixFn>,
        footer: Option<GroupPrefixFn>,
    ) {
        self.error = Marginals { header, footer };
    }

    /// A writer that will write to the underlying sink's out writer according
    /// to this client's behavior.
    pub fn stdout(&self) -> OutputWriter<W> {
        OutputWriter {
            logger: self,
            destination: Destination::Stdout,
            buffer: Vec::new(),
        }
    }

    /// A writer that will write to the underlying sink's err writer according
    /// to this client's behavior.
    pub fn stderr(&self) -> OutputWriter<W> {
        OutputWriter {
            logger: self,
            destination: Destination::Stderr,
            buffer: Vec::new(),
        }
    }

    /// Consume the client and flush any bytes to the underlying sink if
    /// necessary
    pub fn finish(self, use_error: bool) -> io::Result<Option<Vec<u8>>> {
        let Self {
            behavior,
            buffer,
            writers,
            primary,
            error,
        } = self;
        let buffers = buffer.map(|cell| cell.into_inner().expect("lock poisoned"));
        let header = use_error
            .then_some(error.header)
            .flatten()
            .or(primary.header);
        let footer = use_error
            .then_some(error.footer)
            .flatten()
            .or(primary.footer);

        if matches!(behavior, OutputClientBehavior::Grouped) {
            let buffers = buffers
                .as_ref()
                .expect("grouped logging requires buffer to be present");
            // We hold the mutex until we write all of the bytes associated for the client
            // to ensure that the bytes aren't interspersed.
            let mut writers = writers.lock().expect("lock poisoned");
            if let Some(prefix) = header {
                let start_time = chrono::Utc::now();
                writers.out.write_all(prefix(start_time).as_bytes())?;
            }
            for SinkBytes {
                buffer,
                destination,
            } in buffers
            {
                let writer = match destination {
                    Destination::Stdout => &mut writers.out,
                    Destination::Stderr => &mut writers.err,
                };
                writer.write_all(buffer)?;
            }
            if let Some(suffix) = footer {
                let end_time = chrono::Utc::now();
                writers.out.write_all(suffix(end_time).as_bytes())?;
            }
        }

        Ok(buffers.map(|buffers| {
            // TODO: it might be worth the list traversal to calculate length so we do a
            // single allocation
            let mut bytes = Vec::new();
            for SinkBytes { buffer, .. } in buffers {
                bytes.extend_from_slice(&buffer[..]);
            }
            bytes
        }))
    }

    fn handle_bytes(&self, bytes: SinkBytes) -> io::Result<()> {
        if matches!(
            self.behavior,
            OutputClientBehavior::InMemoryBuffer | OutputClientBehavior::Grouped
        ) {
            // This reconstruction is necessary to change the type of bytes from
            // SinkBytes<'a> to SinkBytes<'static>
            let bytes = SinkBytes {
                destination: bytes.destination,
                buffer: bytes.buffer.to_vec().into(),
            };
            self.add_bytes_to_buffer(bytes);
        }
        if matches!(
            self.behavior,
            OutputClientBehavior::Passthrough | OutputClientBehavior::InMemoryBuffer
        ) {
            self.write_bytes(bytes)
        } else {
            // If we only wrote to the buffer, then we consider it a successful write
            Ok(())
        }
    }

    fn write_bytes(&self, bytes: SinkBytes) -> io::Result<()> {
        let SinkBytes {
            buffer: line,
            destination,
        } = bytes;
        let mut writers = self.writers.lock().expect("writer lock poisoned");
        let writer = match destination {
            Destination::Stdout => &mut writers.out,
            Destination::Stderr => &mut writers.err,
        };
        writer.write_all(&line)
    }

    fn add_bytes_to_buffer(&self, bytes: SinkBytes<'static>) {
        let buffer = self
            .buffer
            .as_ref()
            .expect("attempted to add line to nil buffer");
        buffer.write().expect("lock poisoned").push(bytes);
    }
}

impl<W: Write> Write for OutputWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for line in buf.split_inclusive(|b| *b == b'\n') {
            self.buffer.extend_from_slice(line);
            // If the line doesn't end in a newline we assume it isn't finished and add it
            // to the buffer
            if line.ends_with(b"\n") {
                self.logger.handle_bytes(SinkBytes {
                    buffer: self.buffer.as_slice().into(),
                    destination: self.destination,
                })?;
                self.buffer.clear();
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.logger.handle_bytes(SinkBytes {
            buffer: self.buffer.as_slice().into(),
            destination: self.destination,
        })?;
        self.buffer.clear();
        Ok(())
    }
}

---
File: /crates/turborepo-ui/src/prefixed.rs
---

use std::{
    fmt::{Debug, Display},
    io::Write,
};

use console::{Style, StyledObject};
use tracing::error;

use crate::{ColorConfig, LineWriter};

/// Writes messages with different prefixes, depending on log level.
///
/// Note that this does output the prefix when message is empty, unlike the Go
/// implementation. We do this because this behavior is what we actually
/// want for replaying logs.
pub struct PrefixedUI<W> {
    color_config: ColorConfig,
    output_prefix: Option<StyledObject<String>>,
    warn_prefix: Option<StyledObject<String>>,
    error_prefix: Option<StyledObject<String>>,
    out: W,
    err: W,
    default_prefix: StyledObject<String>,
}

impl<W: Write> PrefixedUI<W> {
    pub fn new(color_config: ColorConfig, out: W, err: W) -> Self {
        Self {
            color_config,
            out,
            err,
            output_prefix: None,
            warn_prefix: None,
            error_prefix: None,
            default_prefix: Style::new().apply_to(String::new()),
        }
    }

    pub fn with_output_prefix(mut self, output_prefix: StyledObject<String>) -> Self {
        self.output_prefix = Some(self.color_config.apply(output_prefix));
        self
    }

    pub fn with_warn_prefix(mut self, warn_prefix: StyledObject<String>) -> Self {
        self.warn_prefix = Some(self.color_config.apply(warn_prefix));
        self
    }

    pub fn with_error_prefix(mut self, error_prefix: StyledObject<String>) -> Self {
        self.error_prefix = Some(self.color_config.apply(error_prefix));
        self
    }

    pub fn output(&mut self, message: impl Display) {
        self.write_line(message, Command::Output)
    }

    pub fn warn(&mut self, message: impl Display) {
        self.write_line(message, Command::Warn)
    }

    pub fn error(&mut self, message: impl Display) {
        self.write_line(message, Command::Error)
    }

    fn write_line(&mut self, message: impl Display, command: Command) {
        let prefix = match command {
            Command::Output => &self.output_prefix,
            Command::Warn => &self.warn_prefix,
            Command::Error => &self.error_prefix,
        }
        .as_ref()
        .unwrap_or(&self.default_prefix);
        let writer = match command {
            Command::Output => &mut self.out,
            Command::Warn | Command::Error => &mut self.err,
        };

        // There's no reason to propagate this error
        // because we don't want our entire program to crash
        // due to a log failure.
        if let Err(err) = writeln!(writer, "{}{}", prefix, message) {
            error!("cannot write to logs: {:?}", err);
        }
    }

    /// Construct a PrefixedWriter which will behave the same as `output`, but
    /// without the requirement that messages be valid UTF-8
    pub fn output_prefixed_writer(&mut self) -> PrefixedWriter<&mut W> {
        PrefixedWriter::new(
            self.color_config,
            self.output_prefix
                .clone()
                .unwrap_or_else(|| Style::new().apply_to(String::new())),
            &mut self.out,
        )
    }
}

//
#[derive(Debug, Clone, Copy)]
enum Command {
    Output,
    Warn,
    Error,
}

/// Wraps a writer with a prefix before the actual message.
pub struct PrefixedWriter<W> {
    inner: LineWriter<PrefixedWriterInner<W>>,
}

impl<W: Write> PrefixedWriter<W> {
    pub fn new(color_config: ColorConfig, prefix: StyledObject<impl Display>, writer: W) -> Self {
        Self {
            inner: LineWriter::new(PrefixedWriterInner::new(color_config, prefix, writer)),
        }
    }
}

impl<W: Write> Write for PrefixedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Wraps a writer so that a prefix will be added at the start of each line.
/// Expects to only be called with complete lines.
struct PrefixedWriterInner<W> {
    prefix: String,
    writer: W,
}

impl<W: Write> PrefixedWriterInner<W> {
    pub fn new(color_config: ColorConfig, prefix: StyledObject<impl Display>, writer: W) -> Self {
        let prefix = color_config.apply(prefix).to_string();
        Self { prefix, writer }
    }
}

impl<W: Write> Write for PrefixedWriterInner<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut is_first = true;
        for chunk in buf.split_inclusive(|c| *c == b'\r') {
            // Before we write the chunk we write the prefix as either:
            // - this is the first iteration and we haven't written the prefix
            // - the previous chunk ended with a \r and the cursor is currently as the start
            //   of the line so we want to rewrite the prefix over the existing prefix in
            //   the line
            // or if the last chunk is just a newline we can skip rewriting the prefix
            if is_first || chunk != b"\n" {
                self.writer.write_all(self.prefix.as_bytes())?;
            }
            self.writer.write_all(chunk)?;
            is_first = false;
        }

        // We do end up writing more bytes than this to the underlying writer, but we
        // cannot report this to the callers as the amount of bytes we report
        // written must be less than or equal to the number of bytes in the buffer.
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

---
File: /crates/turborepo-ui/src/sender.rs
---

use std::sync::{Arc, Mutex};

use crate::{
    tui,
    tui::event::{CacheResult, OutputLogs, PaneSize, TaskResult},
    wui::sender,
};

/// Enum to abstract over sending events to either the Tui or the Web UI
#[derive(Debug, Clone)]
pub enum UISender {
    Tui(tui::TuiSender),
    Wui(sender::WebUISender),
}

impl UISender {
    pub fn start_task(&self, task: String, output_logs: OutputLogs) {
        match self {
            UISender::Tui(sender) => sender.start_task(task, output_logs),
            UISender::Wui(sender) => sender.start_task(task, output_logs),
        }
    }

    pub fn restart_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        match self {
            UISender::Tui(sender) => sender.restart_tasks(tasks),
            UISender::Wui(sender) => sender.restart_tasks(tasks),
        }
    }

    pub fn end_task(&self, task: String, result: TaskResult) {
        match self {
            UISender::Tui(sender) => sender.end_task(task, result),
            UISender::Wui(sender) => sender.end_task(task, result),
        }
    }

    pub fn status(&self, task: String, status: String, result: CacheResult) {
        match self {
            UISender::Tui(sender) => sender.status(task, status, result),
            UISender::Wui(sender) => sender.status(task, status, result),
        }
    }
    fn set_stdin(&self, task: String, stdin: Box<dyn std::io::Write + Send>) {
        match self {
            UISender::Tui(sender) => sender.set_stdin(task, stdin),
            UISender::Wui(sender) => sender.set_stdin(task, stdin),
        }
    }

    pub fn output(&self, task: String, output: Vec<u8>) -> Result<(), crate::Error> {
        match self {
            UISender::Tui(sender) => sender.output(task, output),
            UISender::Wui(sender) => sender.output(task, output),
        }
    }

    /// Construct a sender configured for a specific task
    pub fn task(&self, task: String) -> TaskSender {
        match self {
            UISender::Tui(sender) => sender.task(task),
            UISender::Wui(sender) => sender.task(task),
        }
    }
    pub async fn stop(&self) {
        match self {
            UISender::Tui(sender) => sender.stop().await,
            UISender::Wui(sender) => sender.stop(),
        }
    }
    pub fn update_tasks(&self, tasks: Vec<String>) -> Result<(), crate::Error> {
        match self {
            UISender::Tui(sender) => sender.update_tasks(tasks),
            UISender::Wui(sender) => sender.update_tasks(tasks),
        }
    }

    pub async fn pane_size(&self) -> Option<PaneSize> {
        match self {
            UISender::Tui(sender) => sender.pane_size().await,
            // Not applicable to the web UI
            UISender::Wui(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskSender {
    pub(crate) name: String,
    pub(crate) handle: UISender,
    pub(crate) logs: Arc<Mutex<Vec<u8>>>,
}

impl TaskSender {
    /// Access the underlying UISender
    pub fn as_app(&self) -> &UISender {
        &self.handle
    }

    /// Mark the task as started
    pub fn start(&self, output_logs: OutputLogs) {
        self.handle.start_task(self.name.clone(), output_logs);
    }

    /// Mark the task as finished
    pub fn succeeded(&self, is_cache_hit: bool) -> Vec<u8> {
        if is_cache_hit {
            self.finish(TaskResult::CacheHit)
        } else {
            self.finish(TaskResult::Success)
        }
    }

    /// Mark the task as finished
    pub fn failed(&self) -> Vec<u8> {
        self.finish(TaskResult::Failure)
    }

    fn finish(&self, result: TaskResult) -> Vec<u8> {
        self.handle.end_task(self.name.clone(), result);
        self.logs.lock().expect("logs lock poisoned").clone()
    }

    pub fn set_stdin(&self, stdin: Box<dyn std::io::Write + Send>) {
        self.handle.set_stdin(self.name.clone(), stdin);
    }

    pub fn status(&self, status: &str, result: CacheResult) {
        // Since this will be rendered via ratatui we any ANSI escape codes will not be
        // handled.
        // TODO: prevent the status from having ANSI codes in this scenario
        let status = console::strip_ansi_codes(status).into_owned();
        self.handle.status(self.name.clone(), status, result);
    }
}

impl std::io::Write for TaskSender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let task = self.name.clone();
        {
            self.logs
                .lock()
                .expect("log lock poisoned")
                .extend_from_slice(buf);
        }

        self.handle
            .output(task, buf.to_vec())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "receiver dropped"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

