use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::style::Color;
use ratatui::{Terminal, backend::TestBackend};
use std::fmt::Write;
use tui_alchemy::App;

fn main() {
    let mut app = App::new();
    let backend = TestBackend::new(100, 32);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut html = html_header();

    render_section("initial", &mut terminal, &mut app, &mut html);

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();
    render_section("created steam", &mut terminal, &mut app, &mut html);

    let lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&lines, "water").unwrap_or((8, 10));
    let fire = find_text_position(&lines, "fire").unwrap_or((28, 10));
    app.handle_event(mouse(
        MouseEventKind::Down(MouseButton::Left),
        water.0,
        water.1,
    ));
    app.handle_event(mouse(
        MouseEventKind::Drag(MouseButton::Left),
        fire.0,
        fire.1,
    ));
    app.tick();
    render_section("drag ghost", &mut terminal, &mut app, &mut html);

    app.reveal_elements_for_preview(&[
        "Dust",
        "Energy",
        "Lava",
        "Mud",
        "Pressure",
        "Rain",
        "Sea",
        "Steam",
        "Atmosphere",
        "Brick",
        "Cloud",
        "Plant",
        "Stone",
        "Volcano",
        "Wind",
        "Grass",
        "Metal",
        "Mountain",
        "Sand",
        "Sky",
        "Storm",
        "Glass",
        "Time",
        "Life",
        "Human",
        "Tool",
        "Book",
        "Bird",
        "Fish",
        "House",
        "Tree",
        "Vase",
    ]);
    render_section("seeded chain in game", &mut terminal, &mut app, &mut html);

    let mut object_app = App::new();
    object_app.reveal_elements_for_preview(&[
        "Glass", "Time", "Life", "Human", "Tool", "Book", "Bird", "Fish", "House", "Tree", "Vase",
    ]);
    render_section(
        "authored object sprites in game",
        &mut terminal,
        &mut object_app,
        &mut html,
    );

    let mut iconic_app = App::new();
    iconic_app.reveal_elements_for_preview(&[
        "Sun", "Coal", "Moon", "Flower", "Egg", "Honey", "Paper", "Hammer", "Wheat", "Wood",
        "Snow", "Ice", "Cotton", "Needle", "Chain", "Web", "Spider", "Bee",
    ]);
    render_section(
        "classic reference sprites in game",
        &mut terminal,
        &mut iconic_app,
        &mut html,
    );

    let mut expanded_object_app = App::new();
    expanded_object_app.reveal_elements_for_preview(&[
        "Glasses",
        "Clock",
        "Boat",
        "Car",
        "Cat",
        "Dog",
        "Scissors",
        "Wheel",
        "Blade",
        "Newspaper",
    ]);
    render_section(
        "expanded object sprites in game",
        &mut terminal,
        &mut expanded_object_app,
        &mut html,
    );

    let mut reference_extra_app = App::new();
    reference_extra_app.reveal_elements_for_preview(&[
        "Lizard",
        "Bread",
        "Fishing rod",
        "Crystal ball",
        "Butterfly",
        "Flying fish",
    ]);
    render_section(
        "reference sheet extras in game",
        &mut terminal,
        &mut reference_extra_app,
        &mut html,
    );

    let mut craft_object_app = App::new();
    craft_object_app.reveal_elements_for_preview(&["Axe", "Clay", "Pottery"]);
    render_section(
        "craft object sprites in game",
        &mut terminal,
        &mut craft_object_app,
        &mut html,
    );

    let mut la2_object_app = App::new();
    la2_object_app.handle_event(key(KeyCode::Tab));
    la2_object_app.reveal_elements_for_preview(&["Shovel", "Knife"]);
    render_section(
        "little alchemy 2 object sprites in game",
        &mut terminal,
        &mut la2_object_app,
        &mut html,
    );

    let mut nature_cosmos_app = App::new();
    nature_cosmos_app.reveal_elements_for_preview(&[
        "Seaweed", "Hay", "Bacteria", "Wool", "Cow", "Horse", "Rainbow", "Star",
    ]);
    render_section(
        "nature and cosmos sprites in game",
        &mut terminal,
        &mut nature_cosmos_app,
        &mut html,
    );

    let mut cosmic_electric_app = App::new();
    cosmic_electric_app.reveal_elements_for_preview(&[
        "Planet",
        "Space",
        "Electricity",
        "Wire",
        "Light bulb",
        "Solar system",
        "Galaxy",
        "Telescope",
        "Rocket",
        "Astronaut",
    ]);
    render_section(
        "cosmic and electric sprites in game",
        &mut terminal,
        &mut cosmic_electric_app,
        &mut html,
    );

    let mut natural_force_app = App::new();
    natural_force_app.reveal_elements_for_preview(&[
        "Earthquake",
        "Flood",
        "Geyser",
        "Granite",
        "Gunpowder",
        "Obsidian",
        "Ocean",
        "Salt",
        "Algae",
        "Ash",
        "Eruption",
        "Explosion",
        "Fog",
        "Hurricane",
        "Tsunami",
        "Wave",
    ]);
    render_section(
        "natural force sprites in game",
        &mut terminal,
        &mut natural_force_app,
        &mut html,
    );

    let mut water_force_app = App::new();
    water_force_app.reveal_elements_for_preview(&["Tsunami", "Wave"]);
    render_section(
        "water force sprites in game",
        &mut terminal,
        &mut water_force_app,
        &mut html,
    );

    let mut constructed_botanical_app = App::new();
    constructed_botanical_app.reveal_elements_for_preview(&[
        "Wall",
        "Archipelago",
        "Atomic bomb",
        "Beach",
        "Boiler",
        "Bullet",
        "Cactus",
        "Desert",
        "Dew",
        "Diamond",
        "Dune",
        "Fireworks",
        "Garden",
        "Ivy",
        "Moss",
        "Pond",
    ]);
    render_section(
        "constructed and botanical sprites in game",
        &mut terminal,
        &mut constructed_botanical_app,
        &mut html,
    );

    let mut pond_app = App::new();
    pond_app.reveal_elements_for_preview(&["Moss", "Pond"]);
    render_section(
        "pond and moss sprites in game",
        &mut terminal,
        &mut pond_app,
        &mut html,
    );

    let mut midgame_device_app = App::new();
    midgame_device_app.reveal_elements_for_preview(&[
        "Aquarium",
        "Blender",
        "Bridge",
        "Dam",
        "Day",
        "Eclipse",
        "Gold",
        "Golem",
        "Greenhouse",
        "Gun",
        "Hourglass",
        "Mirror",
        "Night",
        "Oasis",
        "Oxygen",
        "Plankton",
    ]);
    render_section(
        "midgame device and world sprites in game",
        &mut terminal,
        &mut midgame_device_app,
        &mut html,
    );

    let mut midgame_world_app = App::new();
    midgame_world_app.reveal_elements_for_preview(&[
        "Greenhouse",
        "Night",
        "Oasis",
        "Oxygen",
        "Plankton",
    ]);
    render_section(
        "midgame world overflow sprites in game",
        &mut terminal,
        &mut midgame_world_app,
        &mut html,
    );

    let mut civilization_transport_app = App::new();
    civilization_transport_app.reveal_elements_for_preview(&[
        "Airplane",
        "Bank",
        "Castle",
        "City",
        "Farm",
        "Farmer",
        "Field",
        "Forest",
        "Helicopter",
        "Hospital",
        "Lake",
        "River",
        "Sailboat",
        "Swamp",
        "Train",
        "Village",
    ]);
    render_section(
        "civilization and transport sprites in game",
        &mut terminal,
        &mut civilization_transport_app,
        &mut html,
    );

    let mut landscape_transport_app = App::new();
    landscape_transport_app
        .reveal_elements_for_preview(&["Lake", "River", "Sailboat", "Swamp", "Train", "Village"]);
    render_section(
        "landscape transport overflow sprites in game",
        &mut terminal,
        &mut landscape_transport_app,
        &mut html,
    );

    let mut material_iconic_app = App::new();
    material_iconic_app.reveal_elements_for_preview(&[
        "Isle",
        "Grenade",
        "Horizon",
        "Mountain range",
        "Quicksand",
        "Rust",
        "Sandstone",
        "Sandstorm",
        "Sound",
        "Steel",
        "Perfume",
        "Pyramid",
        "Ring",
        "Robot",
        "Scythe",
        "Sunflower",
    ]);
    render_section(
        "material and iconic sprites in game",
        &mut terminal,
        &mut material_iconic_app,
        &mut html,
    );

    let mut iconic_overflow_app = App::new();
    iconic_overflow_app.reveal_elements_for_preview(&[
        "Sound",
        "Steel",
        "Perfume",
        "Pyramid",
        "Ring",
        "Robot",
        "Scythe",
        "Sunflower",
    ]);
    render_section(
        "iconic overflow sprites in game",
        &mut terminal,
        &mut iconic_overflow_app,
        &mut html,
    );

    let mut common_object_app = App::new();
    common_object_app.reveal_elements_for_preview(&[
        "Skyscraper",
        "Sword",
        "Tide",
        "Water lily",
        "Waterfall",
        "Windmill",
        "Window",
        "Barn",
        "Birdhouse",
        "Dynamite",
        "Eagle",
        "Lamp",
        "Lawn mower",
        "Microscope",
        "Oil",
        "Paint",
    ]);
    render_section(
        "common object and scenery sprites in game",
        &mut terminal,
        &mut common_object_app,
        &mut html,
    );

    let mut common_object_overflow_app = App::new();
    common_object_overflow_app.reveal_elements_for_preview(&[
        "Birdhouse",
        "Dynamite",
        "Eagle",
        "Lamp",
        "Lawn mower",
        "Microscope",
        "Oil",
        "Paint",
    ]);
    render_section(
        "common object overflow sprites in game",
        &mut terminal,
        &mut common_object_overflow_app,
        &mut html,
    );

    let mut fantasy_character_app = App::new();
    fantasy_character_app.reveal_elements_for_preview(&[
        "Angel",
        "Corpse",
        "Cyborg",
        "Fireman",
        "Gardener",
        "Grim reaper",
        "Nerd",
        "Phoenix",
        "Scarecrow",
        "Surfer",
        "Unicorn",
        "Warrior",
        "Wizard",
        "Alligator",
        "Armor",
        "Dragon",
    ]);
    render_section(
        "fantasy and character sprites in game",
        &mut terminal,
        &mut fantasy_character_app,
        &mut html,
    );

    let mut fantasy_overflow_app = App::new();
    fantasy_overflow_app.reveal_elements_for_preview(&[
        "Scarecrow",
        "Surfer",
        "Unicorn",
        "Warrior",
        "Wizard",
        "Alligator",
        "Armor",
        "Dragon",
    ]);
    render_section(
        "fantasy character overflow sprites in game",
        &mut terminal,
        &mut fantasy_overflow_app,
        &mut html,
    );

    let mut early_missing_app = App::new();
    early_missing_app.reveal_elements_for_preview(&[
        "Tobacco",
        "Allergy",
        "Bayonet",
        "Blood",
        "Carbon dioxide",
        "Cold",
        "Double rainbow!",
        "Duck",
        "Electrician",
        "Excalibur",
        "Family",
        "Flamethrower",
        "Hard roe",
        "Hay bale",
        "Hummingbird",
        "Idea",
    ]);
    render_section(
        "early missing object sprites in game",
        &mut terminal,
        &mut early_missing_app,
        &mut html,
    );

    let mut early_missing_overflow_app = App::new();
    early_missing_overflow_app.reveal_elements_for_preview(&[
        "Double rainbow!",
        "Duck",
        "Electrician",
        "Excalibur",
        "Family",
        "Flamethrower",
        "Hard roe",
        "Hay bale",
        "Hummingbird",
        "Idea",
    ]);
    render_section(
        "early missing overflow sprites in game",
        &mut terminal,
        &mut early_missing_overflow_app,
        &mut html,
    );

    let mut light_bird_object_app = App::new();
    light_bird_object_app.reveal_elements_for_preview(&[
        "Light",
        "Lightsaber",
        "Love",
        "Music",
        "Nest",
        "Omelette",
        "Ostrich",
        "Owl",
        "Ozone",
        "Peacock",
        "Prism",
        "Ruins",
        "Safe",
        "Safety glasses",
        "Seagull",
        "Sickness",
    ]);
    render_section(
        "light bird and object sprites in game",
        &mut terminal,
        &mut light_bird_object_app,
        &mut html,
    );

    let mut light_bird_overflow_app = App::new();
    light_bird_overflow_app.reveal_elements_for_preview(&[
        "Ozone",
        "Peacock",
        "Prism",
        "Ruins",
        "Safe",
        "Safety glasses",
        "Seagull",
        "Sickness",
    ]);
    render_section(
        "light bird overflow sprites in game",
        &mut terminal,
        &mut light_bird_overflow_app,
        &mut html,
    );

    let mut accessory_device_app = App::new();
    accessory_device_app.reveal_elements_for_preview(&[
        "Sunglasses",
        "Swim goggles",
        "Taser",
        "The one ring",
        "Toucan",
        "Turtle",
        "Twilight",
        "Water gun",
        "Wind turbine",
        "Alarm clock",
        "Black hole",
        "Bone",
        "Bonsai tree",
        "Caviar",
        "Chameleon",
        "Charcoal",
    ]);
    render_section(
        "accessory device and nature sprites in game",
        &mut terminal,
        &mut accessory_device_app,
        &mut html,
    );

    let mut accessory_overflow_app = App::new();
    accessory_overflow_app.reveal_elements_for_preview(&[
        "Wind turbine",
        "Alarm clock",
        "Black hole",
        "Bone",
        "Bonsai tree",
        "Caviar",
        "Chameleon",
        "Charcoal",
    ]);
    render_section(
        "accessory device overflow sprites in game",
        &mut terminal,
        &mut accessory_overflow_app,
        &mut html,
    );

    let mut animal_tech_app = App::new();
    animal_tech_app.reveal_elements_for_preview(&[
        "Chicken",
        "Christmas tree",
        "Computer",
        "Constellation",
        "Crow",
        "Cuckoo",
        "Dinosaur",
        "Drone",
        "Dry ice",
        "Duckling",
        "Egg timer",
        "Engineer",
        "Family tree",
        "Fire extinguisher",
        "Flashlight",
        "Frankenstein",
    ]);
    render_section(
        "animal tech and monster sprites in game",
        &mut terminal,
        &mut animal_tech_app,
        &mut html,
    );

    let mut animal_tech_overflow_app = App::new();
    animal_tech_overflow_app.reveal_elements_for_preview(&[
        "Dry ice",
        "Duckling",
        "Egg timer",
        "Engineer",
        "Family tree",
        "Fire extinguisher",
        "Flashlight",
        "Frankenstein",
    ]);
    render_section(
        "animal tech overflow sprites in game",
        &mut terminal,
        &mut animal_tech_overflow_app,
        &mut html,
    );

    let mut food_grave_magic_app = App::new();
    food_grave_magic_app.reveal_elements_for_preview(&[
        "Fridge",
        "Fruit",
        "Grave",
        "Harp",
        "Herb",
        "Jedi",
        "Lava lamp",
        "Leaf",
        "Lighthouse",
        "Livestock",
        "Mayonnaise",
        "Monarch",
        "Mummy",
        "Narwhal",
        "Oil lamp",
        "Optical fiber",
    ]);
    render_section(
        "food grave and magic sprites in game",
        &mut terminal,
        &mut food_grave_magic_app,
        &mut html,
    );

    let mut food_grave_overflow_app = App::new();
    food_grave_overflow_app.reveal_elements_for_preview(&[
        "Lighthouse",
        "Livestock",
        "Mayonnaise",
        "Monarch",
        "Mummy",
        "Narwhal",
        "Oil lamp",
        "Optical fiber",
    ]);
    render_section(
        "food grave overflow sprites in game",
        &mut terminal,
        &mut food_grave_overflow_app,
        &mut html,
    );

    let mut shore_tool_sky_app = App::new();
    shore_tool_sky_app.reveal_elements_for_preview(&[
        "Palm",
        "Pegasus",
        "Pigeon",
        "Pilot",
        "Pitchfork",
        "Rose",
        "Seaplane",
        "Seasickness",
        "Sewing machine",
        "Shark",
        "Shuriken",
        "Skeleton",
        "Smog",
        "Soap",
        "Soda",
        "Solar cell",
    ]);
    render_section(
        "shore tool and sky sprites in game",
        &mut terminal,
        &mut shore_tool_sky_app,
        &mut html,
    );

    let mut shore_tool_sky_overflow_app = App::new();
    shore_tool_sky_overflow_app.reveal_elements_for_preview(&[
        "Seasickness",
        "Sewing machine",
        "Shark",
        "Shuriken",
        "Skeleton",
        "Smog",
        "Soap",
        "Soda",
        "Solar cell",
    ]);
    render_section(
        "shore tool overflow sprites in game",
        &mut terminal,
        &mut shore_tool_sky_overflow_app,
        &mut html,
    );

    let mut space_time_undead_weather_app = App::new();
    space_time_undead_weather_app.reveal_elements_for_preview(&[
        "Spaceship",
        "Starfish",
        "Statue",
        "Steam engine",
        "Sundial",
        "Super nova",
        "Swimmer",
        "Thread",
        "Treehouse",
        "Umbrella",
        "Vampire",
        "Vulture",
        "Watch",
        "Zombie",
        "Acid rain",
        "Alcohol",
    ]);
    render_section(
        "space time undead and weather sprites in game",
        &mut terminal,
        &mut space_time_undead_weather_app,
        &mut html,
    );

    let mut space_time_undead_weather_overflow_app = App::new();
    space_time_undead_weather_overflow_app.reveal_elements_for_preview(&[
        "Treehouse",
        "Umbrella",
        "Vampire",
        "Vulture",
        "Watch",
        "Zombie",
        "Acid rain",
        "Alcohol",
    ]);
    render_section(
        "space time overflow sprites in game",
        &mut terminal,
        &mut space_time_undead_weather_overflow_app,
        &mut html,
    );

    let mut alien_winter_food_character_app = App::new();
    alien_winter_food_character_app.reveal_elements_for_preview(&[
        "Alien",
        "Antarctica",
        "Avalanche",
        "Blizzard",
        "Broom",
        "Bulletproof vest",
        "Camel",
        "Campfire",
        "Chicken soup",
        "Chicken wing",
        "Coconut",
        "Coffin",
        "Crown",
        "Darth vader",
        "Doctor",
        "Electric eel",
    ]);
    render_section(
        "alien winter food and character sprites in game",
        &mut terminal,
        &mut alien_winter_food_character_app,
        &mut html,
    );

    let mut alien_winter_food_character_overflow_app = App::new();
    alien_winter_food_character_overflow_app.reveal_elements_for_preview(&[
        "Chicken soup",
        "Chicken wing",
        "Coconut",
        "Coffin",
        "Crown",
        "Darth vader",
        "Doctor",
        "Electric eel",
    ]);
    render_section(
        "alien winter overflow sprites in game",
        &mut terminal,
        &mut alien_winter_food_character_overflow_app,
        &mut html,
    );

    let mut material_nature_creature_app = App::new();
    material_nature_creature_app.reveal_elements_for_preview(&[
        "Fabric",
        "Fence",
        "Flour",
        "Flute",
        "Fossil",
        "Fountain",
        "Fruit tree",
        "Glacier",
        "Gnome",
        "Goat",
        "Godzilla",
        "Gravestone",
        "Graveyard",
        "Hail",
        "Iceberg",
        "Igloo",
    ]);
    render_section(
        "material nature and creature sprites in game",
        &mut terminal,
        &mut material_nature_creature_app,
        &mut html,
    );

    let mut material_nature_creature_overflow_app = App::new();
    material_nature_creature_overflow_app.reveal_elements_for_preview(&[
        "Fruit tree",
        "Glacier",
        "Gnome",
        "Goat",
        "Godzilla",
        "Gravestone",
        "Graveyard",
        "Hail",
        "Iceberg",
        "Igloo",
    ]);
    render_section(
        "material nature overflow sprites in game",
        &mut terminal,
        &mut material_nature_creature_overflow_app,
        &mut html,
    );

    let mut la2_nature_app = App::new();
    la2_nature_app.handle_event(key(KeyCode::Tab));
    la2_nature_app.reveal_elements_for_preview(&["Lightning"]);
    render_section(
        "little alchemy 2 lightning sprite in game",
        &mut terminal,
        &mut la2_nature_app,
        &mut html,
    );

    html.push_str("</body></html>\n");
    std::fs::create_dir_all("output").expect("output dir");
    std::fs::write("output/tui-preview.html", html).expect("html preview");
}

fn render_section(
    name: &str,
    terminal: &mut Terminal<TestBackend>,
    app: &mut App,
    html: &mut String,
) {
    terminal.draw(|frame| app.render(frame)).expect("draw");
    println!("--- {name} ---");
    for line in buffer_lines(terminal.backend().buffer()) {
        println!("{line}");
    }
    html.push_str("<section><h2>");
    html_escape_into(name, html);
    html.push_str("</h2><pre>");
    buffer_to_html(terminal.backend().buffer(), html);
    html.push_str("</pre></section>");
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn buffer_lines(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
    let area = buffer.area;
    let mut lines = Vec::new();
    for y in 0..area.height {
        let mut line = String::new();
        for x in 0..area.width {
            line.push_str(buffer[(area.x + x, area.y + y)].symbol());
        }
        lines.push(line);
    }
    lines
}

fn buffer_to_html(buffer: &ratatui::buffer::Buffer, html: &mut String) {
    let area = buffer.area;
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(area.x + x, area.y + y)];
            let fg = css_color(cell.fg);
            let bg = css_color(cell.bg);
            let _ = write!(html, "<span style=\"color:{fg};background:{bg}\">");
            html_escape_into(cell.symbol(), html);
            html.push_str("</span>");
        }
        html.push('\n');
    }
}

fn html_header() -> String {
    r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
body { margin: 0; padding: 24px; background: #071012; color: #eee; }
section { margin: 0 0 28px; }
h2 { font: 700 15px system-ui, sans-serif; color: #e6c46f; margin: 0 0 10px; }
pre {
  display: inline-block;
  margin: 0;
  padding: 16px;
  line-height: 1;
  font-size: 14px;
  font-family: "JetBrains Mono", "Menlo", "SFMono-Regular", monospace;
  background: #101d1f;
}
span { display: inline-block; width: 8px; height: 14px; }
</style>
</head>
<body>
"#
    .to_string()
}

fn html_escape_into(value: &str, html: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            ' ' => html.push_str("&nbsp;"),
            _ => html.push(ch),
        }
    }
}

fn css_color(color: Color) -> String {
    match color {
        Color::Reset => "transparent".to_string(),
        Color::Black => "#000000".to_string(),
        Color::Red => "#ff5555".to_string(),
        Color::Green => "#55ff55".to_string(),
        Color::Yellow => "#ffff55".to_string(),
        Color::Blue => "#5555ff".to_string(),
        Color::Magenta => "#ff55ff".to_string(),
        Color::Cyan => "#55ffff".to_string(),
        Color::Gray => "#aaaaaa".to_string(),
        Color::DarkGray => "#555555".to_string(),
        Color::LightRed => "#ff7777".to_string(),
        Color::LightGreen => "#77ff77".to_string(),
        Color::LightYellow => "#ffff77".to_string(),
        Color::LightBlue => "#7777ff".to_string(),
        Color::LightMagenta => "#ff77ff".to_string(),
        Color::LightCyan => "#77ffff".to_string(),
        Color::White => "#ffffff".to_string(),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(index) => format!("rgb({index},{index},{index})"),
    }
}

fn find_text_position(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, line)| {
        line.find(needle).map(|byte_column| {
            let column = line[..byte_column].chars().count();
            (column as u16, row as u16)
        })
    })
}
