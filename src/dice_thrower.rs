use rand::Rng;

pub fn throw_roll(roll_request_list: &RollRequest) -> u16 {
    let mut roll_result= Vec::new();
    let mut mod_result = 0_u16;

    for token in &roll_request_list.request_list {
        match token {
            RequestToken::Dice(roll) => roll_result.push(roll_dice(roll.number_of_dice, roll.number_of_die_sides)),
            RequestToken::Modifier(modifier) => mod_result += modifier.value as u16, 
            RequestToken::Error => println!("Can't Throw. Your token makes no sense!"),
        }
    }
    mod_result + roll_result.iter().fold(0_u16, |sum, val| sum + val.total)
}

pub fn parse_request(request_string: &str) -> Option<RollRequest> {
    let tokens = tokenize(request_string.to_owned());
    Some(RollRequest{request_list:tokens})
}

fn roll_dice(num_dice: u16, num_sides: u8) -> RollResult{
    let mut rng = rand::thread_rng();
    let mut roll_result = RollResult { total: 0, };

    for _i in 1..=num_dice {
        roll_result.total += rng.gen_range(1..=num_sides) as u16;
    }
    println!("{} {} {}", num_dice, num_sides,roll_result.total);
    roll_result
}

struct RollResult {
    total: u16,
    // request_token: RequestToken,
}

#[derive(Debug, Clone)]
struct DiceThrowDescription {
    number_of_dice: u16,
    number_of_die_sides: u8,
}

#[derive(Debug, Clone)]
struct DiceThrowModifier {
    operator: ModifierOperators,
    value: u8,
}

#[derive(Debug, Clone)]
pub struct RollRequest {
    request_list: Vec<RequestToken>,
}

#[derive(Debug, Clone)]
enum ModifierOperators {
    Add,
    _Subtract,
    _Multiply,
    _Divide,
}

#[derive(Debug, Clone)]
enum RequestToken {
    Dice(DiceThrowDescription),
    Modifier(DiceThrowModifier),
    Error,
}

fn tokenize(mut request_string: String) -> Vec<RequestToken> {
    request_string.retain(|c| !c.is_whitespace());
    let general_token_list: Vec<&str> = request_string.rsplit('+').collect();
    let mut request_token_list: Vec<RequestToken> = Vec::new();
    for general_token in general_token_list {
        let request_token = make_request_token(general_token);


        request_token_list.push(request_token);
    }
    request_token_list
}


fn make_request_token(in_token: &str) -> RequestToken {
    let token_parts = in_token.split('d').collect::<Vec<&str>>();

    match token_parts.len() {
        1 => return RequestToken::Modifier(DiceThrowModifier {operator: ModifierOperators::Add,
            value: token_parts[0].parse().unwrap()}),
        2 => return RequestToken::Dice(DiceThrowDescription {number_of_dice: token_parts[0].parse().unwrap_or(1),
            number_of_die_sides: token_parts[1].parse().unwrap()}),
        _ => return RequestToken::Error,
        };
}