use std::fmt;
use std::thread;
// use std::fmt::Write;
use rand::Rng;
use std::collections::HashMap;

fn main() -> Result<(),String> {
    let player_vec = get_players();

    let desired_iterations = 5_000;
    let threads_desired: u8 = 1;
    let thread_iterations = desired_iterations/threads_desired as u32;

    let mut battle_collection_list:Vec<BattleResultCollection> = Vec::with_capacity(6);
    let mut thread_list: Vec<thread::JoinHandle<_>> = Vec::with_capacity(6);

    for i in 0..threads_desired as usize{
        let local_player_vec = player_vec.clone();
        thread_list.push(thread::spawn(move||
            {
                battle(&local_player_vec, thread_iterations, i as u8, ReportOutputLevel::Summary)
            }));   
    }
 
    for thread_counter in thread_list {
        battle_collection_list.push(thread_counter.join().unwrap());
    }

    for battle in &battle_collection_list {
        // let summary = battle.summarize().expect("something");
        let collection = battle.summarize().expect("something");
                println!("Battle Summary: ");
                println!("{}", collection.battle_count );
                println!("{}", collection);
        }

    for battle in &battle_collection_list {
        let collection = battle.accumulate_summary().expect("accumulator says not");
                println!("Battle Accumlation: ");
                println!("{}", collection.number_of_battles);
                println!("{}", collection)
    }

    Ok(())
}

fn get_players() -> Vec<CharacterStruct> {
    let player1 = CharacterStruct {
        name: String::from("Hero"),
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

fn battle( players: &[CharacterStruct], battle_count: u32, arena_id: u8, report_level: ReportOutputLevel) -> BattleResultCollection {
    let battle_order_list = make_battle_order_list(players, &report_level);
    let mut battle_result_collection = BattleResultCollection {
        battle_order_list: battle_order_list.battle_order_list.clone(),
        battle_count,
        arena_id,
        ..Default::default()
    };

    let initiative_winner = &battle_order_list.battle_order_list.iter().max_by_key(|p| p.initative_roll).expect("duff list");
    battle_result_collection.battle_order_list = battle_order_list.battle_order_list.clone();

    for battle_num in 0..battle_count {
        let mut battle_result = battle_order_list.clone().run_battle(battle_num, &report_level);
        battle_result.battle_id = format!("{}{:0>6}", arena_id, battle_num);        // sus
        battle_result.initiative_winner = initiative_winner.character.name.clone();
        battle_result_collection.battle_result_list.push(battle_result.clone());
    }

    battle_result_collection
}

fn make_battle_order_list(players: &[CharacterStruct], report_level: &ReportOutputLevel) -> BattleOrderList {
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


#[derive(Clone, Debug)]
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
}

struct TurnResultAccumulator {
}

impl Summary<TurnResultSummary, TurnResultAccumulator> for TurnResult {
    fn summarize(&self) -> Option<TurnResultSummary>{
        todo!("can I wrote here?");
    }
    
    fn accumulate_summary(&self) -> Option<TurnResultAccumulator> {
        todo!();
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

impl BattleOrderList{
    fn run_battle(mut self, battle_num: u32, report_level: &ReportOutputLevel) -> BattleResult {
        let mut winning_result = false;
        let mut turn_count: u8 = 0;
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

trait Summary <T, B> {
    fn summarize(&self) -> Option<T>;
    fn accumulate_summary(&self) -> Option<B>;
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

impl fmt::Display for CollectionSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.arena_id, self.battle_count, self.total_turns_run, self.average_turns_run, self._max_turns_run)
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
            let write_row = format!("{:?} {}\n", row.0, row.1);
            output.push_str(&write_row);
        }
        write!(f, "{}", output)
    }    
}

impl Summary<CollectionSummary, CollectionAccumulation> for BattleResultCollection {
    fn summarize(&self ) -> Option<CollectionSummary> {
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
        Some(battle_collection_summary)
    }

    fn accumulate_summary(&self) -> Option<CollectionAccumulation>{
        let mut number_of_battles: u32 = 0;
        let mut accumulation:HashMap<(u16,String),u32> = HashMap::new();

        for battle in &self.battle_result_list {
            number_of_battles += 1;
            let res = battle.summarize();

            if let Some(battle_summary) = res {
                let winner = if battle_summary.winner == battle_summary.initiative_winner { 
                    format!("{:?}*", battle_summary.winning_team)
                }
                else { format!("{:?}", battle_summary.winning_team) };
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

struct BattleAccumulator {

}

impl Summary<BattleSummary, BattleAccumulator> for BattleResult {
    fn summarize(&self) -> Option<BattleSummary> {
        let battle_summary = BattleSummary {
            battle_id: self.battle_id.clone(), 
            turns_run: self.turns_run as u8, 
            winner: self.winner.name.clone(),
            initiative_winner: self.initiative_winner.clone(),
            winning_team: self.winner.team,
        };
        Some(battle_summary)
    }

    fn accumulate_summary(&self) -> Option<BattleAccumulator>{
        todo!()
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