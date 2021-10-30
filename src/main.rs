use std::{fmt};
use std::{io::Error, thread};
use dice_thrower::RollRequest;
use rand::Rng;
use std::collections::HashMap;
use std::sync::mpsc::{SendError, Sender, channel};

use crate::file_writer::FileWriter;

mod dice_thrower;
mod file_writer;
mod characterize;
use characterize::{CharacterStruct, HealthState, Team, load_players};

const BATTLE_COLLECTION_SUMMARY_FILE: &str = "./output/bc_summary.out";
const BATTLE_COLLECTION_ACCUMULATION_FILE: &str = "./output/bc_accumulation.out";
const DESIRED_ITERATIONS: u32 = 10_000_000;
const THREADS_DESIRED: u32 = 10;
const THREAD_ITERATIONS: u32 = DESIRED_ITERATIONS/THREADS_DESIRED;
const WRITE_TO_FILE_TRIGGER: u32 = 50000;

fn main() -> Result<(),String> {
    let player_vec= load_players("./input/temp.json").expect("Main");

    let mut thread_list: Vec<thread::JoinHandle<()>> = Vec::with_capacity(6);
    let (sender, receiver):(Sender<SendBuffer>, std::sync::mpsc::Receiver<_>) = channel();

    let _writer_thread = thread::Builder::new().name("Writer".to_string()).spawn(move || {
        loop {
            let rx_res = receiver.recv();
            match rx_res {
                Ok(foo) => {
                    let res= write_to_file(&foo.buffer , &foo.file_name);
                    match res {
                        Err(error) => println!("Received Write Thread error: {}", error),
                        Ok(_) => continue,
                    };
                }
                Err(error) => println!("Writer Thread: {}", error),
            }
        }
    });

    for i in 0..THREADS_DESIRED as usize{
        let local_player_vec: Vec<CharacterStruct> = player_vec.clone();
        let sender = sender.clone();
        let name = format!("Sender-{}",i);
        let builder = thread::Builder::new().name(name);

        let handle: thread::JoinHandle<()> = builder.spawn(move || {
            battle(&local_player_vec, THREAD_ITERATIONS, i as u8, ReportOutputLevel::None, sender).unwrap();
        }).unwrap();   
        thread_list.push(handle);
    }
 
    for thread_counter in thread_list {
        if let Err(error) = thread_counter.join() {
            println!("{:?}",error)
        };
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
    println!("HFE{}",error);
    error
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
        let roll_request = dice_thrower::parse_request(
            &format!("d{}",&self.to_hit.to_string()));

        AttackResult { 
            attack_roll: dice_thrower::throw_roll(&roll_request.unwrap()) as u8,
            _roll_string: self.weapon.clone(),
        }
    }

    fn _defend_attack(){}

    fn take_damage(mut self, damage: u16) -> Self {
        self.hs2 = self.hs2.update_health_state(damage as i16);
        self
    }

    fn _react_to(){}
}

fn battle( players: &[CharacterStruct], battle_count: u32, arena_id: u8, report_level: ReportOutputLevel, sender: Sender<SendBuffer>) -> Result<String, SendError<SendBuffer>> {
    const DUMP_INCREMENT: u32 = WRITE_TO_FILE_TRIGGER;
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
        let mut current_battle = battle_order_list.clone();
        current_battle = current_battle.run_battle(battle_num, &report_level);
        current_battle.battle_result.battle_id = format!("{}{:0>6}", arena_id, battle_num);        
        current_battle.battle_result.initiative_winner = initiative_winner.team.to_string();
        battle_result_collection.battle_result_list.push(current_battle.battle_result.clone());

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

fn make_battle_order_list(players: &[characterize::CharacterStruct], report_level: &ReportOutputLevel) -> BattleOrderList {
    let mut battle_order_list: Vec<BattleOrder> = Vec::with_capacity(players.len());
    let initiative_die = "1d20".to_string();
    let roll_request = dice_thrower::parse_request(&initiative_die).unwrap();
    let battle_result: BattleResult = Default::default();
    
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
            report_level: ReportOutputLevel::Summary,
            battle_result },
        ReportOutputLevel::Accumulate => BattleOrderList {
            battle_order_list,
            report_level: ReportOutputLevel::Accumulate,
            battle_result },
        ReportOutputLevel::None => BattleOrderList {
            battle_order_list,
            report_level: ReportOutputLevel::None,
            battle_result },
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
    _roll_string: String,
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
    battle_result: BattleResult,
}

impl BattleOrderList {
    fn run_battle(mut self, battle_num: u32, report_level: &ReportOutputLevel) -> Self {
        let mut winning_result = false;
        let mut turn_number: u8 = 1;
        let mut turn_result: TurnResult = Default::default();
        // let turn_result_ref = &mut turn_result;
        self.battle_result = BattleResult {
            battle_id: battle_num.to_string(),
            initiative_winner: self.battle_order_list[0].character.name.clone(),
            ..Default::default()
        };
    
        while !winning_result {
            self = self.run_battle_turn(turn_number);
            self.battle_result.turn_result.push(turn_result.clone());

            if self.is_there_a_winner() {
                winning_result = true;
            }
            else {
                turn_number += 1;
            }
        }

        let winner = self.get_winner();
        if let Some(body) = winner {
            self.battle_result.winner = body.character.clone()
        }

        self.battle_result.turns_run = turn_number;
        if let ReportOutputLevel::Summary = report_level {
            let result_summary = self.battle_result.summarize();
            if let Some(summary) = result_summary {     
                println!("{}",summary);           
            }
        }
        
        self
    }
    
    fn run_battle_turn(mut self, turn_number: u8) -> Self {
        let mut action_result: ActionResult;
        let mut turn_order = self.battle_order_list.clone();
        let mut turn_result = TurnResult {
            turn_number,
            ..Default::default()
        }; 

        for i in 0..turn_order.len(){
            if turn_order[i].character.is_concious() {
                if let Some(target) = turn_order[i].get_target(&turn_order) {
                    let a_res = turn_order[i].make_attack();

                    if turn_order[target].is_attack_successful(&a_res) {
                        let d_res = turn_order[i].get_damage();
                        turn_order[target].give_damage(d_res.damage as u16);

                        action_result = ActionResult {
                            actor: turn_order[i].character.name.clone(),
                            target: turn_order[target].character.name.clone(),
                            action_type: ActionType::Attack,
                            action_roll: a_res.attack_roll,
                            action_result: ActionResultType::Hit,
                            action_damage: d_res.damage as u16, 
                            action_number: i as u16,
                        };
                    }
                    else {
                        action_result = ActionResult {
                            actor: turn_order[i].character.name.clone(),
                            target: turn_order[target].character.name.clone(),
                            action_type: ActionType::Attack,
                            action_roll: a_res.attack_roll,
                            action_result: ActionResultType::Miss,
                            action_damage: 0, 
                            action_number: i as u16,
                        };
                    }
                }
                else {
                    action_result = ActionResult {
                        actor: turn_order[i].character.name.clone(),
                        target: "no target".to_string(),
                        action_type: ActionType::_NoTarget,
                        action_roll: 0,
                        action_result: ActionResultType::Miss,
                        action_damage: 0, 
                        action_number: i as u16,
                    };
                }
                turn_result.action_results.push(action_result);
            }
        }
    self.battle_order_list = turn_order;
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
    use characterize::get_players;

    let players = get_players(); 
    let test_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);
    assert_ne!(test_list.battle_order_list.len(),0,"no list");
}

#[test]
fn test_order_of_make_battler_order_list() {
    use characterize::get_players;

    let players = get_players();
    let test_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);
    assert_eq!(test_list.battle_order_list[0].initative_roll > test_list.battle_order_list[1].initative_roll,true,"list not ordered");
}

#[test]
fn get_players_test() {
    use characterize::get_players;

    let list = get_players();
    assert_ne!(list.len(), 0, "Character get fail")
}

#[test]
fn select_target_test() {
    use characterize::get_players;

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
    use characterize::get_players;

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
    use characterize::get_players;

    let players = get_players();
    let mut actor = players[0].clone();
    let original_health_state = actor.hs2;

    actor = actor.take_damage(5);
    assert_ne!(original_health_state,actor.hs2);
}

#[test]
fn is_winner_test() {
    use characterize::get_players;

    let players = get_players();
    let order_list = make_battle_order_list(&players, &ReportOutputLevel::Summary);

    assert_eq!(order_list.is_there_a_winner(),false);

    let one_list = vec!(players[0].clone());
    let one_order_list = make_battle_order_list(&one_list, &ReportOutputLevel::Summary);

    assert_eq!(one_order_list.is_there_a_winner(),true);
}

#[test]
fn is_attack_successful() {
    use characterize::get_players;

    let players = get_players();
    let successful_attack = AttackResult {
        attack_roll: 20,
        _roll_string: "1d12".to_string(),
    };
    let failure_attack = AttackResult {
        attack_roll: 1,
        _roll_string: "1d12".to_string(),
    };

    assert_eq!(players[0].is_attack_successful(&successful_attack),true);
    assert_eq!(players[0].is_attack_successful(&failure_attack),false);
}