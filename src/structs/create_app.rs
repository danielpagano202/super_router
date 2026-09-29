use std::collections::HashSet;

use crate::structs::create_enums::{Languages, ExtraChoices, Step};
#[derive(Debug)]
pub struct Results {
    pub name: String,
    pub project_type: usize,
    pub languages: Vec<Languages>,
    pub choices: Vec<ExtraChoices>,
}

pub struct ListSet {
    pub items: Vec<&'static str>,
    pub max_selections: usize,
    pub min_selections: usize,
    pub title: &'static str,
}

pub struct CreateApp {
    pub current_step: Step,
    pub list_sets: Vec<ListSet>,
    pub selected_options: HashSet<usize>,
    pub current_highlighted_option: usize,
    pub results: Results,
}

impl CreateApp {
    pub fn new() -> Self {
        Self {
            current_step: Step::ChooseName,
            list_sets: vec![
                ListSet {
                    items: vec![],
                    max_selections: 0,
                    min_selections: 0,
                    title: "Choose your project name",
                },
                ListSet {
                    items: vec![
                        "Blank Project",
                        "Example Project",
                    ],
                    max_selections: 1,
                    min_selections: 1,
                    title: "Choose your project type",
                },
                ListSet {
                    items: vec![
                        "Python",
                        "JavaScript",
                        "TypeScript",
                        "Rust",
                        "Go",
                        "Java",
                        "C",
                    ],
                    max_selections: 7,
                    min_selections: 0,
                    title: "Choose your programming language(s)",
                },
                ListSet {
                    items: vec![
                        "Include a Dockerfile",
                        "Include a Readme",
                    ],
                    max_selections: 2,
                    min_selections: 0,
                    title: "Choose your extra options",
                },
                ListSet {
                    items: vec![],
                    max_selections: 0,
                    min_selections: 0,
                    title: "You are all finished!",
                },

            ],
            selected_options: HashSet::new(),
            current_highlighted_option: 0,
            results: Results {
                name: String::new(),
                project_type: 0,
                languages: Vec::new(),
                choices: Vec::new(),
            },
        }
    }

    pub fn next_item(&mut self) {
        let step_index = self.current_step.clone()  as usize;
        let items_len = self.list_sets[step_index].items.len();
        if items_len == 0 { return; }
        
        self.current_highlighted_option = (self.current_highlighted_option + 1) % items_len;
    }

    pub fn previous_item(&mut self) {
        let step_index = self.current_step.clone() as usize;
        let items_len = self.list_sets[step_index].items.len();
        if items_len == 0 { return; }

        if self.current_highlighted_option == 0{
            self.current_highlighted_option = items_len - 1;
            return;
        }
        self.current_highlighted_option =  (self.current_highlighted_option - 1) % items_len;
    }

    pub fn select_item(&mut self) {
        if self.selected_options.contains(&self.current_highlighted_option) {
            self.selected_options.remove(&self.current_highlighted_option);
        } else if self.selected_options.len() < self.list_sets[self.current_step.clone() as usize].max_selections {
            self.selected_options.insert(self.current_highlighted_option);
        }
    }

    // Advance the selection process step by step
    pub fn confirm_selection(&mut self) {
        let current_set = &self.list_sets[self.current_step.clone() as usize];
        if self.selected_options.len() < current_set.min_selections {
            return;
        }
        match self.current_step {
            Step::ChooseName => {
                self.current_step = Step::ChooseProjectType
            },
            Step::ChooseProjectType => {
                for option in &self.selected_options{
                    self.results.project_type = *option;
                }

                if self.results.project_type == 0 {
                    self.current_step = Step::ChooseExtras;
                }else{
                    self.current_step = Step::ChooseLanguage;
                }
            }
            Step::ChooseLanguage => {
                for option in &self.selected_options {
                    let language = match Languages::try_from(*option){ 
                        Ok(language) => language,
                        Err(_) => continue,
                    };
                    self.results.languages.push(language);
                }
                self.current_step = Step::ChooseExtras
            },
            Step::ChooseExtras => {
                for option in &self.selected_options {
                    let extra = match ExtraChoices::try_from(*option){ 
                        Ok(extra) => extra,
                        Err(_) => continue,
                    };
                    self.results.choices.push(extra);
                }
                self.current_step = Step::Confirmation
            },
            Step::Confirmation => {
                self.current_step = Step::Confirmation
            },
        }
        self.selected_options = HashSet::new();
        self.current_highlighted_option = 0;
    }
}
