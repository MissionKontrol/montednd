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

struct AttackResult {
    attack_roll: u8,
    damage_roll: u8,
}

struct DamageResult{
    remaining_hit_points: u8,
    target_state: HealthState,
}

#[derive(Debug)]
enum HealthState{
    DEAD,
    KO,
    NOMINAL,
}

fn battle( players: &Vec<CharacterStruct>, num_turns: u8) {
    let mut rng = rand::thread_rng();
    let turn_order: [bool;10]= [rand::random(); 10];

    let mut p1 = players[0].stats;
    let mut p2 = players[1].stats;

    for i in 0..num_turns {
        let attack_result = melee_attack(p1.to_hit, p2.armour_class, p1.damage);
        println!("{} Attacks with {} for {} damage.",players[0].name,attack_result.attack_roll, attack_result.damage_roll);        
        let damage_done = resolve_damage(attack_result.damage_roll, p2.hit_points);
        println!("  {:?} is {:?}  remaining hit point: {:?}", players[1].name, damage_done.target_state, damage_done.remaining_hit_points);
        p2.hit_points=damage_done.remaining_hit_points;

        let attack_result = melee_attack(p1.to_hit, p2.armour_class, p1.damage);
        println!("{} Attacks with {} for {} damage.",players[1].name,attack_result.attack_roll, attack_result.damage_roll);        
        let damage_done = resolve_damage(attack_result.damage_roll, p1.hit_points);
        println!("  {:?} is {:?}  remaining hit point: {:?}", players[0].name, damage_done.target_state, damage_done.remaining_hit_points);
        p1.hit_points=damage_done.remaining_hit_points;
    }    
}

fn battle_turn(){
}

fn resolve_damage(damage: u8, hit_points: u8) -> DamageResult {
    let mut result = DamageResult{remaining_hit_points:hit_points,target_state:HealthState::NOMINAL};

    match hit_points as i8 - damage as i8 {
        d if d < 0  => result = DamageResult{remaining_hit_points: 0,target_state:HealthState::DEAD},
        0 => result = DamageResult{remaining_hit_points: 0,target_state:HealthState::KO},
        d if d > 0 => result = DamageResult{remaining_hit_points: d as u8,target_state:HealthState::NOMINAL},
        _ => panic!("yup we got here...resolve_damage(damage: u8, hit_points: u8)"),
    }
    result
}

fn melee_attack(to_hit: u8, armour_class: u8, damage: u8) -> AttackResult {
    let mut rng = rand::thread_rng();
    let damage_done = 0;

    let mut result = AttackResult{
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