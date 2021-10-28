use std::{fmt};
use std::{io::Error, thread};
use rand::Rng;
use std::collections::HashMap;
use std::sync::mpsc::{SendError, Sender, channel};

use crate::file_writer::FileWriter;

mod dice_thrower;
mod file_writer;

const BATTLE_COLLECTION_SUMMARY_FILE: &str = "./output/bc_summary.out";
const BATTLE_COLLECTION_ACCUMULATION_FILE: &str = "./output/bc_accumulation.out";

fn main() -> Result<(),String> {
    let player_vec = get_players();

    const DESIRED_ITERATIONS: u32 = 10_000_000;
    const THREADS_DESIRED: u8 = 10;
    let thread_iterations = DESIRED_ITERATIONS/u32::from(THREADS_DESIRED);

    let mut battle_collection_list:Vec<Result<String,SendError<SendBuffer>>> = Vec::with_capacity(6);
    let mut thread_list: Vec<thread::JoinHandle<Result<String,SendError<SendBuffer>>>> = Vec::with_capacity(6);
    let (sender,receiver) = channel();

    thread::spawn(move || {
        loop {
            let rx: SendBuffer = receiver.recv().unwrap();
            match write_to_file(&rx.buffer , &rx.file_name) {
                Err(error) => println!("Received Write Thread error: {}", error),
                Ok(_) => continue,
            }
        }
    });

    for i in 0..THREADS_DESIRED as usize{
        let local_player_vec = player_vec.clone();
        let sender = sender.clone();
        thread_list.push(thread::spawn(move||
            {
                battle(&local_player_vec, thread_iterations, i as u8, ReportOutputLevel::None, sender)
            }));   
    }
 
    for thread_counter in thread_list {
        battle_collection_list.push(thread_counter.join().unwrap());
    }
    
    Ok(())
}

fn write_to_file(buffer: &str, file_name: &str) -> Result<String,std::io::Error> {
    let writer = file_writer::new(file_name);

    match writer {
        FileWriter::Ready(file) => 
            match file.write_buffer(&buffer){
                    Ok(_) => Ok(String::from("Okay")),
                    Err(error) => Err(handle_file_error(error)),
                },
        FileWriter::Error(error) => Err(handle_file_error(error)),
    }
}

fn handle_file_error(error: Error ) -> Error {
    panic!("{}",error);
}

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

fn battle( players: &[CharacterStruct], battle_count: u32, arena_id: u8, report_level: ReportOutputLevel, sender: Sender<SendBuffer>) -> Result<String, SendError<SendBuffer>> {
    const DUMP_INCREMENT: u32 = 500_000;
    let battle_order_list = make_battle_order_list(players, &report_level);
    let mut battle_result_collection = BattleResultCollection {
        battle_order_list: battle_order_list.battle_order_list.clone(),
        battle_count,
        arena_id,
        ..Default::default()
    };

    let initiative_winner = &battle_order_list.battle_order_list.iter().max_by_key(|p| p.initative_roll).unwrap();
    battle_result_collection.battle_order_list = battle_order_list.battle_order_list.clone();
    let mut dump_counter = DUMP_INCREMENT;

    for battle_num in 0..battle_count {
        let mut battle_result = battle_order_list.clone().run_battle(battle_num, &report_level);
        battle_result.battle_id = format!("{}{:0>6}", arena_id, battle_num);        
        battle_result.initiative_winner = initiative_winner.team.to_string();
        battle_result_collection.battle_result_list.push(battle_result.clone());

        if battle_num > dump_counter {        
            dump_counter += DUMP_INCREMENT;
            let summary = battle_result_collection.summarize().unwrap();
            let send_buffer = SendBuffer {
                buffer: format!("{}",summary),
                file_name: BATTLE_COLLECTION_SUMMARY_FILE,                
            };
            write_buffer(send_buffer, &sender)?;

            let summary = battle_result_collection.accumulate_summary().unwrap();
            let send_buffer = SendBuffer {
                buffer: format!("{}",summary),
                file_name: BATTLE_COLLECTION_ACCUMULATION_FILE,                
            };
            write_buffer(send_buffer, &sender)?;

            battle_result_collection.battle_result_list.clear();
        }
    }
    Ok(String::from("Okay"))
}

fn write_buffer(send_buffer: SendBuffer, sender: &Sender<SendBuffer>) -> Result<String,SendError<SendBuffer>> {
    if let Err(error) = sender.send(send_buffer){
        Err(error)
    } else { Ok("grand".to_string()) } 
}

struct SendBuffer {
    buffer: String,
    file_name: &'static str,
}

fn make_battle_order_list(players: &[CharacterStruct], report_level: &ReportOutputLevel) -> BattleOrderList {
    let mut battle_order_list: Vec<BattleOrder> = Vec::with_capacity(players.len());
    let initiative_die = "1d20".to_string();
    let roll_request = dice_thrower::parse_request(&initiative_die).unwrap();
    
    for player in players {
        let initative_roll = dice_thrower::throw_roll(&roll_request);
        
        let order = BattleOrder {
            initative_roll: initative_roll as u8,
            character: player.clone(),
            team: player.team,
        };
        battle_order_list.push(order);
    }
    battle_order_list.sort_by(|a,b| b.initative_roll.cmp(&a.initative_roll));
    match report_level {
        ReportOutputLevel::Summary => BattleOrderList { 
            battle_order_list, 
            report_level: ReportOutputLevel::Summary },
        ReportOutputLevel::Accumulate => BattleOrderList {
            battle_order_list,
            report_level: ReportOutputLevel::Accumulate },
        ReportOutputLevel::None => BattleOrderList {
            battle_order_list,
            report_level: ReportOutputLevel::None },
    }
}

#[derive(Clone, Copy, Debug)]
enum ReportOutputLevel {
    None,
    Summary,
    Accumulate,
}

impl Default for ReportOutputLevel {
    fn default() -> Self { ReportOutputLevel::None }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct CharacterStruct {
    name: String,
    // hit_points: u8,
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
        for (i, target) in combatant_list.iter().enumerate() {
            if target.character.team != self.team &&
                target.character.is_concious() {
                    return Some(i)
            }
        }
        None
    }

    fn is_concious(&self) ->  bool {
        match self.hs2 {
            HealthState::Dead | HealthState::Ko => false,
            HealthState::Alive(_) => true,
        }
    }

    fn do_some_damage(&self) -> DamageResult {
        let roll_request = dice_thrower::parse_request(&self.weapon.to_string()).unwrap();

        DamageResult {
            damage: dice_thrower::throw_roll(&roll_request) as u8,
        }
    }

    fn is_attack_successful(&self, attack_result: &AttackResult ) -> bool {
        if attack_result.attack_roll > self.armour_class {
            return true
        }
            false
    }

    fn make_attack(&self) -> AttackResult {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..=self.to_hit);

        AttackResult { 
            attack_roll: roll,
            _roll_string: self.weapon,
        }
    }

    fn _defend_attack(){}

    fn take_damage(mut self, damage: u16) -> Self {
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

#[derive(Clone, Debug, Copy)]
enum ActionResultType {
    _CritMiss,
    Miss,
    Hit,
    _CritHit,
}

#[derive(Clone, Debug, Copy)]
enum ActionType {
    Attack,
    _Dodge,
    _Cast,
    _Dash,
    _NoAction,
    _NoTarget,
}

struct AttackResult {
    attack_roll: u8,
    _roll_string: &'static str,
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

struct TurnResultSummary {
    _action_count: u8,
    _number_of_hits: u8,
    _damage_done: u8,
}

impl Summary<TurnResultSummary> for TurnResult {
    fn summarize(&self) -> Option<TurnResultSummary>{
        let _action_count: u8 = self.action_results.len() as u8;
        let _number_of_hits = self.action_results.iter().fold(
            0, |i, action| 
            match action.action_result {
                ActionResultType::Hit => i + 1,
                ActionResultType::_CritHit => i + 1,
                _ => i,
            }
        );
        let _damage_done: u8 = self.action_results.iter().fold(
            0, |i: u8, action| 
            i + action.action_damage as u8);
        Some(TurnResultSummary {
            _action_count,
            _number_of_hits,
            _damage_done,
        })
    }
}

struct TurnAccumulation {

}

impl Accumulate<TurnAccumulation> for TurnResult {
    fn accumulate_summary(&self) -> Option<TurnAccumulation> {
        todo!()
    }
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

impl fmt::Display for ActionResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{},{},{},{:?},{},{:?},{}", 
            self.action_number, 
            self.actor,
            self.target,
            self.action_type, 
            self.action_roll,
            self.action_result,
            self.action_damage,
        )
    }
}

#[derive(Clone, Debug, Default)]
struct BattleOrderList {
    battle_order_list: Vec<BattleOrder>,
    report_level: ReportOutputLevel,
}

impl BattleOrderList {
    fn run_battle(mut self, battle_num: u32, report_level: &ReportOutputLevel) -> BattleResult {
        let mut winning_result = false;
        let mut turn_count: u8 = 1;
        let mut turn_result: TurnResult = Default::default();
        let turn_result_ref = &mut turn_result;
        let mut battle_result = BattleResult {
            battle_id: battle_num.to_string(),
            initiative_winner: self.battle_order_list[0].character.name.clone(),
            ..Default::default()
        };
    
        while !winning_result {
            turn_result_ref.turn_number = turn_count;

            self = self.run_battle_turn(turn_result_ref);

            if self.is_there_a_winner() {
                winning_result = true;
            }
            else {
                turn_count += 1;
            }
        }
        battle_result.turn_result.push(turn_result_ref.clone());

        let winner = self.get_winner();
        if let Some(body) = winner {
            battle_result.winner = body.character.clone()
        }

        battle_result.turns_run = turn_count;
        if let ReportOutputLevel::Summary = report_level {
            let result_summary = battle_result.summarize();
            if let Some(summary) = result_summary {     
                println!("{}",summary);           
            }
        }
        
        battle_result
    }
    
    fn run_battle_turn(mut self, turn_result: &mut TurnResult) -> Self {
        let mut action_result: ActionResult;
        let turn_order = self.battle_order_list.clone();
        let mut casuality_list = self.battle_order_list.clone();

        for (i, actor) in turn_order.iter().enumerate() {
            if !casuality_list[i].character.is_concious() { continue }
            if let HealthState::Alive(_) = actor.character.hs2 {
                let target = actor.get_target(&casuality_list);
                if let Some(target) = target {
                    let a_res = actor.make_attack();
                    if casuality_list[target].is_attack_successful(&a_res){
                        let d_res = actor.get_damage();
                        action_result = ActionResult {
                            actor: actor.character.name.clone(),
                            target: casuality_list[target].character.name.clone(),
                            action_type: ActionType::Attack,
                            action_roll: a_res.attack_roll,
                            action_result: ActionResultType::Hit,
                            action_damage: d_res.damage as u16, 
                            action_number: i as u16,
                        };
                        turn_result.action_results.push(action_result);
                        casuality_list[target].give_damage(d_res.damage as u16);
                        self.battle_order_list[target].give_damage(d_res.damage as u16);
                    }
                    else {
                        action_result = ActionResult {
                            actor: actor.character.name.clone(),
                            target: self.battle_order_list[target].character.name.clone(),
                            action_type: ActionType::Attack,
                            action_roll: a_res.attack_roll,
                            action_result: ActionResultType::Miss,
                            action_damage: 0, 
                            action_number: i as u16,
                        };
                        turn_result.action_results.push(action_result);
                    }
                }
                else {
                    action_result = ActionResult {
                        actor: actor.character.name.clone(),
                        target: "no target".to_string(),
                        action_type: ActionType::_NoTarget,
                        action_roll: 0,
                        action_result: ActionResultType::Miss,
                        action_damage: 0, 
                        action_number: i as u16,
                    };
                    turn_result.action_results.push(action_result);
                }
            }
        }
        self
    }

    fn is_there_a_winner(&self) -> bool {
        let mut alive_team = None;

         for player in &self.battle_order_list {
            if player.character.is_concious() {
                match alive_team {
                    Some(team) => if team != player.team { return false },
                    None => alive_team = Some(player.team),
                };
            }
         }
        true
    }

    fn get_winner(&self) -> Option<&BattleOrder> {
        for player in &self.battle_order_list {
            if player.character.is_concious() {
                return Some(player);
            }
        }
        println!("get_winner but none found in {:?}",self.battle_order_list);
        None
    }

    fn _get_initiative_winner(&self) -> CharacterStruct {
        self.battle_order_list[0].character.clone()
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
        self.character.make_attack()
    }

    fn get_target(&self, target_list: &[BattleOrder]) -> Option<usize> {
        self.character.select_target(target_list)
    }

    fn is_attack_successful(&self, attack_result: &AttackResult) -> bool {
        self.character.is_attack_successful(&attack_result)
    }

    fn get_damage(&self) -> DamageResult {
        self.character.do_some_damage()
    }

    fn give_damage(&mut self, damage: u16) -> DamageResult {
        self.character = self.character.clone().take_damage(damage);
        DamageResult {
            damage: damage as u8,
        }
    }
    
}

trait Summary <T> {
    fn summarize(&self) -> Option<T>;
}

trait Accumulate<T> {
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
    max_turns_run: u8,    
}

impl fmt::Display for CollectionSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.arena_id, self.battle_count, self.total_turns_run, self.average_turns_run, self.max_turns_run)
    }
}

struct CollectionAccumulation {
    number_of_battles: u32,
    accumulation: HashMap<(u16,String),u32>,
}

impl fmt::Display for CollectionAccumulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        for row in self.accumulation.clone() {
            let write_row = format!("{:?},{}\n", row.0, row.1);
            output.push_str(&write_row);
        }
        write!(f, "{}", output)
    }    
}

impl Summary<CollectionSummary> for BattleResultCollection {
    fn summarize(&self) -> Option<CollectionSummary> {
        let total_turns_run: u32 = self.battle_result_list.iter()
            .fold(0_u32, |acc, battle_result| acc + battle_result.turns_run as u32);
        let battle_collection_summary = CollectionSummary {
            arena_id: self.arena_id, 
            battle_count: self.battle_count,
            total_turns_run,
            average_turns_run: (total_turns_run /self.battle_count) as u16,
            max_turns_run: self.battle_result_list.iter().fold(0_u8, |max, battle_result| if max > battle_result.turn_result.len() as u8 { max }
                else { battle_result.turn_result.len() as u8 }),    
        };
        Some(battle_collection_summary)
    }
}

impl Accumulate<CollectionAccumulation> for BattleResultCollection {

    fn accumulate_summary(&self) -> Option<CollectionAccumulation>{
        let mut number_of_battles: u32 = 0;
        let mut accumulation:HashMap<(u16,String),u32> = HashMap::new();

        for battle in &self.battle_result_list {
            number_of_battles += 1;
            let res = battle.summarize();

            if let Some(battle_summary) = res {
                let winner = if battle_summary.winner == battle_summary.initiative_winner { 
                    format!("{}*", battle_summary.winning_team)
                }
                else { format!("{}", battle_summary.winning_team) };
                    *accumulation.entry((battle.turns_run as u16,winner)).or_insert(0) += 1;
            }
        }
        let battle_collection_accumulation = CollectionAccumulation{
            number_of_battles,
            accumulation,
        };
        Some(battle_collection_accumulation)
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

impl fmt::Display for BattleSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{},{:?}", self.battle_id, self.turns_run, self.initiative_winner, self.winning_team)
    }
}

impl Summary<BattleSummary> for BattleResult {
    fn summarize(&self) -> Option<BattleSummary> {
        let battle_summary = BattleSummary {
            battle_id: self.battle_id.clone(), 
            turns_run: self.turns_run as u8, 
            winner: self.winner.team.to_string(),
            initiative_winner: self.initiative_winner.clone(),
            winning_team: self.winner.team,
        };
        Some(battle_summary)
    }
}


#[test]
fn test_make_battle_order_list() {
    let players = get_players(); 
    let test_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);
    assert_ne!(test_list.battle_order_list.len(),0,"no list");
}

#[test]
fn test_order_of_make_battler_order_list() {
    let players = get_players();
    let test_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);
    assert_eq!(test_list.battle_order_list[0].initative_roll > test_list.battle_order_list[1].initative_roll,true,"list not ordered");
}

#[test]
fn get_players_test() {
    let list = get_players();
    assert_ne!(list.len(), 0, "Character get fail")
}

#[test]
fn select_target_test() {
    let players = get_players();
    let test_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);
    let target = test_list.battle_order_list[0].character.select_target(&test_list.battle_order_list);

    assert_ne!(target,Some(0),"select_target selected self");
    match target {
        Some(index) => assert_eq!(test_list.battle_order_list[index].character.is_concious(),true),
        None => ()
    }
}

#[test]
fn is_concious_test() {
    let players = get_players();

    let mut actor = players[0].clone();
    assert_eq!(actor.is_concious(),true);

    actor.hs2 = HealthState::Ko;
    assert_eq!(actor.is_concious(),false);

    actor.hs2 = HealthState::Dead;
    assert_eq!(actor.is_concious(),false);
}

#[test]
fn take_damage_test() {
    let players = get_players();
    let mut actor = players[0].clone();
    let original_health_state = actor.hs2;

    actor = actor.take_damage(5);
    assert_ne!(original_health_state,actor.hs2);
}

#[test]
fn is_winner_test() {
    let players = get_players();
    let order_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);

    assert_eq!(order_list.is_there_a_winner(),false);

    let one_list = vec!(players[0].clone());
    let one_order_list = make_battle_order_list(&one_list, &ReportOutputLevel::Summary);

    assert_eq!(one_order_list.is_there_a_winner(),true);
}

#[test]
fn is_attack_successful() {
    let players = get_players();
    let successful_attack = AttackResult {
        attack_roll: 20,
        _roll_string: "1d12",
    };
    let failure_attack = AttackResult {
        attack_roll: 1,
        _roll_string: "1d12",
    };

    assert_eq!(players[0].is_attack_successful(&successful_attack),true);
    assert_eq!(players[0].is_attack_successful(&failure_attack),false);
}