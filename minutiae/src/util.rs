//! General-purpose utility functions

use std::{collections::HashMap, fmt::Debug};

use uuid::Uuid;

use cell::{Cell, CellState};
use container::EntityContainer;
use entity::{Entity, EntityState, MutEntityState};

pub type ColorCalculator<C, E, M> = fn(&Cell<C>, &[usize], &EntityContainer<C, E, M>) -> [u8; 4];

pub fn debug<T: Debug>(x: T) -> String { format!("{:?}", x) }

/// Given an index of the universe and the universe's size returns X and Y coordinates.
pub fn get_coords(index: usize, universe_size: usize) -> (usize, usize) {
    debug_assert!(index < universe_size * universe_size);
    let x = index % universe_size;
    let y = (index - x) / universe_size;
    (x, y)
}

/// Given an X and Y coordinate in the universe and the universe's size, returns the index of that
/// coordinate in the universe.
pub fn get_index(x: usize, y: usize, universe_size: usize) -> usize {
    debug_assert!(x < universe_size);
    debug_assert!(y < universe_size);
    y * universe_size + x
}

/// Calculates the manhattan distance between the two provided grid cells
pub fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    let x_diff = if x1 < x2 { x2 - x1 } else { x1 - x2 };
    let y_diff = if y1 < y2 { y2 - y1 } else { y1 - y2 };
    x_diff + y_diff
}

/// Calculates the offset between two coordinates; where point 2 is located relative to point 1
pub fn calc_offset(x1: usize, y1: usize, x2: usize, y2: usize) -> (isize, isize) {
    (x2 as isize - x1 as isize, y2 as isize - y1 as isize)
}

/// Searches one coordinate of the universe and attempts to find the entity ID of the entity with
/// the supplied UUID.
pub fn locate_entity_simple<C: CellState, E: EntityState<C>, M: MutEntityState>(
    uuid: Uuid,
    entities: &[Entity<C, E, M>],
) -> Option<usize> {
    entities.iter().position(|&ref entity| entity.uuid == uuid)
}

pub enum EntityLocation {
    Deleted, // entity no longer exists
    Expected(usize), /* entity is where it's expecte to be with the returned entity
              * index */
    Moved(usize, usize, usize), // entity moved to (arg 0, arg 1) with entity index arg 2
}

/// Attempts to find the entity index of an entity with a specific UUID and a set of coordinates
/// where it is expected to be located.
pub fn locate_entity<C: CellState, E: EntityState<C>, M: MutEntityState>(
    entities: &[Vec<Entity<C, E, M>>],
    uuid: Uuid,
    expected_index: usize,
    entity_meta: &HashMap<Uuid, (usize, usize)>,
    universe_size: usize,
) -> EntityLocation {
    debug_assert!(expected_index < (universe_size * universe_size));
    // first attempt to find the entity at its expected coordinates
    match locate_entity_simple(uuid, &entities[expected_index]) {
        Some(entity_index) => EntityLocation::Expected(entity_index),
        None => {
            // unable to locate entity at its expected coordinates, so check the coordinates in the
            // meta `HashMap`
            match entity_meta.get(&uuid) {
                Some(&(real_x, real_y)) => {
                    let real_index = get_index(real_x, real_y, universe_size);
                    let entity_index = locate_entity_simple(uuid, &entities[real_index])
                        .expect("Entity not present at coordinates listed in meta `HashMap`!");

                    EntityLocation::Moved(real_x, real_y, entity_index)
                },
                // If no entry in the `HashMap`, then the entity has been deleted.
                None => EntityLocation::Deleted,
            }
        },
    }
}

/// Given current X and Y coordinates of an entity and the view distance of the universe, creates an
/// iterator visiting the indexes of all visible grid coordinates.  Note that this will include the
/// index of the source entity.
pub fn iter_visible(
    cur_x: usize,
    cur_y: usize,
    view_distance: usize,
    universe_size: usize,
) -> impl Iterator<Item = (usize, usize)> {
    let y_range =
        cur_y.saturating_sub(view_distance)..=(cur_y + view_distance).min(universe_size - 1);
    y_range.flat_map(move |y| {
        let x_range =
            cur_x.saturating_sub(view_distance)..=(cur_x + view_distance).min(universe_size - 1);
        x_range.map(move |x| (x, y))
    })
}

pub fn translate_entity<CS: CellState, ES: EntityState<CS>, MES: MutEntityState>(
    x_offset: isize,
    y_offset: isize,
    entities: &mut EntityContainer<CS, ES, MES>,
    entity_index: usize,
    entity_uuid: Uuid,
    universe_size: usize,
) {
    // this function will return early if the entity has been deleted
    let universe_index = match entities.get_verify(entity_index, entity_uuid) {
        Some((_, universe_index)) => universe_index,
        None => {
            return;
        }, // entity has been deleted, so do nothing.
    };

    // if this is the entity that we're looking for, check to see if the requested move is in bounds
    let (cur_x, cur_y) = get_coords(universe_index, universe_size);
    let new_x = cur_x as isize + x_offset;
    let new_y = cur_y as isize + y_offset;
    let dst_universe_index = get_index(new_x as usize, new_y as usize, universe_size);

    // verify that the supplied desination coordinates are in bounds
    // TODO: verify that the supplied destination coordinates are within ruled bounds of destination
    if new_x >= 0 && new_x < universe_size as isize && new_y >= 0 && new_y < universe_size as isize
    {
        entities.move_entity(entity_index, dst_universe_index);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Color(pub [u8; 3]);

#[test]
fn iter_visible_functionality() {
    let universe_size = 50;
    let mut view_distance = 3;
    let mut cur_x = 6;
    let mut cur_y = 6;

    let indexes: Vec<(usize, usize)> =
        iter_visible(cur_x, cur_y, view_distance, universe_size).collect();
    assert_eq!(indexes.len(), 49);

    view_distance = 4;
    cur_x = 3;
    cur_y = 2;
    let indexes: Vec<(usize, usize)> =
        iter_visible(cur_x, cur_y, view_distance, universe_size).collect();

    assert_eq!(indexes.len(), 56);
}

#[test]
fn manhattan_distance_accuracy() {
    let x1 = 1;
    let y1 = 5;
    let x2 = 4;
    let y2 = 0;

    assert_eq!(manhattan_distance(x1, y1, x2, y2), 8);
}
