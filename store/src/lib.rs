pub use bevy::prelude::*;
use map::components::CubeCoords;
use serde::{Deserialize, Serialize};
use ships::ShipType;
use std::collections::{HashMap, VecDeque};

use crate::ships::SHIPS;

pub mod camera;
pub mod map;
pub mod ships;

/// Struct for storing player related data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
}

/// An event that progresses the GameState forward
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub enum GameEvent {
    BeginGame {
        first_player: PlayerId,
    },
    EndGame {
        reason: EndGameReason,
    },
    PlayerJoined {
        player_id: PlayerId,
        player_details: Player,
    },
    PlayerDisconnected {
        player_id: PlayerId,
    },
    // PlayerSelects {
    //     player_id: PlayerId,
    //     select_box: SelectQuad,
    // },
    ShipMove {
        player_id: PlayerId,
        at: CubeCoords,
    },
    ShipPlaced {
        player_id: PlayerId,
        at: CubeCoords,
        rotation: u32,
    },
}

/// The different states a game can be in. (not to be confused with the entire "GameState")
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameStage {
    Lobby,
    PreGame,
    InGame,
    Paused,
    Ended,
}

/// This just makes it easier to dissern between a player id and any ol' u64
type PlayerId = u64;

/// A GameState object that is able to keep track of a game of TicTacTussle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
pub struct GameState {
    pub stage: GameStage,
    pub players: HashMap<PlayerId, Player>,
    pub players_garage: HashMap<PlayerId, VecDeque<ShipType>>,
    pub history: Vec<GameEvent>,
    pub cur_player: Option<PlayerId>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            stage: GameStage::Lobby,
            players: HashMap::new(),
            players_garage: HashMap::new(),
            history: Vec::new(),
            cur_player: None,
        }
    }
}

impl GameState {
    /// Determines whether an event is valid considering the current GameState
    pub fn validade(&self, event: &GameEvent) -> bool {
        use GameEvent::*;
        match event {
            BeginGame { first_player } => {
                if None == self.players.get(first_player) {
                    return false;
                }
                if self.players.len() != 2 {
                    return false;
                }
            }
            EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    if self.stage != GameStage::InGame {
                        return false;
                    }
                }
                _ => {}
            },
            PlayerJoined {
                player_id,
                player_details: _,
            } => {
                if self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerDisconnected { player_id } => {
                if !self.players.contains_key(player_id) {
                    return false;
                }
            }
            ShipMove { player_id, at: _ } => return self.is_player_turn(player_id),
            ShipPlaced {
                player_id,
                at,
                rotation,
            } => {
                if self.stage != GameStage::PreGame {
                    return false;
                }
                match self.players_garage.get(player_id) {
                    Some(garage) => {
                        if garage.len() == 0 {
                            return false;
                        }
                    }
                    None => {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn consume(&mut self, valid_event: &GameEvent) {
        use GameEvent::*;
        match valid_event {
            BeginGame { first_player } => {
                self.cur_player = Some(*first_player);
                trace!("First player: {:?}", *first_player);
                for player in self.players.keys() {
                    let mut deque = VecDeque::new();
                    for ship in SHIPS {
                        deque.push_back(ship);
                    }
                    self.players_garage.insert(*player, deque);
                }
                self.stage = GameStage::PreGame;
            }
            EndGame { reason: _ } => self.stage = GameStage::Ended,
            PlayerDisconnected { player_id } => {
                self.players.remove(player_id);
            }
            PlayerJoined {
                player_id,
                player_details,
            } => {
                self.players.insert(*player_id, player_details.clone());
            }
            ShipMove {
                player_id: _,
                at: _,
            } => {
                self.cur_player = self.next_player();
            }
            ShipPlaced {
                player_id,
                at: _,
                rotation: _,
            } => {
                let player_ships = self
                    .players_garage
                    .get_mut(player_id)
                    .expect("expected garage");

                player_ships
                    .pop_front()
                    .expect("expected ships available at garage");

                let mut ships_remainder = 0;
                for player in self.players.keys() {
                    ships_remainder += self.players_garage.get(player).unwrap().len();
                }
                trace!("ships to place: {:?}", ships_remainder);
                if ships_remainder == 0 {
                    self.stage = GameStage::InGame;
                }
            }
        }

        self.history.push(valid_event.clone());
    }

    fn next_player(&self) -> Option<PlayerId> {
        if let Some(player_moved) = self.cur_player {
            for (key, _) in self.players.iter() {
                if player_moved != *key {
                    return Some(*key);
                }
            }
        }
        None
    }

    fn is_player_turn(&self, player_id: &PlayerId) -> bool {
        if let Some(p) = self.cur_player {
            if *player_id == p {
                return true;
            }
        }
        false
    }
}

/// The various reasons why a game could end
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum EndGameReason {
    PlayerLeft { player_id: PlayerId },
    PlayerWon { winner: PlayerId },
}
