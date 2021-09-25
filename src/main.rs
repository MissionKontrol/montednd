use rand::Rng;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
struct CharacterStruct {
    name: String,
    hit_points: u8,
    armour_class: u8,
    to_hit: u8,
    actions_per_round: u8,
    damage: u8,
}

struct AttackResult {
    attack_roll: u8,
    damage_roll: u8,
    attack_result: ActionResultType,
}

#[derive(Default)]
struct DamageResult{
    remaining_hit_points: u8,
    target_state: HealthState,
}

#[derive(Debug, Clone, PartialEq)]
enum HealthState{
    DEAD,
    KO,
    NOMINAL,
}

impl Default for HealthState {
    fn default() -> Self {
        HealthState::NOMINAL
    }
}


#[derive(Default)]
struct BattleData {
    battle_data: Vec<TurnResult>,
}

#[derive(Default)]
struct TurnResult {
    action_results: Vec<ActionResult>,
}

struct ActionResult  {
    actor: String,
    target: String,
    action_number: u16,
    action_type: ActionType,
    action_roll: u8,
    action_result: ActionResultType,
    action_damage: u16
}


#[derive(Default)]
struct BattleResult{
    turn_result: Vec<TurnResult>,
    battle_order: BattleOrder,
}

#[derive(Default, Clone)]
struct BattleOrder {
    initative_roll: u8,
    character_state: HealthState,
    character: CharacterStruct,
}

// impl Iterator for BattleOrder {
//     type Item = BattleOrder;

//     fn next(&mut self) -> Option<Self::Item> {
//         Some(*self)
//     }
// }

enum ActionResultType {
    CritFail,
    Fail,
    Miss,
    Hit,
    CritHit,
}

enum ActionType {
    Attack,
    Dodge,
    Cast,
    Initative,
}

fn battle( players: &Vec<CharacterStruct>, num_iterations: u8) {
    let battle_order = make_battle_order(players);
    let battle_result = run_battle(battle_order);
}

fn make_battle_order(players: &Vec<CharacterStruct>) -> Vec<BattleOrder> {
    let mut rng = rand::thread_rng();
    let mut battle_order = Vec::new();
    
    for player in players {
        let player_order = player.clone();
        let initative_roll = rng.gen_range(1..20);
        let order = BattleOrder {
            initative_roll: initative_roll,
            character: player_order,
            character_state: HealthState::NOMINAL
        };
        battle_order.push(order);
    }
    battle_order.sort_by(|a,b| a.initative_roll.cmp(&b.initative_roll) );
    battle_order
}

fn run_battle(battle_order: Vec<BattleOrder>) -> BattleResult {
    let mut battle_result: BattleResult = Default::default();
    let winning_result = false;

    while !winning_result {
        let turn_result = run_battle_turn(&battle_order);
        battle_result.turn_result.push(turn_result);
    }
    battle_result
}


fn run_battle_turn(battle_order: &Vec<BattleOrder>) -> TurnResult{
    let mut turn_result: TurnResult = Default::default();
    let combatants = battle_order.clone();

    for i in 0..battle_order.len() {
        if battle_order[i].character_state == HealthState::NOMINAL {
            let target = battle_order[i].character.select_target(&combatants);
            let attack_result = melee_attack(battle_order[i].character.to_hit, battle_order[target].character.armour_class, battle_order[i].character.damage);
            println!("{} Attacks with {} for {} damage.",battle_order[i].character.name, attack_result.attack_roll, attack_result.damage_roll);        
            let damage_done = battle_order[target].character.resolve_damage(attack_result.damage_roll);
            println!("  {:?} is {:?}  remaining hit point: {:?}", battle_order[target].character.name, damage_done.target_state, damage_done.remaining_hit_points);


            let action_result =  ActionResult {
                actor: battle_order[i].character.name.clone(),
                target: battle_order[i].character.name.clone(),
                action_number: i as u16,                           // tots not gonna work so fix?
                action_type: ActionType::Attack,
                action_roll: attack_result.attack_roll,
                action_result: attack_result.attack_result,
                action_damage: attack_result.damage_roll as u16,
            };
            turn_result.action_results.push(action_result);
        }
    }
    turn_result
}

impl CharacterStruct {
    fn select_target(&self, combatant_list: &Vec<BattleOrder>) -> usize {
        for i in  0..combatant_list.len() {
            if combatant_list[i].character.name != self.name && combatant_list[i].character_state == HealthState::NOMINAL {
                println!("{} chooses {}", self.name, combatant_list[i].character.name);
                return i;
            }
        }
        panic!("failed to find target");
    }

    fn resolve_damage(&self, damage: u8) -> DamageResult {
        let mut result: DamageResult = Default::default();
        let remaining_hit_points = self.hit_points - damage;
    
        match remaining_hit_points as i8 - damage as i8 {
            d if d < 0  => result = DamageResult{remaining_hit_points: 0,target_state:HealthState::DEAD},
            0 => result = DamageResult{remaining_hit_points: 0,target_state:HealthState::KO},
            d if d > 0 => result = DamageResult{remaining_hit_points: d as u8,target_state:HealthState::NOMINAL},
            _ => panic!("yup we got here...resolve_damage(damage: u8, hit_points: u8)"),
        }
        result
    }
}

fn melee_attack(to_hit: u8, armour_class: u8, damage: u8) -> AttackResult {
    let mut rng = rand::thread_rng();
    let damage_done = 0;

    let mut result = AttackResult{
        attack_roll: 0,
        damage_roll: 0,
        attack_result: ActionResultType::Miss,
    };
    
    result.attack_roll  = rng.gen_range(1..to_hit);
    if  result.attack_roll > armour_class {
        result.damage_roll = rng.gen_range(1..damage);
        result.attack_result = ActionResultType::Hit;
    }
    result
}

fn main() {
    let player1 = CharacterStruct {
        name: String::from("Player 1"),
        hit_points: 10,
        armour_class: 1,
        to_hit: 12,
        actions_per_round: 1,
        damage: 6,  
    };

    let player2 = CharacterStruct {
        name: String::from("Player 2"),
        hit_points: 10,
        armour_class: 1,
        to_hit: 12,
        actions_per_round: 1,
        damage: 6,  
    };

    let mut player_vec = Vec::new(); 
    player_vec.push(player1);
    player_vec.push(player2);
    
    battle(&player_vec, 10);
}