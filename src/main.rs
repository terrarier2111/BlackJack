use rand::Rng;
use std::io::{Error, ErrorKind, Write};
use std::ops::Range;
use std::str::FromStr;

fn main() {
    /// prompt the user for inputs to add another player
    fn prompt_player() -> (f64, String) {
        let name = input("How do you want to be called?\nPlease call me: ".to_owned()).unwrap();
        let mut money = 0.0;
        loop {
            let mut input =
                input("How much money do you want to bet?\nI want to bet: ".to_owned()).unwrap();
            if let Ok(amount) = input.parse::<f64>() {
                money = amount;
                break;
            } else {
                println!("Sorry, but i couldn't understand that!\nTry using a decimal number like \"1.53\" instead!");
            }
        }
        (money, name)
    }

    /// prompt the user to ask if they want to add another player
    fn prompt_continue() -> bool {
        loop {
            let mut input = input("Is there another user?\n".to_owned())
                .unwrap()
                .to_lowercase()
                .replace("no", "false")
                .replace("yes", "true");
            if let Ok(decision) = bool::from_str(input.as_str()) {
                return decision;
            } else {
                println!(
                    "Sorry, but i couldn't understand that!\nTry using \"yes\" or \"no\" instead!"
                );
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

fn start_game(player_entries: Vec<(f64, String)>) {
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

    let mut players = player_entries
        .into_iter()
        .map(|(money, name)| {
            let mut player = Player {
                name,
                ace: false,
                points: 0,
                money,
            };
            // initialize the players by giving them 2 cards
            player.add_card(card_pool.gen_decrementing());
            player.add_card(card_pool.gen_decrementing());
            println!("{} you start off with {}.", player.name, player.points());
            player
        })
        .collect::<Vec<Player>>();

    loop {
        let mut did_draw = false;
        for player in &mut players {
            if !player.is_finished() {
                let mut draw = false;
                loop {
                    let mut input = input(format!("{} do you want to draw again? | ", player.name))
                        .unwrap()
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
                    let val = card_pool.gen_decrementing();
                    player.add_card(val);
                    if player.points() > MAX_VAL {
                        println!(
                            "Unfortunately, {} you busted with {} points and lost {}!",
                            player.name,
                            player.points(),
                            player.money
                        );
                        player.money = 0.0;
                    } else {
                        println!("you drew: {}", val);
                        did_draw = true;
                        println!("You now have {} points, {}.", player.points(), player.name);
                    }
                }
            }
        }
        if !did_draw {
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
                println!(
                    "Congratz, {}, the dealer busted and you won {}!",
                    player.name, player.money
                );
                player.money *= 2.0;
            }
        }
    } else {
        for player in &mut players {
            if player.points() <= MAX_VAL {
                if player.points() > dealer.points {
                    println!(
                        "Congratz, {} you beat the dealer and won {}!",
                        player.name, player.money
                    );
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
    /// whether the dealer AI should draw another card or not
    fn do_draw_card(&self) -> bool {
        self.points < 17
    }

    /// call this when a specific card got drawn and added to the dealer's score
    fn add_card(&mut self, value: u8) {
        if value == 11 {
            if self.mode == DealerMode::Soft17 {
                self.points += 11;
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
    ace: bool,
    points: u8,
    money: f64,
}

impl Player {
    fn points(&self) -> u8 {
        if !self.ace {
            return self.points;
        }
        if self.points + 11 <= MAX_VAL {
            self.points + 11
        } else {
            self.points + 1
        }
    }

    fn add_ace(&mut self) {
        if self.ace {
            self.points += 1;
        } else {
            self.ace = true;
        }
    }

    fn add_card(&mut self, value: u8) {
        if value == 11 {
            self.add_ace();
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
    total_weight: u8,
}

impl WeightedProbability {
    fn new() -> Self {
        Self {
            entries: vec![],
            total_weight: 0,
        }
    }

    fn add_entry(&mut self, value: u8, weight: u8) -> &mut Self {
        self.entries.push((value, weight));
        self.total_weight += weight;
        self
    }

    /// generate a random value respecting the weights
    /// and decrement it's weight, can be used to simulate
    /// scarcity
    fn gen_decrementing(&mut self) -> u8 {
        // generate a random number between 0 and the sum of all weights(probabilities)
        let mut num = rand::thread_rng().gen_range(0..self.total_weight);
        for (value, weight) in self.entries.iter_mut() {
            // if the weight is greater than num, then we return the value associated with weight
            // and reduce weight and the accumulated weight by 1
            // otherwise we reduce num by weight

            // here is an example how a call of this method could look like:
            // num = 7
            // weights: [4, 4, 4, 4, 4]
            // values:  [2, 3, 4, 5, 6]
            // num > weight
            // num -= 4 => num = 3
            // num < weight
            // weights[1] -= 1 => weights[1] = 3
            // total_weight -= 1
            // return values[1] => return 3

            if num < *weight {
                *weight -= 1;
                self.total_weight -= 1;
                return *value;
            }
            num -= *weight;
        }
        unreachable!()
    }
}

/// prompts for input in the console with a specific message
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
