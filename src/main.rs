use std::thread;
use std::fmt::Write;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    let player1 = CharacterStruct {
        name: String::from("Hero"),
        hit_points: 10,
        armour_class: 12,
        to_hit: 20,
        actions_per_round: 1,
        damage: 6,  
        team: Team::Heros,
    };

    let player2 = CharacterStruct {
        name: String::from("Baddie"),
        hit_points: 6,
        armour_class: 10,
        to_hit: 20,
        actions_per_round: 1,
        damage: 4,  
        team: Team::Villains,
    };

    // let player3 = CharacterStruct {
    //     name: String::from("Villan 2"),
    //     hit_points: 8,
    //     armour_class: 2,
    //     to_hit: 20,
    //     actions_per_round: 1,
    //     damage: 4,  
    //     team: Team::Villains,
    // };

    let mut player_vec = Vec::new(); 
    player_vec.push(player1);
    player_vec.push(player2);    
    // player_vec.push(player3);

    let player_vec1 = player_vec.clone();
    let player_vec2 = player_vec.clone();
    let player_vec3 = player_vec.clone();
    let player_vec4 = player_vec.clone();

    let thread_iterations = 1_000_000;

    let thread_one = thread::spawn(move||
        {
            battle(&player_vec1, thread_iterations, "1")
        });
        
    let thread_two = thread::spawn(move||
        {
            battle(&player_vec2, thread_iterations, "2")
        });

    let thread_three = thread::spawn(move||
        {
            battle(&player_vec3, thread_iterations, "3")
        });    

    let thread_four = thread::spawn(move||
        {
            battle(&player_vec4, thread_iterations, "4")
        });   

    let _res = thread_one.join();
    let _res = thread_two.join();
    let _res = thread_three.join();
    let _res = thread_four.join();

}


#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct CharacterStruct {
    name: String,
    hit_points: u8,
    armour_class: u8,
    to_hit: u8,
    actions_per_round: u8,
    damage: u8,
    team: Team,
}

impl CharacterStruct {
    fn select_target(&self, combatant_list: &Vec<BattleOrder>) -> Option<usize> {
        for i in  0..combatant_list.len() {
            if combatant_list[i].character.team != self.team && combatant_list[i].character_state == HealthState::NOMINAL {
                return Some(i);
            }
        }
        None
    }
}

struct AttackResult {
    attack_roll: u8,
    damage_roll: u8,
    attack_result: ActionResultType,
}

#[derive(Default, Clone, Debug)]
struct DamageResult{
    remaining_hit_points: u8,
    target_state: HealthState,
}

#[derive(Debug, Clone, PartialOrd, Eq, Ord, PartialEq)]
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


#[derive(Default, Debug, Clone)]
struct TurnResult {
    turn_number: u8,
    action_results: Vec<ActionResult>,
}

#[derive(Clone, Debug)]
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


#[derive(Default, Debug, Clone)]
struct BattleResult{
    battle_id: u32,
    turns_run: u8,
    winner: CharacterStruct,
    turn_result: Vec<TurnResult>,
    battle_order_list: Vec<BattleOrder>,
}

impl BattleResult {
    fn summarize_battle(&self, prefix: &str ) {
        let mut output: String;

        output = format!("{}-{} {} {:?}", prefix, 
            self.battle_id.to_string(), 
            self.turns_run.to_string(), 
            self.winner.team);

        let initiative_winner = self.battle_order_list.iter().max_by_key(|p| p.initative_roll);
        match initiative_winner {
            Some(player) => {
                if player.team == self.winner.team {
                    write!(output, "*").unwrap()
                }
            },
            None => write!(output, "initative Tie I guess").unwrap(),
        }

        println!("{}", output);
    }
}

#[derive(Default, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
struct BattleOrder {
    initative_roll: u8,
    character_state: HealthState,
    character: CharacterStruct,
    team: Team,
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Team {
    Heros,
    Villains,
}

impl Default for Team {
    fn default() -> Self {
        Team::Heros
    }
}

#[derive(Clone, Debug)]
enum ActionResultType {
    _CritFail,
    _Fail,
    Miss,
    Hit,
    _CritHit,
}

#[derive(Clone, Debug)]
enum ActionType {
    Attack,
    _Dodge,
    _Cast,
    _Initative,
}

fn battle( players: &Vec<CharacterStruct>, num_iterations: u32, battle_prefix: &str) {
    let battle_order_list = make_battle_order_list(players);
    let mut battle_collection_list:Vec<BattleResult> = Vec::new();

    for battle_num in 0..num_iterations {
        let mut battle_result = run_battle(battle_order_list.clone());
        battle_result.battle_id = battle_num;
        battle_result.battle_order_list = battle_order_list.clone();
        battle_collection_list.push(battle_result.clone());
    }

    let mut turn_summary = HashMap::<u8,u32>::new();

    for battle in battle_collection_list {
        // battle.summarize_battle(battle_prefix);
        turn_summary.insert(battle.turns_run, 1 + if turn_summary.contains_key(&battle.turns_run) { turn_summary[&battle.turns_run] } else {1});
        // println!("Battle: {}  {} turns won by {:?} {}",battle.battle_id, battle.turns_run, battle.winner.name, battle.winner.hit_points);
        // for turn in battle.turn_result {
        //     for action in turn.action_results {
        //         println!("  {} {} {} {:?} {} {:?} {} {:?}",
        //             turn.turn_number,
        //             action.action_number,    
        //             action.actor,
        //             action.action_type, action.target, action.action_result, action.action_damage, action.damage_done.target_state);
        //     }
        // }
    }
    for (key, value) in turn_summary.iter(){
        println!("{} {}", key, value);
    }

}

fn make_battle_order_list(players: &Vec<CharacterStruct>) -> Vec<BattleOrder> {
    let mut rng = rand::thread_rng();
    let mut battle_order_list = Vec::new();
    
    for player in players {
        let initative_roll = rng.gen_range(1..=20);
        println!("{} {}", player.name, initative_roll);
        let order = BattleOrder {
            initative_roll: initative_roll,
            character: player.clone(),
            character_state: HealthState::NOMINAL,
            team: player.team.clone(),
        };
        battle_order_list.push(order);
    }
    battle_order_list.sort_by(|a,b| b.initative_roll.cmp(&a.initative_roll) );
    battle_order_list
}

fn run_battle(mut battle_order_list: Vec<BattleOrder>) -> BattleResult {
    let mut battle_result: BattleResult = Default::default();
    let mut winning_result = false;
    let mut turn_count: u8 = 0;

    while !winning_result {
        let turn_result = run_battle_turn(&mut battle_order_list, turn_count);
        turn_count +=1;
        match turn_result {
            Some(x) => battle_result.turn_result.push(x),
            None => winning_result=true,
        }
    }
    let winner = get_winner(battle_order_list).unwrap();
    battle_result.turns_run = turn_count-1;
    battle_result.winner = winner.character;
    battle_result
}

fn get_winner(battle_order_list: Vec<BattleOrder>) -> Option<BattleOrder> {
    for player in battle_order_list {
        if player.character_state == HealthState::NOMINAL {
            return Some(player);
        }
    }
    None
}

fn run_battle_turn(battle_order_list: &mut Vec<BattleOrder>, turn_number: u8) -> Option<TurnResult>{
    let mut turn_result: TurnResult = Default::default();
    turn_result.turn_number = turn_number;

    for i in 0..battle_order_list.len() {
        if battle_order_list[i].character_state == HealthState::NOMINAL {
            let target = battle_order_list[i].character.select_target(&battle_order_list);
            match target {
                Some(target) => {
                    let attack_result = melee_attack(battle_order_list[i].character.to_hit, battle_order_list[target].character.armour_class, battle_order_list[i].character.damage);
                    let damage_done = battle_order_list[target].resolve_damage(attack_result.damage_roll);
                
                    let action_result =  ActionResult {
                        actor: battle_order_list[i].character.name.clone(),
                        target: battle_order_list[target].character.name.clone(),
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
                    return Option::None}
            }

        }
    }
    Some(turn_result)
}

fn melee_attack(to_hit: u8, armour_class: u8, damage: u8) -> AttackResult {
    let mut rng = rand::thread_rng();

    let mut result = AttackResult{
        attack_roll: 0,
        damage_roll: 0,
        attack_result: ActionResultType::Miss,
    };
    
    result.attack_roll  = rng.gen_range(0..=to_hit);
    if  result.attack_roll > armour_class {
        result.damage_roll = rng.gen_range(0..=damage);
        result.attack_result = ActionResultType::Hit;
    }
    result
}