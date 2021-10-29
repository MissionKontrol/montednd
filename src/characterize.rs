use serde::{Deserialize};
use serde_json::Error;
use std::fs;
use std::{fmt};

// fn main() {
//     let contents: String = fs::read_to_string("./input/temp.json")
//         .expect("Something went wrong reading the file");

//     let foo = &contents;
//     let deserialized: Result<CharacterStruct,Error> = serde_json::from_str(foo);
//     if let Err(error) = deserialized {
//         println!("ERROR: {}", error);
//     }
//     else {println!("deserialized = {:?}", deserialized);} 
// }

fn get_players() -> Vec<CharacterStruct> {
    let player1 = CharacterStruct {
        name: String::from("Hero"),
        armour_class: 12,
        to_hit: 20,
        weapon: "1d6",
        actions_per_round: 1,
        damage: 6,  
        team: Team::Heros,
        hs2: HealthState::Alive(10),
    };

    let player2 = CharacterStruct {
        name: String::from("Villan-A"),
        armour_class: 10,
        to_hit: 20,
        weapon: "1d4",
        actions_per_round: 1,
        damage: 4,  
        team: Team::Villains,
        hs2: HealthState::Alive(6),
    };

    let player3 = CharacterStruct {
        name: String::from("Villan-B"),
        armour_class: 10,
        to_hit: 20,
        weapon: "1d4",
        actions_per_round: 1,
        damage: 4,  
        team: Team::Villains,
        hs2: HealthState::Alive(6),
    };

    // let player4 = CharacterStruct {
    //     name: String::from("Hero-B"),
    //     armour_class: 14,
    //     to_hit: 20,
    //     weapon: "1d4",
    //     actions_per_round: 1,
    //     damage: 4,  
    //     team: Team::Heros,
    //     hs2: HealthState::Alive(7),
    // };

    let player_vec = vec!(player1,player2,player3);
    player_vec
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct CharacterStruct {
    name: String,
    // hit_points: u8,
    armour_class: u8,
    to_hit: u8,
    weapon: String,
    actions_per_round: u8,
    damage: u8,
    team: Team,
    hs2: HealthState,
}


#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Debug)]
enum Team {
    Heros,
    Villains,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Team::Heros => write!(f,"Heros"),
            Team::Villains => write!(f,"Villans"),
        }
    }
}

impl Default for Team {
    fn default() -> Self { Team::Heros }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialOrd, Eq, Ord, PartialEq)]
enum HealthState {
    Dead,
    Ko,
    Alive(u16),
}

impl Default for HealthState {
    fn default() -> Self {
        HealthState::Dead        
    }
}

impl HealthState {
    fn update_health_state(self, modifier: i16) -> Self {
        const KO_THRESHOLD: i16 = -10;

        match self {
            HealthState::Dead => HealthState::Dead,
            HealthState::Ko => if (0 - modifier) < KO_THRESHOLD { HealthState::Dead } else { HealthState::Ko },
            HealthState::Alive(hit_points) => self.get_new_state_maybe(hit_points as i16 - modifier),
        }
    }
    
    fn get_new_state_maybe(self, integer_state: i16) -> HealthState {
        match integer_state {
            x if x < 0 =>  HealthState::Dead,
            x if x == 0 =>  HealthState::Ko,
            _ =>  HealthState::Alive(integer_state as u16),
        }
    }
}