use core::fmt;
use std::process::exit;

use anyhow::Context;

fn ellipsis(text: &str) -> String {
    let max_length = 67;
    if text.len() > max_length {
        format!("{}...", &text[..max_length])
    } else {
        text.to_string()
    }
}

#[derive(Debug,Clone)]
pub struct Buffer {
    chapter: String,
    penal_code: String,
    summary: Option<String>,
    illustrations: Option<Vec<String>>,
    sidenotes: Option<Vec<String>>,
}
impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        let summary: String = match &self.summary {
            Some(s) => s.to_string(),
            None => "".to_string(),
        };

        let illustrations: String = match &self.illustrations {
            Some(i) => i.join("\n").to_string(),
            None => "".to_string(),
        };

        let side_notes: String = match &self.sidenotes {
            Some(s) => s.join("\n").to_string(),
            None => "".to_string(),
        };

        write!(
            f,
            "Chapter: {}\nPenal Code: {}\nSummary: {}\nIllustrations: {:?}\nSidenotes: {:?}",
            ellipsis(&self.chapter),
            ellipsis(&self.penal_code),
            ellipsis(&summary),
            ellipsis(&illustrations),
            ellipsis(&side_notes),
        )
    }
}

pub fn from_chapter(chapter: String) -> Buffer {
    Buffer {
        chapter,
        penal_code: String::new(),
        summary: None,
        illustrations: None,
        sidenotes: None,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsingState {
    Chapter,
    Summary,
    Illustrations,
    Sidenotes,
}

pub fn show_prompt(current_buffer: Buffer, parsing_state: ParsingState) -> String {
    let suffix = match parsing_state {
        ParsingState::Summary => "Summary",
        ParsingState::Illustrations => "Illustrations",
        ParsingState::Sidenotes => "Sidenotes",
        ParsingState::Chapter => "Chapter",
    }; 
    let prompt = format!("{}
    Press: i: ignore, s.<section_id>: start new section, c.<chapter_id>: sets chapter
    u: summary, l:illustrations, d: side_bar, a: add to current ({}) q: quit
    ", current_buffer, suffix);
    prompt.to_string()
}

pub fn make_window(i: usize, lines: Vec<String>) -> String {
    let current: String = lines.get(i).unwrap_or(&"".to_string()).to_string();
    let combined_string = format!(">>>> \t{}", current);
    let lines_to_print: [String; 6] = [
        "\n\n\n---------------------------------------".to_string(),
        combined_string,
        lines.get(i + 1).is_some().then(|| lines[i + 1].to_string()).unwrap_or("".to_string()),
        lines.get(i + 2).is_some().then(|| lines[i + 2].to_string()).unwrap_or("".to_string()),
        lines.get(i + 3).is_some().then(|| lines[i + 3].to_string()).unwrap_or("".to_string()),
        lines.get(i + 4).is_some().then(|| lines[i + 4].to_string()).unwrap_or("".to_string()),
    ];
    lines_to_print.join("\n")
}

pub fn parse_input(lines: Vec<String>) -> Vec<Buffer> {
    let mut buffers: Vec<Buffer> = vec![];
    let mut buffer = from_chapter("".to_string());
    
    let mut parsing_state = ParsingState::Chapter;

    let lines_iter = lines.clone().into_iter().enumerate();
    for (idx, line) in lines_iter {
        if line.is_empty() {
            continue;
        }

        let paragrah_window = make_window(idx, lines.clone());
        let buffer_info = show_prompt(buffer.clone(), parsing_state.clone());
        let final_prompt = format!("{}\n\n\n{}", paragrah_window, buffer_info);

        let mut rl = rustyline::DefaultEditor::new()
            .context("Couldnt create readline")
            .unwrap();
        let readline = rl.readline(&final_prompt);
        let input = match readline {
            Ok(line) => line,
            Err(_) => {
                println!("Couldnt read line");
                exit(1);
            }
        };
        
        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        if input == "q" {
            break;
        }

        if input == "i" {
            continue;
        }

        // Press: i: ignore, s.<section_id>: start new section, c.<chapter_id>: sets chapter
        // u: summary, l:illustrations, d: side_bar, a: add to current ({}) q: quit
        if input.starts_with("a") {
            match parsing_state {
                ParsingState::Summary => {
                    let summary = format!("{}\n{}", buffer.summary.unwrap_or("".to_string()), line);
                    buffer.summary = Some(summary);
                }
                ParsingState::Illustrations => {
                    let mut illustrations = buffer.illustrations.unwrap_or(vec![]);
                    illustrations.push(line);
                    buffer.illustrations = Some(illustrations);
                }
                ParsingState::Sidenotes => {
                    let mut sidenotes = buffer.sidenotes.unwrap_or(vec![]);
                    sidenotes.push(line);
                    buffer.sidenotes = Some(sidenotes);
                },
                ParsingState::Chapter => {
                    let chapter = format!("{}\n{}", buffer.chapter, line);
                    buffer.chapter = chapter;
                    continue;
                },
                _ => {
                    println!("Invalid state");
                    continue;
                }
            }
        } else if input.starts_with("s.") {
            let section_id = input.replace("s.", "");
            buffer.penal_code = section_id;
            parsing_state = ParsingState::Summary;
            buffer.summary = Some(line);
        } else if input.starts_with("c.") {
            parsing_state = ParsingState::Chapter;
            let chapter = format!("Chapter{}\n", input.replace("c.", ""));
            buffer.chapter = chapter;
        } else if input == "u" {
            parsing_state = ParsingState::Summary;
            let summary = format!("{}\n{}", buffer.summary.unwrap_or("".to_string()), line);
            buffer.summary = Some(summary);
        } else if input == "l" {
            parsing_state = ParsingState::Illustrations;
            let mut illustrations = buffer.illustrations.unwrap_or(vec![]);
            illustrations.push(line);
            buffer.illustrations = Some(illustrations);
        } else if input == "d" {
            parsing_state = ParsingState::Sidenotes;
            let mut sidenotes = buffer.sidenotes.unwrap_or(vec![]);
            sidenotes.push(line);
            buffer.sidenotes = Some(sidenotes);
        }
        buffers.push(buffer.clone());
    }

    buffers
}
