use rand::Rng;

#[derive(Default, Debug, Clone)]
struct CharacterStruct {
    name: String,
    stats: CharacterCapabilities,
}

#[derive(Default, Debug, Copy, Clone)]
struct CharacterCapabilities {
    hit_points: u8,
    armour_class: u8,
    to_hit: u8,
    actions_per_round: u8,
    damage: u8,
}

struct attack_result {
    attack_roll: u8,
    damage_roll: u8,
}

fn battle( players: &Vec<CharacterStruct>, num_turns: u8) {
    let mut rng = rand::thread_rng();
    let turn_order: [bool;10]= [rand::random(); 10];

    let p1 = players[0].stats;
    let p2 = players[1].stats;

    for i in 0..num_turns {
        let attack_result = melee_attack(p1.to_hit, p2.armour_class, p1.damage);
        println!("{} Attacks with {} for {} damage.",players[0].name,attack_result.attack_roll, attack_result.damage_roll);        
        let attack_result = melee_attack(p1.to_hit, p2.armour_class, p1.damage);
        println!("{} Attacks with {} for {} damage.",players[1].name,attack_result.attack_roll, attack_result.damage_roll);        
    }    
}

fn battle_turn(){
}

fn melee_attack(to_hit: u8, armour_class: u8, damage: u8) -> attack_result {
    let mut rng = rand::thread_rng();
    let damage_done = 0;

    let mut result = attack_result{
        attack_roll: 0,
        damage_roll: 0,
    };
    
    result.attack_roll  = rng.gen_range(1..to_hit);
    if  result.attack_roll > armour_class {
        result.damage_roll = rng.gen_range(1..damage);
    }
    result
}

fn main() {
    let pstats = CharacterCapabilities{
        hit_points: 10,
        armour_class: 1,
        to_hit: 12,
        actions_per_round: 1,
        damage: 6,        
    };

    let player1 = CharacterStruct {
        name: String::from("Player 1"),
        stats: pstats,
    };

    let player2 = CharacterStruct {
        name: String::from("Player 2"),
        stats: pstats,
    };

    let mut player_vec = Vec::new(); 
    player_vec.push(player1);
    player_vec.push(player2);
    
    battle(&player_vec, 10);
}