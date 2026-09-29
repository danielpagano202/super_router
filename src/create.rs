use std::cmp::{max, min};
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, List, ListItem, Paragraph},
};
use crate::structs::create_enums::{Step};
use crate::structs::create_app::CreateApp;


fn draw_simple_list(frame: &mut Frame, area: Rect, app: &mut CreateApp) {
    let max_shown_items = 3;
    let current_list_set = &app.list_sets[app.current_step.clone() as usize];
    let items = current_list_set.items.clone();

    // 2. Convert the text strings into ListItems
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, text)| {
                let color_to_use = if app.current_highlighted_option == i {
                    Color::Yellow
                }else{
                    Color::White
                };

                let selection = if app.selected_options.contains(&i) {
                    "☑"
                } else {
                    "▢"
                };

                let line = Line::from(vec![
                    Span::styled(selection, Style::default().fg(color_to_use)),
                    Span::styled(" ", Style::default().fg(color_to_use)),
                    Span::styled(*text, Style::default().fg(color_to_use))
                ]);

                ListItem::new(line)
            }
        )
        .collect();

    let lower_bound = max(app.current_highlighted_option as i32 - (max_shown_items as i32 / 2), 0) as usize;
    let upper_bound = min(max(app.current_highlighted_option as i32 + (max_shown_items as i32 / 2), (lower_bound + max_shown_items) as i32), items.len() as i32) as usize;
    let lower_bound = max(upper_bound as i32 - max_shown_items as i32, 0) as usize;
    let items_slice = list_items.get(
        lower_bound..upper_bound
    ).unwrap().to_vec();

    // 3. Build the List widget with optional borders and styling
    let simple_list = List::new(items_slice)
        .block(Block::bordered().title(current_list_set.title))
        .style(Style::default().fg(Color::White));

    // 4. Render it directly (no ListState reference required)
    frame.render_widget(simple_list, area);

}

pub fn run() -> std::io::Result<()> {
    let mut app = CreateApp::new();
    let mut project_name = String::new();

    _ = ratatui::run(|terminal| {
        loop {
            // Draw UI frame
            _ = terminal.draw(|frame| {
                // Split screen area vertically: header, choice body, footer instructions
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Header
                        Constraint::Min(5),    // App Body
                        Constraint::Length(3), // Footer
                    ])
                    .split(frame.area());
                let header = Paragraph::new("Super Router Creator Wizard")
                    .centered()
                    .block(Block::bordered().border_style(Style::default().fg(Color::Cyan)));

                frame.render_widget(header, chunks[0]);

                match app.current_step {
                    Step::ChooseName => {
                        let name_input = Line::from(vec![
                            Span::raw("Project Name: "),
                            Span::raw(project_name.clone())
                        ])
                        .centered();

                        frame.render_widget(name_input, chunks[1]);
                    }
                    Step::ChooseProjectType => draw_simple_list(frame, chunks[1], &mut app),
                    Step::ChooseLanguage => draw_simple_list(frame, chunks[1], &mut app),
                    Step::ChooseExtras => draw_simple_list(frame, chunks[1], &mut app),
                    Step::Confirmation => {
                        let final_project_name = &app.results.name;
                        let confirmation_text = format!("You are all finished!\nYour code is in `/{final_project_name}`\nRun `superrouter build {final_project_name}` from this directory to run the code and `superrouter run {final_project_name}` to start the application.");
                        let confirmation_paragraph = Paragraph::new(confirmation_text)
                            .centered()
                            .block(Block::bordered().border_style(Style::default().fg(Color::Green)));
                        frame.render_widget(confirmation_paragraph, chunks[1]);
                    }
                };
                let footer_text = match app.current_step {
                    Step::Confirmation => "Press [Esc] to exit application.",
                    Step::ChooseName => "Press [Enter] to confirm project name after typing it, [Esc] to exit.",
                    _ => "Use [↓/↑] or [j/k] to navigate, [Space] to select, [Enter] to confirm selection, [Esc] to quit.",
                };
                let footer = Paragraph::new(footer_text)
                    .centered()
                    .block(Block::bordered());
                frame.render_widget(footer, chunks[2]);
            });
            if let event::Event::Key(key) = event::read().expect("Can't read keys") {
                if key.kind == KeyEventKind::Press {
                    match app.current_step {
                        Step::ChooseName => {
                            match key.code {
                                        // Exit input mode
                                        KeyCode::Esc => break,
                                        
                                        // Handle backspaces (removes the last character safely)
                                        KeyCode::Backspace => {
                                            project_name.pop();
                                        }
                                        
                                        // Handle generic typing characters
                                        KeyCode::Char(c) => {
                                            project_name.push(c);
                                        }
                                        
                                        // Process the submitted string
                                        KeyCode::Enter => {
                                            app.results.name = project_name.clone();
                                            app.confirm_selection();
                                        }
                                        _ => {}
                                    }
                        }
                        _ => {
                            match key.code {
                                KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                                KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                                KeyCode::Char(' ') => app.select_item(),
                                KeyCode::Enter => app.confirm_selection(),
                                KeyCode::Esc => break,
                                _ => {}
                            }
                        }
                    };
                }
            };
            
        }
    });
    println!("{:?}", app.results);
    return Ok(());
}
