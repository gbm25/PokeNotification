use regex::Regex;
use scraper::{element_ref::Select, ElementRef, Html, Node, Selector};
use std::error::Error;

// Define a struct to represent the Pokemon in the eventy
#[derive(Debug)]
struct Pokemon {
    name: Option<String>,
    gender: Option<String>,
    level: Option<String>,
    ot: Option<String>,
    id: Option<String>,
    ability: Vec<String>,
    tera_type: Option<String>,
    hold_item: Option<String>,
    nature: Option<String>,
    moves: Vec<String>,
    ribbons: Vec<String>,
}

// Define a struct to represent the event
#[derive(Debug)]
struct Event {
    title: String,
    release_dates: String,
    event_description: String,
    pokemons_info: Vec<Pokemon>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO Define a struicture to handle multiple scrapers (maybe in the future we want one per generation)
    // Define the URL of the website to scrape
    let body = reqwest::get("https://www.serebii.net/scarletviolet/serialcode.shtml")
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&body);

    // TODO May be more than one, i dont know if io should filter or process all
    // Define a CSS selector to select tables with class "tab"
    let tab_selector = Selector::parse("table.tab").unwrap();

    // Iterate over each table with class "tab"
    for tab in document.select(&tab_selector) {
        // Define CSS selectors to select title and content
        let title_selector = Selector::parse("td.fooleft h3").unwrap();
        let content_selector = Selector::parse("td.foocontent").unwrap();

        let events_titles = tab.select(&title_selector);
        let events_info = tab.select(&content_selector);

        process_event_data(events_titles, events_info)
    }

    Ok(())
}

/// The event data is separated into 2 tr, one for the title and one for the event data.
/// There is no way to identify each one without looking at the elements inside.
/// All the events go in the same tbody, so I leave an example with just one event
///
/// HTML:
/// <table class="eventpoke">
/// 	<tbody><tr>
///    <td class="column" rowspan="2">
///    <table><tbody><tr><td class="label"><img src="/itemdex/sprites/dreamball.png" loading="lazy"> カビゴン  <font color="#499FFF">♂</font></td></tr><tr><td class="pkmn"><a href="/pokedex-sv/snorlax/"><img src="/scarletviolet/pokemon/new/small/143.png" loading="lazy" border="0" width="120" title=""></a></td></tr><tr>
///     <td class="label">Level 20<br><img src="/events/paldea.png" loading="lazy"></td></tr></tbody></table></td><td class="column" rowspan="2"><table>
///     <tbody><tr><td class="detailhead">OT:</td><td>プロカビ</td></tr>
///     <tr><td class="detailhead">ID:</td><td>240223</td></tr>
///     <tr><td class="detailhead">Ability:</td><td><a href="/abilitydex/gluttony.shtml">Gluttony</a></td></tr>
///     <tr><td colspan="2" class="detailhead">Tera Type</td></tr><tr><td colspan="2">Normal</td></tr><tr><td colspan="2" class="detailhead">Hold Item:</td></tr><tr><td colspan="2">No Item</td></tr></tbody></table></td>
///     <td class="column">Any Nature.<br>Date of Receiving<br> a lovely place. Apparently had a fateful encounter at Lv. 20</td><td class="column"><table width="100"><tbody><tr><td><a href="/attackdex-sv/rest.shtml">Rest</a></td></tr>
/// <tr><td><a href="/attackdex-sv/block.shtml">Block</a></td></tr>
/// <tr><td><a href="/attackdex-sv/yawn.shtml">Yawn</a></td></tr>
/// <tr><td><a href="/attackdex-sv/bodyslam.shtml">Body Slam</a></td></tr>
/// </tbody></table></td>
///     <td> <img src="/games/ribbons/uncommonribbon.png" loading="lazy" style="max-width:32px" title="Uncommon Ribbon">
///     </td>
/// </tr>
/// </tbody></table>
/// </td>
/// <td class="picturetd" width="300" valign="top"><img src="snorlaxgift.jpg" alt="Iron Valiant &amp; Roaring Moon Event Image" class="contentpic" loading="lazy"></td>
/// </tr>
/// </tbody></table>
fn process_event_data(events_titles: Select, events_info: Select) {
    // TODO remove after testing
    let mut debug_loop_count = 0;

    // Combine the iterators using zip
    for (title_element, info_element) in events_titles.zip(events_info) {
        println!(
            "######################### START EVENT NUMBER {} #########################",
            debug_loop_count
        );

        let title = title_element.text().collect::<String>();

        // TODO Write parser for release_dates
        let mut release_dates = String::new();
        // TODO Write parser for event_description
        let mut event_description: String = String::new();

        let pokemon_table_selector = Selector::parse("table.eventpoke").unwrap();

        let pokemon_tables = info_element.select(&pokemon_table_selector);
        
        let mut pokemons_info: Vec<Pokemon> = Vec::new();

        for pokemon_table in pokemon_tables {
            let pokemon_info: Pokemon = parse_pokemon_details(&pokemon_table);
            pokemons_info.push(pokemon_info)
        }

        // Create an Event struct with the extracted data
        let event = Event {
            title,
            release_dates,
            event_description,
            pokemons_info,
        };

        println!("\tEvent: {:?}", event);
        println!(
            "######################### END EVENT NUMBER {} #########################",
            debug_loop_count
        );

        debug_loop_count += 1;
    }
}

fn parse_pokemon_details(event_data: &ElementRef) -> Pokemon {

    // Define CSS selectors to select the pokemon information
    let name_selector = Selector::parse("td.pkmn a[href]").unwrap();
    let gender_selector = Selector::parse("td.label font").unwrap();
    let level_selector = Selector::parse("td.label").unwrap(); // we should take the second one from here
    let detailhead_selector = Selector::parse("td.detailhead").unwrap();
    let tera_type_selector = Selector::parse("td").unwrap();
    let column_selector = Selector::parse("td.column").unwrap();
    let moves_selector = Selector::parse("table td a").unwrap();


    // TODO The name can be a compound name, as in the case of “Roaring Moon”, we will have to look at what to do to recover the name properly.
    // Extract pokemon information
    let name = if let Some(name) = get_element_attr_value(event_data, &name_selector, "href") {
        extract_pokemon_name(name) 
    } else {
        None
    };
    let gender = extract_gender(event_data, &gender_selector);
    let level = get_nth_element_text(event_data, &level_selector, 1);
    let ot = get_sibling_text(event_data, &detailhead_selector, 0); // TODO Pending
    let id = get_sibling_text(event_data, &detailhead_selector, 1); // TODO Pending
                                                                     // let ability = get_sibling_text(&event_data, &detailhead_selector, 2); // TODO Pending, may be more than 1 ability
    let ability = Vec::new();
    let tera_type = get_nth_element_text(event_data, &tera_type_selector, 1); // TODO Pending
    let hold_item = get_nth_element_text(event_data, &tera_type_selector, 2); // TODO Pending
    let nature = get_element_text(event_data, &column_selector); // TODO Pending
    let moves = event_data
        .select(&moves_selector)
        .map(|move_elem| move_elem.text().collect::<String>())
        .collect::<Vec<_>>(); // TODO Pending, mixed with abilities sometimes
    let ribbons = Vec::new();

    // Create and return a Pokemon struct with the extracted info
    Pokemon {
        name,
        gender,
        level,
        ot,
        id,
        ability,
        tera_type,
        hold_item,
        nature,
        moves,
        ribbons,
    }
}

fn extract_gender(document: &ElementRef, selector: &Selector) -> Option<String> {
    let gender = document
        .select(&selector)
        .into_iter()
        .filter(|element| {
            let gender_text = element.text().collect::<String>();
            gender_text == "♂" || gender_text == "♀"
        })
        .map(|element| element.text().collect::<String>())
        .collect::<Vec<String>>()
        .join("/");

    if gender.len() > 0 {
        Some(gender)
    } else {
        None
    }
}
fn get_element_text(document: &ElementRef, selector: &Selector) -> Option<String> {
    document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>())
}

fn get_element_attr_value(document: &ElementRef, selector: &Selector, attr: &str) -> Option<String> {
    document
        .select(&selector)
        .next()
        .map(|element| element.attr(attr).unwrap_or_default().to_string())
}

fn get_sibling_text(document: &ElementRef, selector: &Selector, index: usize) -> Option<String> {
    document.select(&selector).nth(index).and_then(|element| {
        element
            .next_sibling()
            .and_then(|sibling| match sibling.value() {
                Node::Text(text_node) => Some(text_node.text.to_string()),
                _ => None,
            })
    })
}

fn get_nth_element_text(document: &ElementRef, selector: &Selector, n: usize) -> Option<String> {
    document
        .select(&selector)
        .nth(n)
        .map(|element| element.text().collect::<String>())
}

fn extract_pokemon_name(name_url_formatted: String) -> Option<String> {
    println!("\r\n POKEMON NAME :{}\r\n", name_url_formatted);
    let re = Regex::new(r"/[a-zA-Z\-]+/(?P<name>[a-zA-Z0-9]+)").unwrap();
    if let Some(captures) = re.captures(&name_url_formatted) {
        if let Some(name) = captures.name("name") {
            return Some(capitalize(name.as_str()));
        }
    }

    None
}

fn capitalize(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
