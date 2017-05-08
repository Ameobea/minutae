//! These units reside in the a grid separate from the main universe but have the power to directly interact both with it and
//! with other entities.  They are the smallest units of discrete control in the simulation and are the only things that are
//! capable of mutating any aspect of the world outside of the simulation engine itself.

use std::cell::Cell as RustCell;
use std::clone::Clone;
use std::marker::PhantomData;

use test;
use uuid::Uuid;

use cell::CellState;

pub trait EntityState<C: CellState>:Clone {}

pub trait MutEntityState:Clone + Default {}

pub struct Entity<C: CellState, S: EntityState<C>, M: MutEntityState> {
    pub uuid: Uuid,
    pub state: S,
    pub mut_state: RustCell<M>,
    phantom: PhantomData<C>,
}

impl<C: CellState, E: EntityState<C>, M: MutEntityState> Clone for Entity<C, E, M> {
    fn clone(&self) -> Self {
        let mut_state_inner = self.mut_state.take();
        let mut_state_inner_clone = mut_state_inner.clone();
        self.mut_state.set(mut_state_inner_clone);
        Entity {
            uuid: Uuid::new_v4(),
            state: self.state.clone(),
            mut_state: RustCell::new(mut_state_inner),
            phantom: PhantomData,
        }
    }
}

impl<C: CellState, E: EntityState<C>, M: MutEntityState> Entity<C, E, M> {
    pub fn new(state: E, mut_state: M) -> Entity<C, E, M> {
        Entity {
            uuid: Uuid::new_v4(),
            state: state,
            mut_state: RustCell::new(mut_state),
            phantom: PhantomData,
        }
    }
}

#[bench]
fn uuid_v4(b: &mut test::Bencher) {
    b.iter(|| Uuid::new_v4())
}

#[bench]
fn uuid_pcg(b: &mut test::Bencher) {
    use pcg::PcgRng;
    use rand::Rng;

    let mut rng = PcgRng::new_unseeded();
    rng.set_stream(9182837465);
    let mut buf = vec![0u8; 16];

    b.iter(|| {
        rng.fill_bytes(&mut buf);
        Uuid::from_bytes(&buf)
    })
}
