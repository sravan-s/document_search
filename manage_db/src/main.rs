mod prompt;

use anyhow::{Context, Result};
use dotenvy::dotenv;
use prompt::parse_input;
use postgres::{Client, NoTls};
use regex::Regex;
use std::{env, fs::read_to_string};


fn cleanup_chapter(text: &str) -> String {
    let footer_pattern: &str = r#"(?m).*GAZETTE OF INDIA EXTRAORDINARY$|(?m)^.*_{5,}\s*$"#;
    let remove_footer_text: Regex = Regex::new(footer_pattern).unwrap();
    let cleaned_text = remove_footer_text.replace_all(text, "");
    // Trim leading and trailing whitespace (including newlines) from each line
    let trimmed_lines: Vec<&str> = cleaned_text.lines().map(|line| line.trim()).collect();

    // Join trimmed lines into a single string with combined paragraphs
    let combined_text = trimmed_lines.join("\n").replace("\n\n", "\n");

    combined_text.to_string()
}

fn split_into_chapters(text: &str) -> Vec<String> {
    // Define the regex pattern for chapter headers
    let chapter_pattern = r"(?m)\nCHAPTER *\w+";
    let re = Regex::new(chapter_pattern).unwrap();

    // Split the text into chapters
    let mut chapters = Vec::new();
    let mut last_index = 0;

    for mat in re.find_iter(text) {
        // Push the previous chapter text
        if last_index != 0 {
            chapters.push(text[last_index..mat.start()].to_string());
        }

        last_index = mat.start();
    }

    // Push the last chapter text
    if last_index != 0 {
        chapters.push(text[last_index..].to_string());
    } else {
        // If no chapters are found, return the whole text as one chapter
        chapters.push(text.to_string());
    }

    chapters
}

fn write_chapter_to_db(db_client: &mut Client, chapter: String, idx: i32) -> std::io::Result<()> {
    db_client.execute("INSERT INTO chapters_to_move (id, chapter) values ($1, $2)"
        , &[&idx, &chapter]
    )
    .context("Coundt insert chapter into chapters_to_move table")
    .unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let _ = dotenv().context("Couldnt read env file");
    let postgres_pass = env::var("POSTGRES_PASSWORD")
        .context("Couldnt read postgres_pass")
        .unwrap();
    let postgres_port = env::var("POSTGRES_PORT")
        .context("Couldnt read postgres_port")
        .unwrap();

    let postgres_connection_param = format!(
        "host=localhost user=postgres password={} port={}",
        postgres_pass, postgres_port
    );

    let mut client = Client::connect(&postgres_connection_param.to_string(), NoTls)
        .context("Couldnt connect to postgres")
        .unwrap();

    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS laws (
            penal_code VARCHAR(20) PRIMARY KEY,
            chapter TEXT,
            summary TEXT,
            illustrations TEXT[],
            sidenotes TEXT[]);
            CREATE TABLE IF NOT EXISTS moved_chapters (
                id SERIAL PRIMARY KEY,
                chapter TEXT
            );
            CREATE TABLE IF NOT EXISTS chapters_to_move (
                id SERIAL PRIMARY KEY,
                chapter TEXT
            );",
        )
        .context("Couldnt create table laws")
        .unwrap();

    let chapters_in_db = client
        .execute("SELECT count(*) from chapters_to_move", &[])
        .context("Couldnt select chapter from chapters_to_move table")
        .unwrap();

    if chapters_in_db == 0 {
        println!("Adding chapters to database");
        let txt = read_to_string("../data/laws.txt").unwrap();
        let cleaned_text = split_into_chapters(&txt);
        for (index, chapter) in cleaned_text.iter().enumerate() {
            let cleaned_chapter = cleanup_chapter(chapter);
            let idx = (index + 1) as i32;
            write_chapter_to_db(&mut client,cleaned_chapter, idx)
                .context("Couldnt write chapters to disk")
                .unwrap();
        }
        println!("Added chapters to database");
    }
    
    let _result: Vec<String> = Vec::new();
    let work_done = client
        .execute("SELECT count (*) chapter from moved_chapters", &[])
        .context("Couldnt select chapter from moved_chapters table")
        .unwrap();

    if work_done == 0 {
        println!("Starting to move chapters to database from chapter: one");
    } else {
        println!("Continuing to move from chapter: {work_done} to database");
    }
    
    let chapters_to_move = client
        .query("SELECT * from chapters_to_move", &[])
        .context("Couldnt select chapter from chapters_to_move table")
        .unwrap();

    for chapter in chapters_to_move {
        let chapter_text: String = chapter.get(1);
        let lines: Vec<String> = chapter_text.lines().map(|line| line.to_string()).collect();
        let buffers = parse_input(lines);
        print!("{:?}", buffers);
        // let _ = client
        //     .execute("INSERT INTO moved_chapters (chapter) values ($1)", &[&buffer.chapter])
        //     .context("Coundt insert chapter into moved_chapters table")
        //     .unwrap();
    }
    println!("outing");

    Ok(())
}
