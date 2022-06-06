use std::io::{Error, ErrorKind, Write};
use std::ops::Range;
use std::str::FromStr;
use rand::Rng;

fn main() {
    fn prompt_player() -> (f64, String) {
        let name = input("How do you want to be called?\nPlease call me: ".to_owned()).unwrap();
        let mut money = 0.0;
        loop {
            let mut input = input("How much money do you want to bet?\nI want to bet: ".to_owned()).unwrap();
            if let Ok(amount) = input.parse::<f64>() {
                money = amount;
                break;
            } else {
                println!("Sorry, but i couldn't understand that!\nTry using a decimal number like \"1.53\" instead!");
            }
        }
        (money, name)
    }

    fn prompt_continue() -> bool {
        loop {
            let mut input = input("Is there another user?\n".to_owned()).unwrap()
                .to_lowercase()
                .replace("no", "false")
                .replace("yes", "true");
            if let Ok(decision) = bool::from_str(input.as_str()) {
                return decision;
            } else {
                println!("Sorry, but i couldn't understand that!\nTry using \"yes\" or \"no\" instead!");
            }
        }
    }

    let mut players = vec![prompt_player()];

    while prompt_continue() {
        players.push(prompt_player());
    }

    start_game(players);
}

// when a player wins by going over the dealers score, they get an additional 1.5 times their bet (money *= 2.5)
// when a dealer busts, players get an additional 1 times their bet (money *= 2)

const MAX_VAL: u8 = 21;

fn start_game(/*min_bet: f64, */player_entries: Vec<(f64, String)>) {
    let mut card_pool = WeightedProbability::new();
    card_pool
        .add_entry(2, 4)
        .add_entry(3, 4)
        .add_entry(4, 4)
        .add_entry(5, 4)
        .add_entry(6, 4)
        .add_entry(7, 4)
        .add_entry(8, 4)
        .add_entry(9, 4)
        .add_entry(10, 4 * 4)
        .add_entry(11, 4);
    let mut dealer = Dealer {
        mode: DealerMode::Soft17,
        points: 0,
    };
    // initialize the dealer by giving him 1 card
    dealer.add_card(card_pool.gen_decrementing());

    let mut players = player_entries.into_iter().map(|(money, name)| {
        let mut player = Player {
            name,
            ass: false,
            points: 0,
            money
        };
        // initialize the players by giving them 2 cards
        player.add_card(card_pool.gen_decrementing());
        player.add_card(card_pool.gen_decrementing());
        println!("{} you start off with {}.", player.name, player.points());
        player
    }).collect::<Vec<Player>>();

    loop {
        let mut drew = false;
        for player in &mut players {
            if !player.is_finished() {
                let mut draw = false;
                loop {
                    let mut input = input(format!("{} do you want to draw again? | ", player.name)).unwrap()
                        .to_lowercase()
                        .replace("no", "false")
                        .replace("yes", "true");
                    if let Ok(decision) = bool::from_str(input.as_str()) {
                        draw = decision;
                        break;
                    } else {
                        println!("Sorry, but i couldn't understand that!\nTry using \"yes\" or \"no\" instead!");
                    }
                }
                if draw {
                    player.add_card(card_pool.gen_decrementing());
                    if player.points() > MAX_VAL {
                        println!("Unfortunately, {} you busted with {} points and lost {}!", player.name, player.points(), player.money);
                        player.money = 0.0;
                    } else {
                        drew = true;
                        println!("You now have {} points, {}.", player.points(), player.name);
                    }
                }
            }
        }
        if !drew {
            break;
        }
    }
    while dealer.do_draw_card() {
        dealer.add_card(card_pool.gen_decrementing());
    }
    // dealer.add_card(card_pool.gen_decrementing());

    if dealer.points > MAX_VAL {
        for player in &mut players {
            if player.points() <= MAX_VAL {
                println!("Congratz, {}, the dealer busted and you won {}!", player.name, player.money);
                player.money *= 2.0;
            }
        }
    } else {
        for player in &mut players {
            if player.points() <= MAX_VAL {
                if player.points() > dealer.points {
                    println!("Congratz, {} you beat the dealer and won {}!", player.name, player.money);
                    player.money *= 2.5;
                } else {
                    println!("Unfortunately the bank has more points than you (it has {} points) you lost {}, {}.", dealer.points, player.money, player.name);
                }
            }
        }
    }

}

#[derive(PartialEq)]
enum DealerMode {
    Soft17,
    Hard17,
}

struct Dealer {
    mode: DealerMode,
    points: u8,
}

impl Dealer {

    fn do_draw_card(&self) -> bool {
        self.points < 17
    }

    fn add_card(&mut self, value: u8) {
        if value == 11 {
            if self.mode == DealerMode::Soft17 {
                self.points += 11; // FIXME: is this correct?
            } else {
                self.points += 1;
            }
        } else {
            self.points += value;
        }
    }

}

struct Player {
    name: String,
    ass: bool,
    points: u8,
    money: f64,
}

impl Player {

    fn points(&self) -> u8 {
        if !self.ass {
            return self.points;
        }
        if self.points + 11 <= MAX_VAL {
            self.points + 11
        } else {
            self.points + 1
        }
    }

    fn add_ass(&mut self) {
        if self.ass {
            self.points += 1;
        } else {
            self.ass = true;
        }
    }

    fn add_card(&mut self, value: u8) {
        if value == 11 {
            self.add_ass();
        } else {
            self.points += value;
        }
    }

    fn is_finished(&self) -> bool {
        self.points() >= MAX_VAL
    }

}

struct WeightedProbability {
    entries: Vec<(u8, u8)>, // value, weight
    all: u8,
}

impl WeightedProbability {
    
    fn new() -> Self {
        Self {
            entries: vec![],
            all: 0,
        }
    }
    
    fn add_entry(&mut self, value: u8, weight: u8) -> &mut Self {
        self.entries.push((value, weight));
        self.all += weight;
        self
    }

    /// generate a random value respecting the weights
    /// and decrement it's weight, can be used to simulate
    /// scarcity
    fn gen_decrementing(&mut self) -> u8 {
        let mut num = rand::thread_rng().gen_range(0..self.all);
        for (value, weight) in self.entries.iter_mut() {
            // FIXME: remove 0 entries
            if num < *weight {
                *weight -= 1;
                self.all -= 1;
                return *value;
            }
            num -= *weight;
        }
        unreachable!()
    }
    
}

fn input(text: String) -> std::io::Result<String> {
    print!("{}", text);
    std::io::stdout().flush()?; // because print! doesn't flush
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input)? == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "EOF while reading a line",
        ));
    }
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    Ok(input)
}
