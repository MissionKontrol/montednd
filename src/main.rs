use std::fmt;
use std::thread;
// use std::fmt::Write;
use rand::Rng;
use std::collections::HashMap;

fn main() -> Result<(),String> {
    let player_vec = get_players();

    let desired_iterations = 1_200;
    let threads_desired: u8 = 6;
    let thread_iterations = desired_iterations/threads_desired as u32;

    let mut battle_collection_list:Vec<BattleResultCollection> = Vec::with_capacity(6);
    let mut thread_list: Vec<thread::JoinHandle<_>> = Vec::with_capacity(6);

    for i in 0..threads_desired as usize{
        let local_player_vec = player_vec.clone();
        thread_list.push(thread::spawn(move||
            {
                battle(&local_player_vec, thread_iterations, i as u8)
            }));   
    }
 
    for thread_counter in thread_list {
        battle_collection_list.push(thread_counter.join().unwrap());
    }

    for battle in &battle_collection_list {
        // let summary = battle.summarize().expect("something");
        if let BattleCollectionSummary::Summary(collection) = 
            battle.summarize().expect("something") {
                println!("{}", collection.battle_count );
                println!("{}", collection);
        }
    }

    for battle in &battle_collection_list {
        if let BattleCollectionSummary::Accumulation(collection) = 
            battle.accumulate_summary().expect("accumulator says not") {
                println!("{}", collection.number_of_battles);
                println!("{}", collection)
            }
    }

    Ok(())
}

fn get_players() -> Vec<CharacterStruct> {
    let player1 = CharacterStruct {
        name: String::from("Hero"),
        hit_points: 10,
        armour_class: 12,
        to_hit: 20,
        weapon: "+1",
        actions_per_round: 1,
        damage: 6,  
        team: Team::Heros,
        hs2: HealthState::Alive(10),
    };

    let player2 = CharacterStruct {
        name: String::from("Villan-A"),
        hit_points: 6,
        armour_class: 10,
        to_hit: 20,
        weapon: "+1",
        actions_per_round: 1,
        damage: 4,  
        team: Team::Villains,
        hs2: HealthState::Alive(6),
    };

    let player3 = CharacterStruct {
        name: String::from("Villan-B"),
        hit_points: 6,
        armour_class: 10,
        to_hit: 20,
        weapon: "+1",
        actions_per_round: 1,
        damage: 4,  
        team: Team::Villains,
        hs2: HealthState::Alive(6),
    };

    let player_vec = vec!(player1,player2,player3);
    player_vec
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct CharacterStruct {
    name: String,
    hit_points: u8,
    armour_class: u8,
    to_hit: u8,
    weapon: &'static str,
    actions_per_round: u8,
    damage: u8,
    team: Team,
    hs2: HealthState,
}

impl CharacterStruct {
    fn select_target(&self, combatant_list: &[BattleOrder]) -> Option<usize> {
        for i in  0..combatant_list.len() {
            if combatant_list[i].character.team != self.team &&
                combatant_list[i].character.is_concious() {
                    return Some(i)
            }
        }
        None
    }

    fn is_concious(&self) ->  bool {
        match self.hs2 {
            HealthState::Dead => false,
            HealthState::Ko => false,
            HealthState::Alive(_) => true,
        }
    }

    fn do_some_damage(&self) -> DamageResult {
        let mut rng = rand::thread_rng();

        DamageResult {
            damage: rng.gen_range(1..=self.damage),
        }
    }

    fn is_attack_successful(&self, attack_result: &AttackResult ) -> bool {
        if attack_result.attack_roll > self.armour_class {
            return true
        }
            false
    }

    fn decide_attack(&self, target: usize) -> ActionType{
        ActionType::Attack(AttackRequest {
            roll_string: self.weapon,
            target_index: target,
        })
    }

    fn _defend_attack(){}

    fn take_damage(&mut self, damage: u16) -> &Self {
        self.hs2 = self.hs2.update_health_state(damage as i16);
        self
    }

    fn _react_to(){}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Debug)]
enum Team {
    Heros,
    Villains,
}

impl Default for Team {
    fn default() -> Self {
        Team::Heros
    }
}

#[derive(Clone, Debug, Copy)]
enum ActionResultType {
    _CritMiss,
    Miss,
    Hit,
    _CritHit,
}

#[derive(Clone, Debug, Copy)]
enum ActionType {
    Attack(AttackRequest),
    _Dodge,
    _Cast,
    _Dash,
}

#[derive(Clone, Copy, Debug)]
struct AttackRequest {
    roll_string: &'static str,
    target_index: usize,
}

#[derive(Debug, Clone, Copy, PartialOrd, Eq, Ord, PartialEq)]
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

struct AttackResult {
    attack_roll: u8,
    damage_roll: u8,
    attack_result: ActionResultType,
}

#[derive(Default, Clone, Debug)]
struct DamageResult{
    damage: u8,
    // hit_type: ActionResultType,
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
}

trait Summary <T> {
    fn summarize(&self) -> Option<T>;
    fn accumulate_summary(&self) -> Option<T>;
}

#[derive(Default)]
struct BattleResultCollection {
    arena_id: u8,
    battle_count: u32,
    battle_result_list: Vec<BattleResult>,
    battle_order_list: Vec<BattleOrder>,
}

struct CollectionSummary {
    arena_id: u8,
    battle_count: u32,
    total_turns_run: u32,
    average_turns_run: u16,
    _max_turns_run: u8,    
}

struct CollectionAccumulation {
    number_of_battles: u32,
    accumulation: HashMap<(u16,String),u16>,
}

impl fmt::Display for CollectionSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.arena_id, self.battle_count, self.total_turns_run, self.average_turns_run, self._max_turns_run)
    }
}

impl fmt::Display for CollectionAccumulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        for row in self.accumulation.clone() {
            let write_row = format!("{:?} {}\n", row.0, row.1);
            output.push_str(&write_row);
        }
        write!(f, "{}", output)
    }    
}

enum BattleCollectionSummary {
    Summary(CollectionSummary),
    Accumulation(CollectionAccumulation),
}

impl Summary<BattleCollectionSummary> for BattleResultCollection {
    fn summarize(&self ) -> Option<BattleCollectionSummary> {
        let total_turns_run: u32 = self.battle_result_list.iter()
            .fold(0u32, |acc, battle_result| acc + battle_result.turns_run as u32);
        let battle_collection_summary = CollectionSummary {
            arena_id: self.arena_id, 
            battle_count: self.battle_count,
            total_turns_run,
            average_turns_run: (total_turns_run /self.battle_count) as u16,
            _max_turns_run: self.battle_result_list.iter().fold(0u8, |max, battle_result| if max > battle_result.turn_result.len() as u8 { max }
                else { battle_result.turn_result.len() as u8 }),    
        };
        Some(BattleCollectionSummary::Summary( battle_collection_summary))
    }

    fn accumulate_summary(&self) -> Option<BattleCollectionSummary>{
        let mut number_of_battles: u32 = 0;
        let mut accumulation:HashMap<(u16,String),u16> = HashMap::new();

        for battle in &self.battle_result_list {
            number_of_battles += 1;
            let res = battle.summarize()?;

            if let BattleResultSummary::Summary(battle_summary) = res {
                let winner = if battle_summary.winner == battle_summary.initiative_winner { 
                    format!("{:?}* {}", battle_summary.winning_team, battle_summary.winner)
                }
                else { format!("{:?} {}", battle_summary.winning_team, battle_summary.winner) };
                *accumulation.entry((battle_summary.turns_run as u16,winner)).or_insert(0) += 1;
            }
        }
        let battle_collection_accumulation = CollectionAccumulation{
            number_of_battles,
            accumulation,
        };
        Some(BattleCollectionSummary::Accumulation( battle_collection_accumulation))
    }
}

#[derive(Default, Debug, Clone)]
struct BattleResult{
    battle_id: String,
    turns_run: u8,
    winner: CharacterStruct,
    initiative_winner: String,
    turn_result: Vec<TurnResult>,
}

struct BattleSummary {
    battle_id: String,
    turns_run: u8,
    winner: String,
    initiative_winner: String,
    winning_team: Team,
}

struct BattleAccumulation {
    numer_of_battles: u32,
}

enum BattleResultSummary {
    Summary(BattleSummary),
    Accumulation(BattleAccumulation),
}

impl Summary<BattleResultSummary> for BattleResult {
    fn summarize(&self) -> Option<BattleResultSummary> {
        let battle_summary = BattleSummary {
            battle_id: self.battle_id.clone(), 
            turns_run: self.turn_result.len() as u8, 
            winner: self.winner.name.clone(),
            initiative_winner: self.initiative_winner.clone(),
            winning_team: self.winner.team,
        };
        Some(BattleResultSummary::Summary(battle_summary))
    }

    fn accumulate_summary(&self) -> Option<BattleResultSummary>{
        todo!()
    }
}

#[derive(Clone)]
struct BattleOrderList {
    battle_order_list: Vec<BattleOrder>,
}

impl BattleOrderList{
    fn run_battle(mut self) -> BattleResult {
        let mut battle_result: BattleResult = Default::default();
        let mut winning_result = false;
        let mut turn_count: u8 = 0;
    
        while !winning_result {
            let turn_result = self.clone().run_battle_turn(turn_count);
            match turn_result {
                Some(x) => {
                    turn_count +=1;
                    battle_result.turn_result.push(x.clone());
                },
                None => winning_result=true,
            }
        }
        let winner = self.get_winner().unwrap();
        battle_result.turns_run = turn_count;
        battle_result.winner = winner.character.clone();
        battle_result
    }

    fn get_winner(&self) -> Option<&BattleOrder> {
        for player in &self.battle_order_list {
            if let HealthState::Alive(_) = player.character.hs2 {
                return Some(player);
            }
        }
        None
    }
    
    fn run_battle_turn(mut self, turn_number: u8) -> Option<TurnResult>{
        let mut turn_result = TurnResult { turn_number, ..Default::default() };
        turn_result.turn_number = turn_number;
    
        for actor in 0..self.battle_order_list.len() {
            if let HealthState::Alive(_) = self.battle_order_list[actor].character.hs2 {
                let target = self.battle_order_list[actor].character.select_target(&self.battle_order_list);
                match target {
                    Some(target) => {
                        let a_res = self.battle_order_list[actor].make_attack();
                        if self.battle_order_list[target].character.is_attack_successful(&a_res){
                            let d_res = self.battle_order_list[actor].character.do_some_damage();
                            self.battle_order_list[target].character.take_damage(d_res.damage as u16);

                        }
                    }
                    None => return Option::None
                };
            }
        }
        Some(turn_result)
    }
}

#[derive(Default, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
struct BattleOrder {
    initative_roll: u8,
    character: CharacterStruct,
    team: Team,
}

impl BattleOrder {
    fn make_attack(&self) -> AttackResult {
        let mut rng = rand::thread_rng();

        let roll = rng.gen_range(1..=self.character.to_hit);
        // TODO
        // add weapon + ability
        AttackResult { 
            damage_roll: roll,
            attack_roll: roll,
            attack_result: ActionResultType::Hit,
        }
    }

    // fn take_action(&self, target: usize) -> ActionType {
    //     let attack_request = self.character.decide_attack(target);
    //     let attack_ = self.make_attack(attack_request);
    // }
}
 
// fn run_turn_action(turn_number: u8, target: &mut CharacterStruct, actor: &mut CharacterStruct) -> ActionResult {
//     let a_res = actor.make_attack();
//     if target.attack_is_successful(&a_res){
//         let d_res = actor.do_some_damage();
//         target.take_damage(d_res.damage as u16);

//         let action_result =  ActionResult {
//             actor: actor.name.clone(),
//             target: target.name.clone(),
//             action_number: 1,                         // TODO fix  
//             action_type: ActionType::Attack,
//             action_roll: a_res.attack_roll,
//             action_result: ActionResultType::Hit,
//             action_damage: d_res.damage as u16,
//         };
//         action_result
//     }
//     else {
//         let action_result =  ActionResult {
//             actor: actor.name.clone(),
//             target: target.name.clone(),
//             action_number: 1,                         // TODO fix  
//             action_type: ActionType::Attack,
//             action_roll: a_res.attack_roll,
//             action_result: ActionResultType::Miss,
//             action_damage: 0,
//         };
//         action_result
//     }
// }

fn battle( players: &[CharacterStruct], battle_count: u32, arena_id: u8) -> BattleResultCollection {
    let battle_order_list = make_battle_order_list(players);
    let mut battle_collection_list = BattleResultCollection {
        battle_order_list: battle_order_list.battle_order_list.clone(),
        battle_count,
        arena_id,
        ..Default::default()
    };

    let initiative_winner = &battle_order_list.battle_order_list.iter().max_by_key(|p| p.initative_roll).expect("duff list");

    for battle_num in 0..battle_count {
        let mut battle_result = battle_order_list.clone().run_battle();
        battle_result.battle_id = format!("{}{:0>6}", arena_id, battle_num);
        battle_result.initiative_winner = initiative_winner.character.name.clone();
        battle_collection_list.battle_order_list = battle_order_list.battle_order_list.clone();
        battle_collection_list.battle_result_list.push(battle_result.clone());
    }

    battle_collection_list
}

fn make_battle_order_list(players: &[CharacterStruct]) -> BattleOrderList {
    let mut rng = rand::thread_rng();
    let mut battle_order_list = Vec::new();
    
    for player in players {
        let initative_roll = rng.gen_range(1..=20);
        let order = BattleOrder {
            initative_roll,
            character: player.clone(),
            team: player.team,
        };
        battle_order_list.push(order);
    }
    battle_order_list.sort_by(|a,b| b.initative_roll.cmp(&a.initative_roll) );
    BattleOrderList { battle_order_list }
}

#[test]
fn test_make_battle_order_list() {
    let players = get_players(); 
    let test_list = make_battle_order_list(&players);
    assert_ne!(test_list.battle_order_list.len(),0,"no list");
}

#[test]
fn test_order_of_make_battler_order_list() {
    let players = get_players();
    let test_list = make_battle_order_list(&players);
    assert_eq!(test_list.battle_order_list[0].initative_roll > test_list.battle_order_list[1].initative_roll,true,"list not ordered");
}

#[test]
fn get_players_test() {
    let list = get_players();
    assert_ne!(list.len(), 0, "Character get fail")
}
