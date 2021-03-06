//! This is the core of the simulation.  It manages the various aspects of keeping track of the universe's
//! state and the state of all its cells and entities.  It drives the simulation forward by applying transformations
//! of the cells and processing actions of the entities sequentially.

use universe::Universe;
use cell::CellState;
use entity::{EntityState, MutEntityState};
use action::{CellAction, EntityAction};

pub mod serial;
// 90% sure this hella broken, and the code is terrible anyway.
// pub mod parallel;
pub mod iterator;

pub trait Engine<
    C: CellState,
    E: EntityState<C>,
    M: MutEntityState,
    CA: CellAction<C>,
    EA: EntityAction<C, E>,
    U: Universe<C, E, M>
> {
    /// The main function of the simulation process.  This is called repeatedly to drive progress in the simulation.
    fn step(&mut self, &mut U);
}
