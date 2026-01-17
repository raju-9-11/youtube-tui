use std::error::Error;
use std::sync::Arc;

use crate::global::structs::{Page, LoginPage, Tasks, Task, TaskFunction, Message};
use crate::global::traits::SearchProviderWrapper;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_additions::framework::{FrameworkClean, FrameworkItem, ItemInfo};

#[derive(Clone, Default)]
pub struct LoginItem {
    pub page: LoginPage,
}

impl FrameworkItem for LoginItem {
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        _framework: &mut FrameworkClean,
        area: Rect,
        _popup_render: bool,
        _info: ItemInfo,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        let title = Paragraph::new("Google Login")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        if let Some(err) = &self.page.error {
             let error_msg = Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            frame.render_widget(error_msg, chunks[1]);
        } else if !self.page.user_code.is_empty() {
             let text = vec![
                Line::from(vec![
                    Span::raw("Please visit: "),
                    Span::styled(&self.page.verification_url, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("And enter code: "),
                    Span::styled(&self.page.user_code, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]),
            ];

            let info = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Instructions"));
            frame.render_widget(info, chunks[1]);

            let status = Paragraph::new("Waiting for authentication... (Press 'q' or 'back' to cancel)")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(status, chunks[2]);

        } else if self.page.loading {
             let status = Paragraph::new("Initializing login flow...")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(status, chunks[1]);
        }
    }

    fn load_item(
        &mut self,
        framework: &mut FrameworkClean,
        _info: ItemInfo,
    ) -> Result<(), Box<dyn Error>> {
        // If we already have a code, don't reload
        if !self.page.user_code.is_empty() {
            return Ok(());
        }

        self.page.loading = true;

        // Start login flow in a separate task

        let tasks = framework.data.state.get_mut::<Tasks>().unwrap();

        // Push a custom task to run login_start
        tasks.priority.push(Task::Custom(TaskFunction::new(Arc::new(
            move |framework| {
                match SearchProviderWrapper::login_start() {
                    Ok((code, url)) => {
                        // Update the page state
                        if let Page::Login(ref mut page) = framework.data.state.get_mut::<Page>().unwrap() {
                            page.user_code = code.clone();
                            page.verification_url = url;
                            page.loading = false;
                        }

                        // Spawn the wait task
                        let code_clone = code.clone();
                        framework.data.state.get_mut::<Tasks>().unwrap().last.push(Task::Custom(TaskFunction::new(Arc::new(
                            move |_framework| {
                                let code = code_clone.clone();

                                std::thread::spawn(move || {
                                     let res = SearchProviderWrapper::login_wait(&code);
                                     let mut status = LOGIN_STATUS.lock().unwrap();
                                     *status = Some(res.map(|_| true).unwrap_or(false));
                                });
                            }
                        ))));
                    },
                    Err(e) => {
                         if let Page::Login(ref mut page) = framework.data.state.get_mut::<Page>().unwrap() {
                            page.error = Some(e.to_string());
                            page.loading = false;
                        }
                    }
                }
            }
        ))));

        framework.data.state.get_mut::<Tasks>().unwrap().priority.push(Task::RenderAll);

        Ok(())
    }

    fn selectable(&self) -> bool {
        true
    }

    fn key_event(
        &mut self,
        framework: &mut FrameworkClean,
        key: crossterm::event::KeyEvent,
        _info: ItemInfo,
    ) -> Result<(), Box<dyn Error>> {
        // Check for 'q' or 'Esc' to exit
        if key.code == crossterm::event::KeyCode::Char('q') || key.code == crossterm::event::KeyCode::Esc {
             framework.data.state.get_mut::<Tasks>().unwrap().priority.push(Task::LoadPage(Page::MainMenu(Default::default())));
             return Ok(());
        }

        // Poll the status
        {
            let mut status = LOGIN_STATUS.lock().unwrap();
            if let Some(success) = *status {
                // Clear status
                *status = None;
                if success {
                    *framework.data.global.get_mut::<Message>().unwrap() = Message::Success("Login successful!".to_string());
                    framework.data.state.get_mut::<Tasks>().unwrap().priority.push(Task::LoadPage(Page::MainMenu(Default::default())));
                } else {
                     self.page.error = Some("Login failed.".to_string());
                     framework.data.state.get_mut::<Tasks>().unwrap().priority.push(Task::RenderAll);
                }
            }
        }

        Ok(())
    }
}

static LOGIN_STATUS: std::sync::LazyLock<std::sync::Mutex<Option<bool>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));
