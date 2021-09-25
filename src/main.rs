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
    attack_result: ActionResultType,
}

#[derive(Default, Clone)]
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


#[derive(Default, Clone)]
struct TurnResult {
    action_results: Vec<ActionResult>,
}

#[derive(Clone)]
struct ActionResult  {
    actor: String,
    target: String,
    action_number: u16,
    action_type: ActionType,
    action_roll: u8,
    action_result: ActionResultType,
    action_damage: u16,
    damage_done: DamageResult,
}


#[derive(Default, Clone)]
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

#[derive(Clone)]
enum ActionResultType {
    _CritFail,
    _Fail,
    Miss,
    Hit,
    _CritHit,
}

#[derive(Clone)]
enum ActionType {
    Attack,
    _Dodge,
    _Cast,
    _Initative,
}

fn battle( players: &Vec<CharacterStruct>, num_iterations: u8) {
    let battle_order = make_battle_order(players);
    let mut battle_collection:Vec<BattleResult> = Vec::new();

    for _ in 0..num_iterations {
        let battle_result = run_battle(battle_order.clone());
        battle_collection.push(battle_result.clone());
    }
    for battle in battle_collection {
        for turn in battle.turn_result {
            for action in turn.action_results {
                println!("attacker: {}",action.actor);
            }
        }
    }

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

fn run_battle(mut battle_order: Vec<BattleOrder>) -> BattleResult {
    let mut battle_result: BattleResult = Default::default();
    let mut winning_result = false;

    while !winning_result {
        let turn_result = run_battle_turn(&mut battle_order);
        if turn_result.action_results.len() == 0 { winning_result=true}
        battle_result.turn_result.push(turn_result);
    }
    battle_result
}


fn run_battle_turn(battle_order: &mut Vec<BattleOrder>) -> TurnResult{
    let mut turn_result: TurnResult = Default::default();

    for i in 0..battle_order.len() {
        if battle_order[i].character_state == HealthState::NOMINAL {
            let target = battle_order[i].character.select_target(&battle_order);
            match target {
                Some(x) => {
                    let attack_result = melee_attack(battle_order[i].character.to_hit, battle_order[x].character.armour_class, battle_order[i].character.damage);
                    let damage_done = battle_order[x].resolve_damage(attack_result.damage_roll);
                
                    let action_result =  ActionResult {
                        actor: battle_order[i].character.name.clone(),
                        target: battle_order[i].character.name.clone(),
                        action_number: i as u16,                           
                        action_type: ActionType::Attack,
                        action_roll: attack_result.attack_roll,
                        action_result: attack_result.attack_result,
                        action_damage: attack_result.damage_roll as u16,
                        damage_done: damage_done,
                    };
                    turn_result.action_results.push(action_result);
                }
                None => {
                    return turn_result}
            }

        }
    }
    turn_result
}

impl BattleOrder {
    fn resolve_damage(&mut self, damage: u8) -> DamageResult {
        let mut result: DamageResult = Default::default();
        let remaining_hit_points = self.character.hit_points as i8 - damage as i8;
    
        match remaining_hit_points  {
            d if d < 0  => { 
                result =  DamageResult{remaining_hit_points: 0,target_state:HealthState::DEAD};
                self.character.hit_points = 0;
                self.character_state = HealthState::DEAD;
            },
            0 => {
                result = DamageResult{remaining_hit_points: 0,target_state:HealthState::KO};
                self.character.hit_points = 0;
                self.character_state = HealthState::KO;

            }
            d if d > 0 => {
                result = DamageResult{remaining_hit_points: d as u8,target_state:HealthState::NOMINAL};
                self.character.hit_points = remaining_hit_points as u8;
            }
            _ => panic!("yup we got here...resolve_damage(damage: u8, hit_points: u8)"),
        }
        result
    }
}

impl CharacterStruct {
    fn select_target(&self, combatant_list: &Vec<BattleOrder>) -> Option<usize> {
        for i in  0..combatant_list.len() {
            if combatant_list[i].character.name != self.name && combatant_list[i].character_state == HealthState::NOMINAL {
                println!("{} chooses {}", self.name, combatant_list[i].character.name);
                return Some(i);
            }
        }
        None
    }
}

fn melee_attack(to_hit: u8, armour_class: u8, damage: u8) -> AttackResult {
    let mut rng = rand::thread_rng();

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

    let player3 = CharacterStruct {
        name: String::from("Player 3"),
        hit_points: 10,
        armour_class: 1,
        to_hit: 12,
        actions_per_round: 1,
        damage: 6,  
    };

    let mut player_vec = Vec::new(); 
    player_vec.push(player1);
    player_vec.push(player2);    
    player_vec.push(player3);

    
    battle(&player_vec, 10);
}