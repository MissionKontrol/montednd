use rand::Rng;

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
    attack_result: ActionResult,
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

#[derive(Default)]
struct TurnResult {
    winner: CharacterStruct,
    battle_data: Vec<BattleData>,
}

struct BattleData {
    actor: String,
    action_number: u16,
    action_type: ActionType,
    action_roll: u8,
    action_result: ActionResult,
    action_damage: u16
}

#[derive(Default, Clone)]
struct BattleOrder {
    initative_roll: u8,
    character: CharacterStruct,
}


enum ActionResult {
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

fn battle( players: &Vec<CharacterStruct>, num_turns: u8) {
    let battle_order = make_battle_order(players);
    for i in 1..num_turns {
        run_battle_turn(battle_order.clone());
    }   
}

fn make_battle_order(players: &Vec<CharacterStruct>) -> Vec<BattleOrder> {
    let mut rng = rand::thread_rng();
    let mut battle_order = Vec::new();

    for player in players {
        let player_order = BattleOrder {
            character: player.clone(),
            initative_roll: rng.gen_range(1..20),
        };
        battle_order.push(player_order);
    }
    battle_order
}

fn run_battle_turn(mut battle_order: Vec<BattleOrder>) -> TurnResult{
    let mut turn_result: TurnResult = Default::default();

    let x = battle_order.clone();
    battle_order.sort_by(|a, b| b.initative_roll.cmp(&a.initative_roll));
    let mut i = 1;
    for action in battle_order {
        if action.character.hit_points > 0 {
            let attack_target = action.select_target(x.clone());
            let attack_result = melee_attack(action.character.to_hit, attack_target.character.armour_class, action.character.damage);
            println!("{} Attacks with {} for {} damage.",action.character.name, attack_result.attack_roll, attack_result.damage_roll);        
            let damage_done = resolve_damage(attack_result.damage_roll, attack_target.character.hit_points);
            println!("  {:?} is {:?}  remaining hit point: {:?}", attack_target.character.name, damage_done.target_state, damage_done.remaining_hit_points);

            let battle_data = BattleData {
                actor: action.character.name,
                action_number: i,
                action_type: ActionType::Attack,
                action_roll: attack_result.attack_roll,
                action_result: attack_result.attack_result,
                action_damage: attack_result.damage_roll as u16,
            };
            i += 1;
            turn_result.battle_data.push(battle_data);
        }
        else {break}
    }
    turn_result
}

impl BattleOrder { 
    fn select_target(&self, battle_order: Vec<BattleOrder>) -> BattleOrder {
        for player in battle_order {
            if player.character.name != self.character.name {
                return player.clone();
            }
        }
        panic!("failed to find target");
    }
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
        attack_result: ActionResult::Miss,
    };
    
    result.attack_roll  = rng.gen_range(1..to_hit);
    if  result.attack_roll > armour_class {
        result.damage_roll = rng.gen_range(1..damage);
        result.attack_result = ActionResult::Hit;
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