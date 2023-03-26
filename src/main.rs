mod configuration;

use anyhow::Context;
use clap::Parser;
use configuration::AppConfig;
use dialoguer::{theme::ColorfulTheme, Input, Password};
use notion::{
    ids::BlockId,
    models::{
        block::{CreateBlock, TextAndChildren},
        text::{Annotations, RichText, RichTextCommon, Text, TextColor},
        UpdateBlockChildrenRequest,
    },
    NotionApi,
};
use regex::Regex;
use std::str::FromStr;

#[derive(Parser)]
#[command()]
struct Cli {
    #[arg(short, long)]
    element_id: Option<String>,

    #[arg(short, long)]
    text: Option<String>,

    #[arg(short, long)]
    save_token: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let term_theme = ColorfulTheme::default();

    if cli.save_token {
        let api_key: String = Password::with_theme(&term_theme)
            .with_prompt("Notion api key:")
            .interact()?;
        let config = AppConfig::new(api_key);
        config.save_user_config()?;
        return Ok(());
    }

    let id_or_url = if let Some(element_id) = cli.element_id {
        element_id
    } else {
        Input::with_theme(&term_theme)
            .with_prompt("Notion element id:")
            .interact_text()?
    };

    let id = extract_element_id_from_link(&id_or_url)?;

    let text = if let Some(text) = cli.text {
        text
    } else {
        Input::with_theme(&term_theme)
            .with_prompt("Input text:")
            .interact_text()?
    };

    let config = configuration::AppConfig::load_user_config()?;
    let notion_api = NotionApi::new(config.notion_api_key)?;

    let rainbows: Vec<&[TextColor]> = vec![
        &GAY_PRIDE_RAINBOW_SIX,
        &GAY_PRIDE_RAINBOW_SIX_BACKGROUND,
        &GAY_PRIDE_RAINBOW_SEVEN,
        &GAY_PRIDE_RAINBOW_SEVEN_BACKGROUND,
        &BI_PRIDE_THREE,
        &BI_PRIDE_THREE_BACKGROUND,
        &LESBIAN_PRIDE_FIVE,
        &LESBIAN_PRIDE_SIX,
        &TRANS_PRIDE,
        &ACE_PRIDE,
    ];

    let mut children = vec![];

    for rainbow_pattern in &rainbows {
        let new_element = gay_agendify_with_automated_size(&text, rainbow_pattern);
        children.push(new_element);
    }

    for rainbow_pattern in &rainbows {
        let new_element = gay_agendify_repeating(&text, rainbow_pattern);
        children.push(new_element);
    }

    let request = UpdateBlockChildrenRequest { children };

    let block_id = BlockId::from_str(&id).unwrap();

    notion_api
        .append_block_children(&block_id, request)
        .await
        .unwrap();

    Ok(())
}

fn extract_element_id_from_link(link: &str) -> anyhow::Result<String> {
    let re = Regex::new(r"([0-9a-fA-F]{32})")?;
    let id = re
        .captures(link)
        .and_then(|caps| caps.get(1).map(|text| text.as_str().to_string()))
        .context("Failed to extract id from link")?;
    Ok(id)
}

fn gay_agendify_with_automated_size(text: &str, rainbow: &[TextColor]) -> CreateBlock {
    let chunk_size = text.len() / rainbow.len();
    gay_agendify(text, chunk_size, rainbow)
}

fn gay_agendify_repeating(text: &str, rainbow: &[TextColor]) -> CreateBlock {
    gay_agendify(text, 1, rainbow)
}

fn gay_agendify(text: &str, step: usize, rainbow: &[TextColor]) -> CreateBlock {
    let mut rainbow_repeat = rainbow.iter().cycle();

    let mut rich_text = vec![];

    let characters: Vec<_> = text.chars().collect();

    for i in (0..characters.len()).step_by(step) {
        let end = if i + step > characters.len() {
            characters.len()
        } else {
            i + step
        };
        let chunk: String = characters[i..end].iter().collect();

        let color = rainbow_repeat.next().unwrap().to_owned();

        let annotations = Annotations {
            color: Some(color),
            ..Default::default()
        };

        rich_text.push(RichText::Text {
            rich_text: RichTextCommon {
                plain_text: chunk.clone(),
                href: None,
                annotations: Some(annotations),
            },
            text: Text {
                content: chunk,
                link: None,
            },
        });
    }

    let paragraph = TextAndChildren {
        rich_text,
        children: None,
        color: TextColor::Default,
    };
    CreateBlock::Paragraph { paragraph }
}

// heh github copilot just filled this in
const GAY_PRIDE_RAINBOW_SIX: [TextColor; 6] = [
    TextColor::Red,
    TextColor::Orange,
    TextColor::Yellow,
    TextColor::Green,
    TextColor::Blue,
    TextColor::Purple,
];

const GAY_PRIDE_RAINBOW_SIX_BACKGROUND: [TextColor; 6] = [
    TextColor::RedBackground,
    TextColor::OrangeBackground,
    TextColor::YellowBackground,
    TextColor::GreenBackground,
    TextColor::BlueBackground,
    TextColor::PurpleBackground,
];

const GAY_PRIDE_RAINBOW_SEVEN: [TextColor; 7] = [
    TextColor::Pink,
    TextColor::Red,
    TextColor::Orange,
    TextColor::Yellow,
    TextColor::Green,
    TextColor::Blue,
    TextColor::Purple,
];

const GAY_PRIDE_RAINBOW_SEVEN_BACKGROUND: [TextColor; 7] = [
    TextColor::PinkBackground,
    TextColor::RedBackground,
    TextColor::OrangeBackground,
    TextColor::YellowBackground,
    TextColor::GreenBackground,
    TextColor::BlueBackground,
    TextColor::PurpleBackground,
];

const BI_PRIDE_THREE: [TextColor; 3] = [TextColor::Pink, TextColor::Purple, TextColor::Blue];

const BI_PRIDE_THREE_BACKGROUND: [TextColor; 3] = [
    TextColor::PinkBackground,
    TextColor::PurpleBackground,
    TextColor::BlueBackground,
];

const LESBIAN_PRIDE_FIVE: [TextColor; 5] = [
    TextColor::Red,
    TextColor::Orange,
    TextColor::Default, // this is only white in dark mode
    TextColor::Pink,
    TextColor::Purple,
];

const LESBIAN_PRIDE_SIX: [TextColor; 6] = [
    TextColor::Red,
    TextColor::Orange,
    TextColor::Yellow,  // probably remove yellow?
    TextColor::Default, // this is only white in dark mode
    TextColor::Pink,
    TextColor::Purple,
];

const TRANS_PRIDE: [TextColor; 5] = [
    TextColor::Blue,
    TextColor::Pink,
    TextColor::Default,
    TextColor::Pink,
    TextColor::Blue,
];

const ACE_PRIDE: [TextColor; 4] = [
    TextColor::GrayBackground,
    TextColor::Gray, // you can't make a full ace flag
    TextColor::Default,
    TextColor::Purple,
];
