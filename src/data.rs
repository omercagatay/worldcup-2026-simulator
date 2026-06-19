use std::collections::{HashMap, HashSet};

pub const BASE: f64 = 1.35;
pub const D_DIV: f64 = 1600.0;
pub const HOME_ADV: f64 = 80.0;
pub const ET_FACTOR: f64 = 0.5;
pub const PEN_DAMP: f64 = 0.4;

pub fn elo() -> Vec<(&'static str, f64)> {
    vec![
        ("Argentina", 2055.0),
        ("France", 2035.0),
        ("Spain", 2005.0),
        ("England", 1995.0),
        ("Portugal", 1980.0),
        ("Netherlands", 1965.0),
        ("Brazil", 1955.0),
        ("Belgium", 1930.0),
        ("Germany", 1920.0),
        ("Croatia", 1885.0),
        ("Uruguay", 1870.0),
        ("Austria", 1805.0),
        ("Colombia", 1845.0),
        ("Morocco", 1840.0),
        ("Japan", 1815.0),
        ("Mexico", 1815.0),
        ("United States", 1795.0),
        ("Iran", 1785.0),
        ("Switzerland", 1775.0),
        ("Senegal", 1760.0),
        ("Ecuador", 1760.0),
        ("Australia", 1755.0),
        ("Norway", 1750.0),
        ("Turkey", 1750.0),
        ("Sweden", 1745.0),
        ("South Korea", 1745.0),
        ("Ivory Coast", 1735.0),
        ("Czech Republic", 1715.0),
        ("Scotland", 1700.0),
        ("Tunisia", 1705.0),
        ("Paraguay", 1705.0),
        ("Algeria", 1695.0),
        ("Canada", 1695.0),
        ("Bosnia and Herzegovina", 1695.0),
        ("Saudi Arabia", 1675.0),
        ("Egypt", 1665.0),
        ("Ghana", 1665.0),
        ("DR Congo", 1625.0),
        ("Qatar", 1625.0),
        ("Panama", 1615.0),
        ("Uzbekistan", 1605.0),
        ("South Africa", 1605.0),
        ("Iraq", 1600.0),
        ("Haiti", 1510.0),
        ("Jordan", 1505.0),
        ("Curaçao", 1495.0),
        ("Cape Verde", 1485.0),
        ("New Zealand", 1435.0),
    ]
}

pub fn groups() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "A",
            vec!["Mexico", "South Korea", "Czech Republic", "South Africa"],
        ),
        (
            "B",
            vec!["Switzerland", "Canada", "Qatar", "Bosnia and Herzegovina"],
        ),
        ("C", vec!["Scotland", "Morocco", "Brazil", "Haiti"]),
        (
            "D",
            vec!["United States", "Australia", "Turkey", "Paraguay"],
        ),
        ("E", vec!["Germany", "Ivory Coast", "Ecuador", "Curaçao"]),
        ("F", vec!["Sweden", "Japan", "Netherlands", "Tunisia"]),
        ("G", vec!["New Zealand", "Iran", "Belgium", "Egypt"]),
        ("H", vec!["Uruguay", "Saudi Arabia", "Spain", "Cape Verde"]),
        ("I", vec!["Norway", "France", "Senegal", "Iraq"]),
        ("J", vec!["Argentina", "Austria", "Jordan", "Algeria"]),
        ("K", vec!["Portugal", "DR Congo", "Uzbekistan", "Colombia"]),
        ("L", vec!["England", "Croatia", "Ghana", "Panama"]),
    ]
}

pub fn played() -> Vec<PlayedMatch> {
    vec![
        PlayedMatch {
            group: "A",
            a: "Mexico",
            sa: 2,
            b: "South Africa",
            sb: 0,
        },
        PlayedMatch {
            group: "A",
            a: "South Korea",
            sa: 2,
            b: "Czech Republic",
            sb: 1,
        },
        PlayedMatch {
            group: "B",
            a: "Canada",
            sa: 1,
            b: "Bosnia and Herzegovina",
            sb: 1,
        },
        PlayedMatch {
            group: "B",
            a: "Qatar",
            sa: 1,
            b: "Switzerland",
            sb: 1,
        },
        PlayedMatch {
            group: "C",
            a: "Brazil",
            sa: 1,
            b: "Morocco",
            sb: 1,
        },
        PlayedMatch {
            group: "C",
            a: "Haiti",
            sa: 0,
            b: "Scotland",
            sb: 1,
        },
        PlayedMatch {
            group: "D",
            a: "United States",
            sa: 4,
            b: "Paraguay",
            sb: 1,
        },
        PlayedMatch {
            group: "D",
            a: "Australia",
            sa: 2,
            b: "Turkey",
            sb: 0,
        },
        PlayedMatch {
            group: "E",
            a: "Germany",
            sa: 7,
            b: "Curaçao",
            sb: 1,
        },
        PlayedMatch {
            group: "E",
            a: "Ivory Coast",
            sa: 1,
            b: "Ecuador",
            sb: 0,
        },
        PlayedMatch {
            group: "F",
            a: "Netherlands",
            sa: 2,
            b: "Japan",
            sb: 2,
        },
        PlayedMatch {
            group: "F",
            a: "Sweden",
            sa: 5,
            b: "Tunisia",
            sb: 1,
        },
        PlayedMatch {
            group: "G",
            a: "Belgium",
            sa: 1,
            b: "Egypt",
            sb: 1,
        },
        PlayedMatch {
            group: "G",
            a: "Iran",
            sa: 2,
            b: "New Zealand",
            sb: 2,
        },
        PlayedMatch {
            group: "H",
            a: "Spain",
            sa: 0,
            b: "Cape Verde",
            sb: 0,
        },
        PlayedMatch {
            group: "H",
            a: "Saudi Arabia",
            sa: 1,
            b: "Uruguay",
            sb: 1,
        },
        PlayedMatch {
            group: "I",
            a: "France",
            sa: 3,
            b: "Senegal",
            sb: 1,
        },
        PlayedMatch {
            group: "I",
            a: "Iraq",
            sa: 1,
            b: "Norway",
            sb: 4,
        },
        PlayedMatch {
            group: "J",
            a: "Argentina",
            sa: 3,
            b: "Algeria",
            sb: 0,
        },
        PlayedMatch {
            group: "J",
            a: "Austria",
            sa: 3,
            b: "Jordan",
            sb: 1,
        },
    ]
}

pub fn hosts() -> HashSet<&'static str> {
    ["Mexico", "Canada", "United States"].into_iter().collect()
}

pub fn third_place_slots() -> HashMap<u32, Vec<&'static str>> {
    HashMap::from([
        (74u32, vec!["A", "B", "C", "D", "F"]),
        (77, vec!["C", "D", "F", "G", "H"]),
        (79, vec!["C", "E", "F", "H", "I"]),
        (80, vec!["E", "H", "I", "J", "K"]),
        (81, vec!["B", "E", "F", "I", "J"]),
        (82, vec!["A", "E", "H", "I", "J"]),
        (85, vec!["E", "F", "G", "I", "J"]),
        (87, vec!["D", "E", "I", "J", "L"]),
    ])
}

pub fn r32() -> Vec<(u32, &'static str, &'static str)> {
    vec![
        (73, "2A", "2B"),
        (74, "1E", "3"),
        (75, "1F", "2C"),
        (76, "1C", "2F"),
        (77, "1I", "3"),
        (78, "2E", "2I"),
        (79, "1A", "3"),
        (80, "1L", "3"),
        (81, "1D", "3"),
        (82, "1G", "3"),
        (83, "2K", "2L"),
        (84, "1H", "2J"),
        (85, "1B", "3"),
        (86, "1J", "2H"),
        (87, "1K", "3"),
        (88, "2D", "2G"),
    ]
}

pub fn r16() -> Vec<(u32, u32, u32)> {
    vec![
        (89, 74, 77),
        (90, 73, 75),
        (91, 76, 78),
        (92, 79, 80),
        (93, 83, 84),
        (94, 81, 82),
        (95, 86, 88),
        (96, 85, 87),
    ]
}

pub fn qf() -> Vec<(u32, u32, u32)> {
    vec![(97, 89, 90), (98, 93, 94), (99, 91, 92), (100, 95, 96)]
}

pub fn sf() -> Vec<(u32, u32, u32)> {
    vec![(101, 97, 98), (102, 99, 100)]
}

pub const FINAL: u32 = 104;
#[allow(dead_code)]
pub const THIRD_PLACE_MATCH: u32 = 103;

#[derive(Debug, Clone)]
pub struct PlayedMatch {
    pub group: &'static str,
    pub a: &'static str,
    pub sa: u16,
    pub b: &'static str,
    pub sb: u16,
}
